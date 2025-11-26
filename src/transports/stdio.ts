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

		const start = async (): Promise<void> => {
			if (running) return
			running = true

			const stdin = options.stdin ?? Bun.stdin.stream()
			const stdout = options.stdout ?? Bun.stdout.writer()
			writer = stdout

			const decoder = new TextDecoder()
			let buffer = ""

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
							const response = await server.handle(line)

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

		const stop = async (): Promise<void> => {
			running = false
		}

		return { start, stop }
	}
}
