/**
 * Serve Function
 *
 * Convenience function to start an MCP server using @sylphx/gust.
 */

import {
	type Context,
	cors,
	createApp,
	get,
	serve as gustServe,
	json,
	post,
	response,
	type SSEEvent,
	sse,
} from "@sylphx/gust"
import * as Rpc from "../protocol/jsonrpc.js"
import { dispatch, type HandlerContext } from "../server/handler.js"
import type { McpApp } from "./app.js"

// ============================================================================
// Types
// ============================================================================

export interface ServeOptions {
	/** MCP application */
	readonly app: McpApp
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
	/** Called when server starts listening */
	readonly onListen?: (info: { port: number; hostname: string }) => void
}

export interface McpServer {
	readonly port: number
	readonly hostname: string
	readonly stop: () => Promise<void>
}

// ============================================================================
// Pending Request Types
// ============================================================================

interface PendingRequest {
	resolve: (result: unknown) => void
	reject: (error: Error) => void
	timer: ReturnType<typeof setTimeout>
}

// ============================================================================
// Serve Function
// ============================================================================

/**
 * Start an MCP server using @sylphx/gust.
 *
 * @example
 * ```ts
 * const app = createMcpApp({ tools: { ... } })
 * const server = await serve({ app, port: 3000 })
 *
 * // Later...
 * await server.stop()
 * ```
 */
export const serve = async (options: ServeOptions): Promise<McpServer> => {
	const { app } = options
	const state = app.state
	const port = options.port ?? 3000
	const hostname = options.hostname ?? "localhost"
	const basePath = options.basePath ?? "/mcp"

	// Session storage
	const sessions = new Map<
		string,
		{
			createdAt: number
			pendingRequests: Map<string | number, PendingRequest>
		}
	>()
	let nextRequestId = 1

	// Helper to check if a message is a JSON-RPC response
	const isJsonRpcResponse = (
		msg: unknown,
	): msg is { jsonrpc: string; id: string | number; result?: unknown; error?: unknown } => {
		if (typeof msg !== "object" || msg === null) return false
		const obj = msg as Record<string, unknown>
		return "jsonrpc" in obj && "id" in obj && !("method" in obj)
	}

	// Main JSON-RPC route
	const jsonRpcRoute = post(basePath, async ({ ctx }: { ctx: Context }) => {
		try {
			const body = ctx.body.toString()
			const accept = ctx.headers["accept"] ?? ""
			const acceptsSSE = accept.includes("text/event-stream")
			const sessionId = ctx.headers["mcp-session-id"]
			const session = sessionId ? sessions.get(sessionId) : undefined

			// Handle responses to pending server requests
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

						return response("", { status: 202 })
					}
				}
			} catch {
				// Not JSON or not a response
			}

			// Check for unknown session
			if (sessionId && !session) {
				return json({ error: "Session not found" }, { status: 404 })
			}

			// Parse message
			const parseResult = Rpc.parseMessage(body)
			if (!parseResult.ok) {
				const errorResponse = Rpc.error(null, Rpc.ErrorCode.ParseError, parseResult.error)
				return json(JSON.parse(Rpc.stringify(errorResponse)), { status: 400 })
			}

			const message = parseResult.value

			if (acceptsSSE) {
				// SSE streaming response
				let activeSessionId = sessionId
				let activeSession = session

				if (Rpc.isRequest(message) && message.method === "initialize") {
					activeSessionId = crypto.randomUUID()
					activeSession = {
						createdAt: Date.now(),
						pendingRequests: new Map(),
					}
					sessions.set(activeSessionId, activeSession)
				}

				const eventQueue: SSEEvent[] = []
				let eventResolve: (() => void) | null = null
				let handlerComplete = false
				let handlerResult: Rpc.JsonRpcResponse | null = null

				const notify = (method: string, params?: unknown) => {
					const notification = Rpc.notification(method, params)
					eventQueue.push({ data: Rpc.stringify(notification), event: "message" })
					eventResolve?.()
				}

				const request = async (method: string, params?: unknown): Promise<unknown> => {
					if (!activeSession) throw new Error("No session available")

					const id = `server-${nextRequestId++}`
					const req = Rpc.request(id, method, params)

					const promise = new Promise<unknown>((resolve, reject) => {
						const timer = setTimeout(() => {
							if (activeSession?.pendingRequests.has(id)) {
								activeSession.pendingRequests.delete(id)
								reject(new Error("Request timed out"))
							}
						}, 30000)

						activeSession.pendingRequests.set(id, { resolve, reject, timer })
					})

					eventQueue.push({ data: Rpc.stringify(req), event: "message" })
					eventResolve?.()

					return promise
				}

				const handlerCtx: HandlerContext = { notify, request }

				dispatch(state, message, handlerCtx)
					.then((result) => {
						if (result.type === "response") {
							handlerResult = result.response
						}
						handlerComplete = true
						eventResolve?.()
					})
					.catch((error) => {
						handlerResult = Rpc.error(
							Rpc.isRequest(message) ? message.id : null,
							Rpc.ErrorCode.InternalError,
							error instanceof Error ? error.message : String(error),
						)
						handlerComplete = true
						eventResolve?.()
					})

				const generateSSE = async function* (): AsyncGenerator<SSEEvent> {
					while (!handlerComplete || eventQueue.length > 0) {
						while (eventQueue.length > 0) {
							const event = eventQueue.shift()
							if (event) yield event
						}

						if (!handlerComplete) {
							await new Promise<void>((resolve) => {
								eventResolve = resolve
								setTimeout(resolve, 100)
							})
							eventResolve = null
						}
					}

					if (handlerResult) {
						yield { data: JSON.stringify(handlerResult), event: "message" }
					}
				}

				const headers: Record<string, string> = {}
				if (activeSessionId && !sessionId) {
					headers["Mcp-Session-Id"] = activeSessionId
				}

				return sse(generateSSE, { headers })
			}

			// Regular JSON response
			const result = await dispatch(state, message, {})
			const headers: Record<string, string> = {}

			if (Rpc.isRequest(message) && message.method === "initialize") {
				const newSessionId = crypto.randomUUID()
				sessions.set(newSessionId, {
					createdAt: Date.now(),
					pendingRequests: new Map(),
				})
				headers["Mcp-Session-Id"] = newSessionId
			}

			if (result.type === "none") {
				return response("", { status: 204 })
			}

			return json(result.response, { headers })
		} catch (error) {
			console.error("MCP Server Error:", error)
			options.onError?.(error instanceof Error ? error : new Error(String(error)))
			const errorResponse = Rpc.error(null, Rpc.ErrorCode.InternalError, "Internal server error")
			return json(JSON.parse(Rpc.stringify(errorResponse)), { status: 500 })
		}
	})

	// Health check route
	const healthRoute = get(`${basePath}/health`, () =>
		json({
			status: "ok",
			server: state.name,
			version: state.version,
		}),
	)

	// Build gust app
	const routes = [jsonRpcRoute, healthRoute]
	const middleware = options.cors
		? cors({
				origin: options.cors,
				allowedHeaders: ["Content-Type", "Accept", "Mcp-Session-Id"],
				exposedHeaders: ["Mcp-Session-Id"],
			})
		: undefined

	const gustApp = createApp({ routes, middleware })

	// Start server
	const server = await gustServe({
		app: gustApp,
		port,
		hostname,
		onListen: options.onListen
			? (info) => options.onListen?.({ port: info.port, hostname: info.hostname })
			: undefined,
	})

	return {
		port: server.port,
		hostname: server.hostname,
		stop: async () => {
			sessions.clear()
			await server.stop()
		},
	}
}
