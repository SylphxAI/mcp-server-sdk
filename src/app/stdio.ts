/**
 * Stdio Runner
 *
 * Run an MCP app over stdio (stdin/stdout).
 * For CLI tools and local MCP servers.
 */

import { Readable } from "node:stream"
import * as Rpc from "../protocol/jsonrpc.js"
import { dispatch, type HandlerContext } from "../server/handler.js"
import type { McpApp } from "./app.js"

// ============================================================================
// Types
// ============================================================================

export interface StdioOptions {
	/** MCP application */
	readonly app: McpApp
	/** Custom stdin (for testing) */
	readonly stdin?: ReadableStream<Uint8Array>
	/** Custom stdout (for testing) */
	readonly stdout?: { write: (data: Uint8Array) => void; flush?: () => void }
	/** Error handler */
	readonly onError?: (error: Error) => void
}

export interface StdioRunner {
	/** Stop the stdio runner */
	readonly stop: () => void
}

// ============================================================================
// Stdio Runner
// ============================================================================

/**
 * Run an MCP app over stdio.
 *
 * Implements bidirectional JSON-RPC over stdin/stdout:
 * - Reads JSON-RPC messages from stdin (newline-delimited)
 * - Writes responses to stdout
 * - Supports server→client requests (sampling, elicitation)
 *
 * @example
 * ```ts
 * const app = createMcpApp({ tools: { greet } })
 * const runner = await runStdio({ app })
 *
 * // Later, to stop:
 * runner.stop()
 * ```
 */
export const runStdio = async (options: StdioOptions): Promise<StdioRunner> => {
	const { app } = options
	const state = app.state

	let running = true
	const encoder = new TextEncoder()

	// Pending requests waiting for responses (for sampling/elicitation)
	const pendingRequests = new Map<
		string | number,
		{
			resolve: (result: unknown) => void
			reject: (error: Error) => void
			timer: ReturnType<typeof setTimeout>
		}
	>()
	let nextRequestId = 1

	// Setup stdin/stdout
	const stdin = options.stdin ?? (Readable.toWeb(process.stdin) as ReadableStream<Uint8Array>)
	const stdout = options.stdout ?? {
		write: (data: Uint8Array) => process.stdout.write(data),
		flush: () => {},
	}

	const decoder = new TextDecoder()
	let buffer = ""

	const reader = stdin.getReader()

	// Write a message to stdout
	const writeMessage = (message: string) => {
		stdout.write(encoder.encode(`${message}\n`))
		stdout.flush?.()
	}

	// Send a request to the client and wait for response
	const request = async (method: string, params?: unknown): Promise<unknown> => {
		const id = `server-${nextRequestId++}`
		const req = Rpc.request(id, method, params)

		const promise = new Promise<unknown>((resolve, reject) => {
			const timer = setTimeout(() => {
				if (pendingRequests.has(id)) {
					pendingRequests.delete(id)
					reject(new Error("Request timed out"))
				}
			}, 30000)

			pendingRequests.set(id, { resolve, reject, timer })
		})

		// Send request to client
		writeMessage(Rpc.stringify(req))

		return promise
	}

	// Send a notification to the client
	const notify = (method: string, params?: unknown) => {
		const notification = Rpc.notification(method, params)
		writeMessage(Rpc.stringify(notification))
	}

	// Check if a message is a JSON-RPC response
	const isJsonRpcResponse = (
		msg: unknown,
	): msg is { jsonrpc: string; id: string | number; result?: unknown; error?: unknown } => {
		if (typeof msg !== "object" || msg === null) return false
		const obj = msg as Record<string, unknown>
		return "jsonrpc" in obj && "id" in obj && !("method" in obj)
	}

	// Process a single line
	const processLine = async (line: string) => {
		// Check if this is a response to a pending request
		try {
			const parsed = JSON.parse(line)
			if (isJsonRpcResponse(parsed)) {
				const pending = pendingRequests.get(parsed.id)
				if (pending) {
					pendingRequests.delete(parsed.id)
					clearTimeout(pending.timer)

					if ("error" in parsed && parsed.error) {
						const errObj = parsed.error as { message?: string }
						pending.reject(new Error(errObj.message ?? "Request failed"))
					} else {
						pending.resolve(parsed.result)
					}
					return
				}
			}
		} catch {
			// Not valid JSON, treat as regular message
		}

		// Parse as JSON-RPC message
		const parseResult = Rpc.parseMessage(line)
		if (!parseResult.ok) {
			const errorResponse = Rpc.error(null, Rpc.ErrorCode.ParseError, parseResult.error)
			writeMessage(Rpc.stringify(errorResponse))
			return
		}

		// Create handler context with bidirectional RPC support
		const ctx: HandlerContext = { notify, request }

		// Dispatch to handler
		try {
			const result = await dispatch(state, parseResult.value, ctx)

			if (result.type === "response") {
				writeMessage(JSON.stringify(result.response))
			}
			// For "none" type, don't write anything (notification responses)
		} catch (error) {
			options.onError?.(error instanceof Error ? error : new Error(String(error)))

			const id = Rpc.isRequest(parseResult.value) ? parseResult.value.id : null
			const errorResponse = Rpc.error(
				id,
				Rpc.ErrorCode.InternalError,
				error instanceof Error ? error.message : "Internal error",
			)
			writeMessage(Rpc.stringify(errorResponse))
		}
	}

	// Main read loop
	const readLoop = async () => {
		try {
			while (running) {
				const { done, value } = await reader.read()

				if (done) break

				buffer += decoder.decode(value, { stream: true })

				// Process complete lines
				for (
					let newlineIndex = buffer.indexOf("\n");
					newlineIndex !== -1;
					newlineIndex = buffer.indexOf("\n")
				) {
					const line = buffer.slice(0, newlineIndex).trim()
					buffer = buffer.slice(newlineIndex + 1)

					if (line.length === 0) continue

					await processLine(line)
				}
			}
		} catch (error) {
			if (running) {
				options.onError?.(error instanceof Error ? error : new Error(String(error)))
			}
		} finally {
			reader.releaseLock()
			running = false
			// Clear pending requests
			for (const pending of pendingRequests.values()) {
				clearTimeout(pending.timer)
				pending.reject(new Error("Transport closed"))
			}
			pendingRequests.clear()
		}
	}

	// Start reading
	readLoop()

	return {
		stop: () => {
			running = false
		},
	}
}
