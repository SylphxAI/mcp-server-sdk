/**
 * HTTP Transport
 *
 * MCP Streamable HTTP Transport using @sylphx/gust
 *
 * Implements the MCP 2025-03-26 Streamable HTTP specification:
 * - Single POST endpoint for JSON-RPC messages
 * - Supports both JSON and SSE response formats
 * - SSE enables server-to-client notifications during request processing
 * - Bidirectional RPC: server can send requests to clients (sampling/elicitation)
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
	sse,
	textStream,
} from "@sylphx/gust"
import * as Rpc from "../protocol/jsonrpc.js"
import type { Transport, TransportFactory } from "./types.js"

// ============================================================================
// Pending Request Types
// ============================================================================

interface PendingRequest {
	resolve: (result: unknown) => void
	reject: (error: Error) => void
	timer: ReturnType<typeof setTimeout>
}

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
 * Bidirectional RPC (sampling/elicitation):
 * - Server sends requests via SSE events
 * - Client responds via POST to same endpoint with session ID
 * - Server resolves pending request and continues processing
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

		// Session storage for Mcp-Session-Id and pending requests
		const sessions = new Map<
			string,
			{
				createdAt: number
				pendingRequests: Map<string | number, PendingRequest>
			}
		>()
		let nextRequestId = 1

		// Helper to check if a message is a JSON-RPC response (not a request)
		const isJsonRpcResponse = (
			msg: unknown
		): msg is { jsonrpc: string; id: string | number; result?: unknown; error?: unknown } => {
			if (typeof msg !== "object" || msg === null) return false
			const obj = msg as Record<string, unknown>
			return "jsonrpc" in obj && "id" in obj && !("method" in obj)
		}

		// Routes
		const jsonRpcRoute = post(basePath, async (ctx: Context) => {
			try {
				const body = ctx.body.toString()
				const accept = ctx.headers.accept ?? ""
				const acceptsSSE = accept.includes("text/event-stream")

				// Check for session ID
				const sessionId = ctx.headers["mcp-session-id"]
				const session = sessionId ? sessions.get(sessionId) : undefined

				// Handle responses to pending server requests (bidirectional RPC)
				try {
					const parsed = JSON.parse(body)
					if (isJsonRpcResponse(parsed) && session) {
						const pending = session.pendingRequests.get(parsed.id)
						if (pending) {
							session.pendingRequests.delete(parsed.id)
							clearTimeout(pending.timer)

							if ("error" in parsed && parsed.error) {
								const errObj = parsed.error as { message?: string }
								pending.reject(new Error(errObj.message ?? "Request failed"))
							} else {
								pending.resolve(parsed.result)
							}

							// Acknowledge the response
							return response("", { status: 202 })
						}
					}
				} catch {
					// Not JSON or not a response, continue with normal handling
				}

				// Check for unknown session
				if (sessionId && !session) {
					return json({ error: "Session not found" }, { status: 404 })
				}

				if (acceptsSSE) {
					// Check if this is an initialize request and create session
					let activeSessionId = sessionId
					let activeSession = session
					const parsedReq = Rpc.parseMessage(body)
					if (
						parsedReq.ok &&
						Rpc.isRequest(parsedReq.value) &&
						parsedReq.value.method === "initialize"
					) {
						activeSessionId = crypto.randomUUID()
						activeSession = {
							createdAt: Date.now(),
							pendingRequests: new Map(),
						}
						sessions.set(activeSessionId, activeSession)
					}

					// Queue for events to be streamed
					const eventQueue: string[] = []
					let eventResolve: (() => void) | null = null
					let handlerComplete = false
					let handlerResult: string | null = null

					// Notification function - adds to queue and signals generator
					const notify = (method: string, params?: unknown) => {
						const notification = Rpc.notification(method, params)
						eventQueue.push(sseEvent(Rpc.stringify(notification), "message"))
						eventResolve?.()
					}

					// Request function for bidirectional RPC (sampling/elicitation)
					const request = async (method: string, params?: unknown): Promise<unknown> => {
						if (!activeSession) throw new Error("No session available")

						const id = `server-${nextRequestId++}`
						const req = Rpc.request(id, method, params)

						// Create promise that will be resolved when client POSTs response
						const promise = new Promise<unknown>((resolve, reject) => {
							const timer = setTimeout(() => {
								if (activeSession?.pendingRequests.has(id)) {
									activeSession.pendingRequests.delete(id)
									reject(new Error("Request timed out"))
								}
							}, 30000)

							activeSession.pendingRequests.set(id, { resolve, reject, timer })
						})

						// Send request via SSE
						eventQueue.push(sseEvent(Rpc.stringify(req), "message"))
						eventResolve?.()

						return promise
					}

					// Start handler execution in background
					const handlerPromise = server.handle(body, { notify, request }).then((result) => {
						handlerResult = result
						handlerComplete = true
						eventResolve?.()
					})

					// Create async generator for SSE stream
					async function* generateSSE(): AsyncGenerator<string> {
						// Yield session ID header event if new session
						if (activeSessionId && !sessionId) {
							// Note: Headers are set separately, this is just for the stream
						}

						// Yield events until handler completes
						while (!handlerComplete || eventQueue.length > 0) {
							// Yield any queued events
							while (eventQueue.length > 0) {
								const event = eventQueue.shift()
								if (event) yield event
							}

							// If handler not complete, wait for more events
							if (!handlerComplete) {
								await new Promise<void>((resolve) => {
									eventResolve = resolve
									// Also check periodically in case we missed a signal
									setTimeout(resolve, 100)
								})
								eventResolve = null
							}
						}

						// Yield final response
						if (handlerResult) {
							yield sseEvent(handlerResult, "message")
						}

						// Ensure handler promise is settled
						await handlerPromise
					}

					// Build headers
					const headers: Record<string, string> = {}
					if (activeSessionId && !sessionId) {
						headers["Mcp-Session-Id"] = activeSessionId
					}

					return sse(textStream(generateSSE()), { headers })
				}

				// Regular JSON response (no streaming, no bidirectional support)
				const responseStr = await server.handle(body)

				if (!responseStr) {
					return response("", { status: 204 })
				}

				// Generate session ID on initialize
				const parsed = Rpc.parseMessage(body)
				const headers: Record<string, string> = {}

				if (parsed.ok && Rpc.isRequest(parsed.value) && parsed.value.method === "initialize") {
					const newSessionId = crypto.randomUUID()
					sessions.set(newSessionId, {
						createdAt: Date.now(),
						pendingRequests: new Map(),
					})
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
