/**
 * HTTP Transport
 *
 * MCP Streamable HTTP Transport using @sylphx/gust
 *
 * Implements the MCP 2025-03-26 Streamable HTTP specification:
 * - Single POST endpoint for JSON-RPC messages
 * - Supports both JSON and SSE response formats
 * - SSE enables server-to-client notifications during request processing
 */

import {
	type Context,
	compose,
	get,
	cors as gustCors,
	json,
	post,
	response,
	router,
	type Server,
	serve,
} from "@sylphx/gust"
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
// SSE Helpers
// ============================================================================

/** Format an SSE event */
const sseEvent = (data: string, event?: string, id?: string): string => {
	let result = ""
	if (id) result += `id: ${id}\n`
	if (event) result += `event: ${event}\n`
	result += `data: ${data}\n\n`
	return result
}

// ============================================================================
// HTTP Transport Factory
// ============================================================================

/**
 * Create a Streamable HTTP transport using @sylphx/gust.
 * Works in both Node.js and Bun environments.
 *
 * Implements MCP Streamable HTTP (2025-03-26):
 * - POST /mcp - JSON-RPC with optional SSE streaming
 * - GET /mcp/health - Health check
 *
 * When client includes `Accept: text/event-stream`, server may respond
 * with SSE to stream notifications during request processing.
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

		// Session storage for Mcp-Session-Id
		const sessions = new Map<string, { createdAt: number }>()

		// Routes
		const jsonRpcRoute = post(basePath, async (ctx: Context) => {
			try {
				const body = ctx.body.toString()
				const accept = ctx.headers.accept ?? ""
				const acceptsSSE = accept.includes("text/event-stream")

				// Check for session ID (optional in Streamable HTTP)
				const sessionId = ctx.headers["mcp-session-id"]
				if (sessionId && !sessions.has(sessionId)) {
					return json({ error: "Session not found" }, { status: 404 })
				}

				if (acceptsSSE) {
					// Streamable HTTP: Use SSE for response
					// This allows sending notifications during processing
					const notifications: string[] = []

					// Collect notifications during handler execution
					const notify = (method: string, params?: unknown) => {
						const notification = Rpc.notification(method, params)
						notifications.push(Rpc.stringify(notification))
					}

					// Execute handler and collect response
					const responseStr = await server.handle(body, { notify })

					// Build SSE body
					let sseBody = ""
					for (const notification of notifications) {
						sseBody += sseEvent(notification, "message")
					}
					if (responseStr) {
						sseBody += sseEvent(responseStr, "message")
					}

					// Generate session ID on initialize response
					const parsed = Rpc.parseMessage(body)
					const headers: Record<string, string> = {
						"Content-Type": "text/event-stream",
						"Cache-Control": "no-cache",
					}

					// Create session on initialize
					if (parsed.ok && Rpc.isRequest(parsed.value) && parsed.value.method === "initialize") {
						const newSessionId = crypto.randomUUID()
						sessions.set(newSessionId, { createdAt: Date.now() })
						headers["Mcp-Session-Id"] = newSessionId
					}

					return response(sseBody, { status: 200, headers })
				}

				// Regular JSON response (no streaming)
				const responseStr = await server.handle(body)

				if (!responseStr) {
					return response("", { status: 204 })
				}

				// Generate session ID on initialize
				const parsed = Rpc.parseMessage(body)
				const headers: Record<string, string> = {}

				if (parsed.ok && Rpc.isRequest(parsed.value) && parsed.value.method === "initialize") {
					const newSessionId = crypto.randomUUID()
					sessions.set(newSessionId, { createdAt: Date.now() })
					headers["Mcp-Session-Id"] = newSessionId
				}

				return json(JSON.parse(responseStr), { headers })
			} catch (error) {
				console.error("HTTP Transport Error:", error)
				options.onError?.(error instanceof Error ? error : new Error(String(error)))
				const errorResponse = Rpc.error(null, Rpc.ErrorCode.InternalError, "Internal server error")
				return json(JSON.parse(Rpc.stringify(errorResponse)), { status: 500 })
			}
		})

		const healthRoute = get(`${basePath}/health`, () => {
			return json({
				status: "ok",
				server: server.name,
				version: server.version,
			})
		})

		// Build app with optional CORS
		const routes = router({
			jsonRpc: jsonRpcRoute,
			health: healthRoute,
		})

		const app = options.cors
			? compose(gustCors({ origin: options.cors }))(routes.handler)
			: routes.handler

		let gustServer: Server | null = null

		const start = async (): Promise<void> => {
			gustServer = await serve({
				fetch: app,
				port,
				hostname,
			})
		}

		const stop = async (): Promise<void> => {
			sessions.clear()
			if (gustServer) {
				await gustServer.stop()
				gustServer = null
			}
		}

		return { start, stop }
	}
}
