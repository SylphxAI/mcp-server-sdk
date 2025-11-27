import { Readable } from "node:stream"
import * as Rpc from "../protocol/jsonrpc.js"
import type { Transport, TransportFactory } from "./types.js"

// ============================================================================
// Options
// ============================================================================

export interface StdioOptions {
	/** Custom stdin (for testing) */
	readonly stdin?: ReadableStream<Uint8Array>
	/** Custom stdout (for testing) */
	readonly stdout?: { write: (data: Uint8Array) => void; flush: () => void }
	/** Error handler */
	readonly onError?: (error: Error) => void
}

// ============================================================================
// Stdio Transport Factory
// ============================================================================

/**
 * Create a stdio transport.
 *
 * Supports bidirectional JSON-RPC for sampling and elicitation.
 *
 * @example
 * ```ts
 * createServer({
 *   tools: { ping },
 *   transport: stdio()
 * })
 * ```
 */
export const stdio = (options: StdioOptions = {}): TransportFactory => {
	return (server, _notify): Transport => {
		let running = false
		let writer: { write: (data: Uint8Array) => void; flush: () => void } | null = null
		const encoder = new TextEncoder()

		// Pending requests waiting for responses (for sampling/elicitation)
		const pendingRequests = new Map<
			string | number,
			{ resolve: (result: unknown) => void; reject: (error: Error) => void }
		>()
		let nextRequestId = 1

		const start = async (): Promise<void> => {
			if (running) return
			running = true

			const stdin = options.stdin ?? (Readable.toWeb(process.stdin) as ReadableStream<Uint8Array>)
			const stdout = options.stdout ?? {
				write: (data: Uint8Array) => process.stdout.write(data),
				flush: () => {},
			}
			writer = stdout

			const decoder = new TextDecoder()
			let buffer = ""

			const reader = stdin.getReader()

			// Send a request to the client and wait for response
			const request = async (method: string, params?: unknown): Promise<unknown> => {
				if (!writer) throw new Error("Transport not started")

				const id = `server-${nextRequestId++}`
				const req = Rpc.request(id, method, params)

				// Create promise that will be resolved when response arrives
				const promise = new Promise<unknown>((resolve, reject) => {
					pendingRequests.set(id, { resolve, reject })

					// Timeout after 30 seconds
					setTimeout(() => {
						if (pendingRequests.has(id)) {
							pendingRequests.delete(id)
							reject(new Error("Request timed out"))
						}
					}, 30000)
				})

				// Send request to client
				writer.write(encoder.encode(`${Rpc.stringify(req)}\n`))
				writer.flush()

				return promise
			}

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

						// Check if this is a response to a pending request
						try {
							const parsed = JSON.parse(line)
							if (
								"id" in parsed &&
								parsed.id !== null &&
								typeof parsed.id === "string" &&
								parsed.id.startsWith("server-")
							) {
								const pending = pendingRequests.get(parsed.id)
								if (pending) {
									pendingRequests.delete(parsed.id)

									if ("error" in parsed) {
										pending.reject(new Error(parsed.error?.message || "Request failed"))
									} else {
										pending.resolve(parsed.result)
									}
									continue
								}
							}
						} catch {
							// Not valid JSON, treat as regular message
						}

						// Handle as regular client→server message
						try {
							const response = await server.handle(line, { request })

							if (response && writer) {
								writer.write(encoder.encode(`${response}\n`))
								writer.flush()
							}
						} catch (error) {
							options.onError?.(error instanceof Error ? error : new Error(String(error)))
						}
					}
				}
			} finally {
				reader.releaseLock()
				running = false
				writer = null
				pendingRequests.clear()
			}
		}

		const stop = async (): Promise<void> => {
			running = false
		}

		return { start, stop }
	}
}
