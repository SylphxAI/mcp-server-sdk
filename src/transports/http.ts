/**
 * HTTP Transport (Streamable HTTP / SSE)
 *
 * Uses Bun.serve for high-performance HTTP server.
 * Supports both request/response and SSE for streaming.
 */

import type { PromptContext } from "../builders/prompt.js"
import type { ResourceContext } from "../builders/resource.js"
import type { ToolContext } from "../builders/tool.js"
import { type NotificationEmitter, createEmitter } from "../notifications/index.js"
import * as Rpc from "../protocol/jsonrpc.js"
import type { HandlerContext, NotificationContext } from "../server/handler.js"
import type { Server as McpServer } from "../server/server.js"

// ============================================================================
// Transport Types
// ============================================================================

export interface HttpTransport {
	/** Start the HTTP server */
	readonly start: () => Promise<void>
	/** Stop the HTTP server */
	readonly stop: () => Promise<void>
	/** Get the server URL */
	readonly url: string
	/** Broadcast notification to all SSE sessions */
	readonly broadcast: NotificationEmitter
	/** Get notification emitter for specific session */
	readonly getSessionNotifier: (sessionId: string) => NotificationEmitter | undefined
}

export interface HttpOptions<
	TToolCtx extends ToolContext,
	TResourceCtx extends ResourceContext,
	TPromptCtx extends PromptContext,
> {
	/** Port to listen on */
	readonly port?: number
	/** Hostname to bind to */
	readonly hostname?: string
	/** Path prefix for MCP endpoints */
	readonly basePath?: string
	/** Custom context factory (receives request and session-specific notification emitter) */
	readonly createContext?: (
		req: Request,
		notify: NotificationEmitter
	) => HandlerContext<TToolCtx, TResourceCtx, TPromptCtx>
	/** CORS origin (set to "*" for all, or specific origin) */
	readonly cors?: string
	/** Error handler */
	readonly onError?: (error: Error) => void
}

// ============================================================================
// Session Management
// ============================================================================

interface Session {
	readonly id: string
	readonly createdAt: number
	controller: ReadableStreamDefaultController<Uint8Array> | null
	readonly notify: NotificationEmitter
}

// ============================================================================
// HTTP Transport Factory
// ============================================================================

/**
 * Create an HTTP transport using Bun.serve.
 *
 * Endpoints:
 * - POST /mcp - JSON-RPC request/response
 * - GET /mcp/sse - Server-Sent Events stream
 * - POST /mcp/sse - Send message to SSE stream
 *
 * @example
 * ```ts
 * const server = createServer({ ... })
 * const transport = http(server, { port: 3000 })
 * await transport.start()
 * console.log(`Server running at ${transport.url}`)
 * ```
 */
export const http = <
	TToolCtx extends ToolContext,
	TResourceCtx extends ResourceContext,
	TPromptCtx extends PromptContext,
>(
	server: McpServer<TToolCtx, TResourceCtx, TPromptCtx>,
	options: HttpOptions<TToolCtx, TResourceCtx, TPromptCtx> = {}
): HttpTransport => {
	const port = options.port ?? 3000
	const hostname = options.hostname ?? "localhost"
	const basePath = options.basePath ?? "/mcp"
	const cors = options.cors
	const encoder = new TextEncoder()

	// Session storage
	const sessions = new Map<string, Session>()

	// Create session notification sender
	const createSessionNotifier = (sessionId: string): NotificationEmitter => {
		return createEmitter((method, params) => {
			const session = sessions.get(sessionId)
			if (!session?.controller) return
			const message = Rpc.notification(method, params)
			const data = `event: message\ndata: ${Rpc.stringify(message)}\n\n`
			session.controller.enqueue(encoder.encode(data))
		})
	}

	// Broadcast notification sender (to all sessions)
	const broadcast = createEmitter((method, params) => {
		const message = Rpc.notification(method, params)
		const data = `event: message\ndata: ${Rpc.stringify(message)}\n\n`
		const encoded = encoder.encode(data)
		for (const session of sessions.values()) {
			session.controller?.enqueue(encoded)
		}
	})

	// Get session-specific notifier
	const getSessionNotifier = (sessionId: string): NotificationEmitter | undefined => {
		const session = sessions.get(sessionId)
		return session?.notify
	}

	// Default context factory
	const defaultContext = (
		_req: Request,
		notify: NotificationEmitter
	): HandlerContext<TToolCtx, TResourceCtx, TPromptCtx> => ({
		toolContext: { notify } as unknown as TToolCtx,
		resourceContext: { notify } as unknown as TResourceCtx,
		promptContext: { notify } as unknown as TPromptCtx,
	})

	const createContext = options.createContext ?? defaultContext

	// CORS headers
	const corsHeaders = cors
		? {
				"Access-Control-Allow-Origin": cors,
				"Access-Control-Allow-Methods": "GET, POST, OPTIONS",
				"Access-Control-Allow-Headers": "Content-Type",
			}
		: {}

	// Bun server instance
	let bunServer: ReturnType<typeof Bun.serve> | null = null

	const handleRequest = async (req: Request): Promise<Response> => {
		const url = new URL(req.url)
		const path = url.pathname

		// Handle CORS preflight
		if (req.method === "OPTIONS") {
			return new Response(null, {
				status: 204,
				headers: corsHeaders,
			})
		}

		// SSE endpoint - establish stream
		if (path === `${basePath}/sse` && req.method === "GET") {
			const sessionId = crypto.randomUUID()
			const sessionNotifier = createSessionNotifier(sessionId)

			const stream = new ReadableStream<Uint8Array>({
				start(controller) {
					sessions.set(sessionId, {
						id: sessionId,
						createdAt: Date.now(),
						controller,
						notify: sessionNotifier,
					})

					// Send session ID as first event
					controller.enqueue(
						encoder.encode(`event: session\ndata: ${JSON.stringify({ sessionId })}\n\n`)
					)
				},
				cancel() {
					sessions.delete(sessionId)
				},
			})

			return new Response(stream, {
				headers: {
					"Content-Type": "text/event-stream",
					"Cache-Control": "no-cache",
					Connection: "keep-alive",
					...corsHeaders,
				},
			})
		}

		// SSE message endpoint
		if (path === `${basePath}/sse` && req.method === "POST") {
			const sessionId = req.headers.get("X-Session-ID")
			if (!sessionId) {
				return new Response(JSON.stringify({ error: "Missing X-Session-ID header" }), {
					status: 400,
					headers: { "Content-Type": "application/json", ...corsHeaders },
				})
			}

			const session = sessions.get(sessionId)
			if (!session) {
				return new Response(JSON.stringify({ error: "Session not found" }), {
					status: 404,
					headers: { "Content-Type": "application/json", ...corsHeaders },
				})
			}

			try {
				const body = await req.text()
				const ctx = createContext(req, session.notify)
				const notificationCtx: NotificationContext = {
					subscriberId: sessionId,
				}
				const response = await server.handle(body, ctx, notificationCtx)

				if (response && session.controller) {
					session.controller.enqueue(encoder.encode(`event: message\ndata: ${response}\n\n`))
				}

				return new Response(JSON.stringify({ ok: true }), {
					headers: { "Content-Type": "application/json", ...corsHeaders },
				})
			} catch (error) {
				options.onError?.(error instanceof Error ? error : new Error(String(error)))
				return new Response(JSON.stringify({ error: "Internal error" }), {
					status: 500,
					headers: { "Content-Type": "application/json", ...corsHeaders },
				})
			}
		}

		// Standard JSON-RPC endpoint (no notification support - use SSE for notifications)
		if (path === basePath && req.method === "POST") {
			try {
				const body = await req.text()
				// Use broadcast emitter for non-session requests
				const ctx = createContext(req, broadcast)
				const notificationCtx: NotificationContext = {
					subscriberId: "http-post",
				}
				const response = await server.handle(body, ctx, notificationCtx)

				return new Response(response ?? "", {
					status: response ? 200 : 204,
					headers: {
						"Content-Type": "application/json",
						...corsHeaders,
					},
				})
			} catch (error) {
				options.onError?.(error instanceof Error ? error : new Error(String(error)))
				const errorResponse = Rpc.error(null, Rpc.ErrorCode.InternalError, "Internal server error")
				return new Response(Rpc.stringify(errorResponse), {
					status: 500,
					headers: { "Content-Type": "application/json", ...corsHeaders },
				})
			}
		}

		// Health check
		if (path === `${basePath}/health` && req.method === "GET") {
			return new Response(
				JSON.stringify({
					status: "ok",
					server: server.name,
					version: server.version,
				}),
				{ headers: { "Content-Type": "application/json", ...corsHeaders } }
			)
		}

		// Not found
		return new Response("Not Found", { status: 404, headers: corsHeaders })
	}

	const start = async (): Promise<void> => {
		bunServer = Bun.serve({
			port,
			hostname,
			fetch: handleRequest,
		})
	}

	const stop = async (): Promise<void> => {
		// Close all SSE sessions
		for (const session of sessions.values()) {
			session.controller?.close()
		}
		sessions.clear()

		bunServer?.stop()
		bunServer = null
	}

	return {
		start,
		stop,
		broadcast,
		getSessionNotifier,
		get url() {
			return `http://${hostname}:${port}${basePath}`
		},
	}
}
