import { type ServerType, serve } from "@hono/node-server"
import { Hono } from "hono"
import { streamSSE } from "hono/streaming"
import * as Rpc from "../protocol/jsonrpc.js"
import type { Transport, TransportFactory } from "./types.js"

// ============================================================================
// Options
// ============================================================================

export interface HttpOptions {
	/** Port to listen on (default: 3000) */
	readonly port?: number
	/** Hostname to bind to (default: localhost) */
	readonly hostname?: string
	/** Path prefix for MCP endpoints (default: /mcp) */
	readonly basePath?: string
	/** CORS origin (set to "*" for all, or specific origin) */
	readonly cors?: string
	/** Error handler */
	readonly onError?: (error: Error) => void
}

// ============================================================================
// HTTP Transport Factory
// ============================================================================

/**
 * Create an HTTP transport using Hono.
 * Works in both Node.js and Bun environments.
 *
 * Endpoints:
 * - POST /mcp - JSON-RPC request/response
 * - GET /mcp/sse - Server-Sent Events stream
 * - GET /mcp/health - Health check
 *
 * @example
 * ```ts
 * createServer({
 *   tools: { ping },
 *   transport: http({ port: 3000 })
 * })
 * ```
 */
export const http = (options: HttpOptions = {}): TransportFactory => {
	return (server, _notify): Transport => {
		const port = options.port ?? 3000
		const hostname = options.hostname ?? "localhost"
		const basePath = options.basePath ?? "/mcp"
		const cors = options.cors

		const app = new Hono()

		// Session storage for SSE
		const sessions = new Map<
			string,
			{
				send: (event: string, data: string) => Promise<void>
			}
		>()

		// CORS middleware
		if (cors) {
			app.use("*", async (c, next) => {
				c.header("Access-Control-Allow-Origin", cors)
				c.header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
				c.header("Access-Control-Allow-Headers", "Content-Type, X-Session-ID")

				if (c.req.method === "OPTIONS") {
					return c.body(null, 204)
				}

				await next()
			})
		}

		// SSE endpoint - establish stream
		app.get(`${basePath}/sse`, (c) => {
			const sessionId = crypto.randomUUID()

			return streamSSE(c, async (stream) => {
				// Store send function for this session
				sessions.set(sessionId, {
					send: async (event: string, data: string) => {
						await stream.writeSSE({ event, data })
					},
				})

				// Send session ID
				await stream.writeSSE({
					event: "session",
					data: JSON.stringify({ sessionId }),
				})

				// Keep connection open
				while (true) {
					await stream.sleep(30000)
				}
			})
		})

		// SSE message endpoint
		app.post(`${basePath}/sse`, async (c) => {
			const sessionId = c.req.header("X-Session-ID")
			if (!sessionId) {
				return c.json({ error: "Missing X-Session-ID header" }, 400)
			}

			const session = sessions.get(sessionId)
			if (!session) {
				return c.json({ error: "Session not found" }, 404)
			}

			try {
				const body = await c.req.text()
				const response = await server.handle(body)

				if (response) {
					await session.send("message", response)
				}

				return c.json({ ok: true })
			} catch (error) {
				options.onError?.(error instanceof Error ? error : new Error(String(error)))
				return c.json({ error: "Internal error" }, 500)
			}
		})

		// Standard JSON-RPC endpoint
		app.post(basePath, async (c) => {
			try {
				const body = await c.req.text()
				const response = await server.handle(body)

				if (!response) {
					return c.body(null, 204)
				}

				return c.json(JSON.parse(response))
			} catch (error) {
				options.onError?.(error instanceof Error ? error : new Error(String(error)))
				const errorResponse = Rpc.error(null, Rpc.ErrorCode.InternalError, "Internal server error")
				return c.json(JSON.parse(Rpc.stringify(errorResponse)), 500)
			}
		})

		// Health check
		app.get(`${basePath}/health`, (c) => {
			return c.json({
				status: "ok",
				server: server.name,
				version: server.version,
			})
		})

		let nodeServer: ServerType | null = null

		const start = async (): Promise<void> => {
			nodeServer = serve({
				fetch: app.fetch,
				port,
				hostname,
			})
		}

		const stop = async (): Promise<void> => {
			sessions.clear()
			if (nodeServer) {
				nodeServer.close()
				nodeServer = null
			}
		}

		return { start, stop }
	}
}
