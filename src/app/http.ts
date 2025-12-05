/**
 * HTTP Fetch Adapter
 *
 * Fetch API handler for MCP Streamable HTTP transport.
 * Compatible with Bun.serve, Deno.serve, Cloudflare Workers.
 */

import * as Rpc from "../protocol/jsonrpc.js"
import { dispatch, type HandlerContext, type ServerState } from "../server/handler.js"

// ============================================================================
// Types
// ============================================================================

interface PendingRequest {
	resolve: (result: unknown) => void
	reject: (error: Error) => void
	timer: ReturnType<typeof setTimeout>
}

interface Session {
	createdAt: number
	pendingRequests: Map<string | number, PendingRequest>
}

// ============================================================================
// Helpers
// ============================================================================

const isJsonRpcResponse = (
	msg: unknown,
): msg is { jsonrpc: string; id: string | number; result?: unknown; error?: unknown } => {
	if (typeof msg !== "object" || msg === null) return false
	const obj = msg as Record<string, unknown>
	return "jsonrpc" in obj && "id" in obj && !("method" in obj)
}

// ============================================================================
// Fetch Handler Factory
// ============================================================================

/**
 * Create a Fetch API handler for MCP.
 *
 * Implements MCP Streamable HTTP (2025-03-26):
 * - POST / - JSON-RPC with optional SSE streaming
 * - Handles session management and bidirectional RPC
 *
 * @internal
 */
export const createFetchHandler = (
	state: ServerState,
): ((request: Request) => Promise<Response>) => {
	// Session storage
	const sessions = new Map<string, Session>()
	let nextRequestId = 1

	return async (request: Request): Promise<Response> => {
		const url = new URL(request.url)

		// Health check endpoint
		if (request.method === "GET" && url.pathname.endsWith("/health")) {
			return Response.json({
				status: "ok",
				server: state.name,
				version: state.version,
			})
		}

		// Only handle POST for JSON-RPC
		if (request.method !== "POST") {
			return new Response("Method not allowed", { status: 405 })
		}

		try {
			const body = await request.text()
			const accept = request.headers.get("accept") ?? ""
			const acceptsSSE = accept.includes("text/event-stream")
			const sessionId = request.headers.get("mcp-session-id")
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

						return new Response("", { status: 202 })
					}
				}
			} catch {
				// Not JSON or not a response, continue
			}

			// Check for unknown session
			if (sessionId && !session) {
				return Response.json({ error: "Session not found" }, { status: 404 })
			}

			// Parse incoming message
			const parseResult = Rpc.parseMessage(body)
			if (!parseResult.ok) {
				const errorResponse = Rpc.error(null, Rpc.ErrorCode.ParseError, parseResult.error)
				return Response.json(JSON.parse(Rpc.stringify(errorResponse)), { status: 400 })
			}

			const message = parseResult.value

			if (acceptsSSE) {
				return handleSSERequest(state, message, sessions, sessionId, session, nextRequestId++)
			}

			// Regular JSON response
			return handleJsonRequest(state, message, sessions, sessionId)
		} catch (error) {
			console.error("HTTP Fetch Error:", error)
			const errorResponse = Rpc.error(null, Rpc.ErrorCode.InternalError, "Internal server error")
			return Response.json(JSON.parse(Rpc.stringify(errorResponse)), { status: 500 })
		}
	}
}

// ============================================================================
// JSON Request Handler
// ============================================================================

const handleJsonRequest = async (
	state: ServerState,
	message: Rpc.JsonRpcMessage,
	sessions: Map<string, Session>,
	sessionId: string | null,
): Promise<Response> => {
	const result = await dispatch(state, message, {})
	const headers = new Headers({ "Content-Type": "application/json" })

	// Generate session ID on initialize
	if (Rpc.isRequest(message) && message.method === "initialize") {
		const newSessionId = crypto.randomUUID()
		sessions.set(newSessionId, {
			createdAt: Date.now(),
			pendingRequests: new Map(),
		})
		headers.set("Mcp-Session-Id", newSessionId)
	}

	if (result.type === "none") {
		return new Response("", { status: 204 })
	}

	return new Response(JSON.stringify(result.response), { headers })
}

// ============================================================================
// SSE Request Handler
// ============================================================================

const handleSSERequest = async (
	state: ServerState,
	message: Rpc.JsonRpcMessage,
	sessions: Map<string, Session>,
	sessionId: string | null,
	existingSession: Session | undefined,
	requestIdBase: number,
): Promise<Response> => {
	// Create session on initialize
	let activeSessionId = sessionId
	let activeSession = existingSession

	if (Rpc.isRequest(message) && message.method === "initialize") {
		activeSessionId = crypto.randomUUID()
		activeSession = {
			createdAt: Date.now(),
			pendingRequests: new Map(),
		}
		sessions.set(activeSessionId, activeSession)
	}

	// SSE stream state
	const eventQueue: string[] = []
	let eventResolve: (() => void) | null = null
	let handlerComplete = false
	let handlerResult: Rpc.JsonRpcResponse | null = null
	let nextReqId = requestIdBase

	// Notification function
	const notify = (method: string, params?: unknown) => {
		const notification = Rpc.notification(method, params)
		eventQueue.push(`event: message\ndata: ${Rpc.stringify(notification)}\n\n`)
		eventResolve?.()
	}

	// Request function for bidirectional RPC
	const request = async (method: string, params?: unknown): Promise<unknown> => {
		if (!activeSession) throw new Error("No session available")

		const id = `server-${nextReqId++}`
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

		eventQueue.push(`event: message\ndata: ${Rpc.stringify(req)}\n\n`)
		eventResolve?.()

		return promise
	}

	// Create handler context
	const ctx: HandlerContext = { notify, request }

	// Start handler execution in background
	dispatch(state, message, ctx)
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

	// Create ReadableStream for SSE
	const stream = new ReadableStream({
		async pull(controller) {
			while (!handlerComplete || eventQueue.length > 0) {
				// Yield queued events
				while (eventQueue.length > 0) {
					const event = eventQueue.shift()
					if (event) {
						controller.enqueue(new TextEncoder().encode(event))
					}
				}

				// Wait for more events
				if (!handlerComplete) {
					await new Promise<void>((resolve) => {
						eventResolve = resolve
						setTimeout(resolve, 100)
					})
					eventResolve = null
				}
			}

			// Yield final response
			if (handlerResult) {
				const finalEvent = `event: message\ndata: ${JSON.stringify(handlerResult)}\n\n`
				controller.enqueue(new TextEncoder().encode(finalEvent))
			}

			controller.close()
		},
	})

	// Build response headers
	const headers = new Headers({
		"Content-Type": "text/event-stream",
		"Cache-Control": "no-cache",
		Connection: "keep-alive",
	})

	if (activeSessionId && !sessionId) {
		headers.set("Mcp-Session-Id", activeSessionId)
	}

	return new Response(stream, { headers })
}
