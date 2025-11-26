/**
 * Stdio Transport
 *
 * Line-delimited JSON-RPC over stdin/stdout.
 * Uses Bun's native streams for optimal performance.
 */

import type { PromptContext } from "../builders/prompt.js"
import type { ResourceContext } from "../builders/resource.js"
import type { ToolContext } from "../builders/tool.js"
import { type NotificationEmitter, createEmitter } from "../notifications/index.js"
import * as Rpc from "../protocol/jsonrpc.js"
import type { HandlerContext, NotificationContext } from "../server/handler.js"
import type { Server } from "../server/server.js"

// ============================================================================
// Transport Types
// ============================================================================

export interface StdioTransport {
	/** Start processing stdin and writing to stdout */
	readonly start: () => Promise<void>
	/** Stop the transport */
	readonly stop: () => void
	/** Notification emitter for server-to-client messages */
	readonly notify: NotificationEmitter
}

export interface StdioOptions<
	TToolCtx extends ToolContext,
	TResourceCtx extends ResourceContext,
	TPromptCtx extends PromptContext,
> {
	/** Custom context factory (called for each message, receives notification emitter) */
	readonly createContext?: (
		notify: NotificationEmitter
	) => HandlerContext<TToolCtx, TResourceCtx, TPromptCtx>
	/** Custom stdin (for testing) */
	readonly stdin?: ReadableStream<Uint8Array>
	/** Custom stdout (for testing) */
	readonly stdout?: WritableStream<Uint8Array>
	/** Error handler */
	readonly onError?: (error: Error) => void
}

// ============================================================================
// Stdio Transport Factory
// ============================================================================

/**
 * Create a stdio transport for the server.
 *
 * @example
 * ```ts
 * const server = createServer({ ... })
 * const transport = stdio(server)
 * await transport.start()
 * ```
 */
export const stdio = <
	TToolCtx extends ToolContext,
	TResourceCtx extends ResourceContext,
	TPromptCtx extends PromptContext,
>(
	server: Server<TToolCtx, TResourceCtx, TPromptCtx>,
	options: StdioOptions<TToolCtx, TResourceCtx, TPromptCtx> = {}
): StdioTransport => {
	let running = false
	let abortController: AbortController | null = null
	let writer: { write: (data: Uint8Array) => void; flush: () => void } | null = null
	const encoder = new TextEncoder()

	// Create notification sender
	const sendNotification = (method: string, params?: unknown): void => {
		if (!writer) return
		const message = Rpc.notification(method, params)
		writer.write(encoder.encode(`${Rpc.stringify(message)}\n`))
		writer.flush()
	}

	const notify = createEmitter(sendNotification)

	const defaultContext = (
		emitter: NotificationEmitter
	): HandlerContext<TToolCtx, TResourceCtx, TPromptCtx> => ({
		toolContext: { signal: abortController?.signal, notify: emitter } as unknown as TToolCtx,
		resourceContext: {
			signal: abortController?.signal,
			notify: emitter,
		} as unknown as TResourceCtx,
		promptContext: { signal: abortController?.signal, notify: emitter } as unknown as TPromptCtx,
	})

	const createContext = options.createContext ?? defaultContext

	const start = async (): Promise<void> => {
		if (running) return
		running = true
		abortController = new AbortController()

		const stdin = options.stdin ?? Bun.stdin.stream()
		const stdout = options.stdout ?? Bun.stdout.writer()
		writer = stdout as { write: (data: Uint8Array) => void; flush: () => void }

		const decoder = new TextDecoder()
		let buffer = ""

		// Track in-flight requests for cancellation
		const inFlightRequests = new Map<string | number, AbortController>()

		// Notification context for handling cancellations
		const notificationCtx: NotificationContext = {
			subscriberId: "stdio",
			onCancelled: (requestId, _reason) => {
				const controller = inFlightRequests.get(requestId)
				controller?.abort()
				inFlightRequests.delete(requestId)
			},
		}

		const reader = stdin.getReader()

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

					try {
						const ctx = createContext(notify)
						const response = await server.handle(line, ctx, notificationCtx)

						if (response) {
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
		}
	}

	const stop = (): void => {
		running = false
		abortController?.abort()
		abortController = null
	}

	return { start, stop, notify }
}
