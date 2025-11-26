/**
 * Notification Helpers
 *
 * Pure functions for creating and emitting notifications.
 */

import type * as Mcp from "../protocol/mcp.js"
import type { Notification, NotificationEmitter } from "./types.js"

// ============================================================================
// Progress Helpers
// ============================================================================

/**
 * Create a progress notification.
 */
export const progress = (
	progressToken: Mcp.ProgressToken,
	current: number,
	options?: { total?: number; message?: string }
): Notification => ({
	type: "progress",
	progressToken,
	progress: current,
	total: options?.total,
	message: options?.message,
})

/**
 * Create a progress reporter bound to a token.
 *
 * @example
 * ```ts
 * const report = createProgressReporter(emitter, "token-123", 100)
 * report(25, "Processing...")
 * report(50, "Halfway done...")
 * report(100, "Complete!")
 * ```
 */
export const createProgressReporter = (
	emitter: NotificationEmitter,
	token: Mcp.ProgressToken,
	total?: number
) => {
	return (current: number, message?: string): void => {
		emitter.emit(progress(token, current, { total, message }))
	}
}

/**
 * Create an async progress tracker that reports progress automatically.
 *
 * @example
 * ```ts
 * const results = await withProgress(emitter, "token-123", items, async (item, report) => {
 *   report(`Processing ${item.name}`)
 *   return await process(item)
 * })
 * ```
 */
export const withProgress = async <T, R>(
	emitter: NotificationEmitter,
	token: Mcp.ProgressToken,
	items: readonly T[],
	processor: (item: T, report: (message?: string) => void) => Promise<R>
): Promise<R[]> => {
	const total = items.length
	const results: R[] = []

	let current = 0
	for (const item of items) {
		current++
		const idx = current
		const report = (message?: string) => {
			emitter.emit(progress(token, idx, { total, message }))
		}
		results.push(await processor(item, report))
	}

	return results
}

// ============================================================================
// Logging Helpers
// ============================================================================

/**
 * Create a log notification.
 */
export const log = (level: Mcp.LogLevel, data: unknown, logger?: string): Notification => ({
	type: "log",
	level,
	logger,
	data,
})

/**
 * Create a logger bound to a namespace.
 *
 * @example
 * ```ts
 * const logger = createLogger(emitter, "my-tool")
 * logger.info("Processing started")
 * logger.error({ message: "Failed", code: 500 })
 * ```
 */
export interface Logger {
	readonly debug: (data: unknown) => void
	readonly info: (data: unknown) => void
	readonly notice: (data: unknown) => void
	readonly warning: (data: unknown) => void
	readonly error: (data: unknown) => void
	readonly critical: (data: unknown) => void
	readonly alert: (data: unknown) => void
	readonly emergency: (data: unknown) => void
}

export const createLogger = (emitter: NotificationEmitter, namespace?: string): Logger => {
	const emit = (level: Mcp.LogLevel) => (data: unknown) => {
		emitter.emit(log(level, data, namespace))
	}

	return {
		debug: emit("debug"),
		info: emit("info"),
		notice: emit("notice"),
		warning: emit("warning"),
		error: emit("error"),
		critical: emit("critical"),
		alert: emit("alert"),
		emergency: emit("emergency"),
	}
}

// ============================================================================
// List Change Helpers
// ============================================================================

/**
 * Emit a resources list changed notification.
 */
export const resourcesListChanged = (): Notification => ({
	type: "resources/list_changed",
})

/**
 * Emit a tools list changed notification.
 */
export const toolsListChanged = (): Notification => ({
	type: "tools/list_changed",
})

/**
 * Emit a prompts list changed notification.
 */
export const promptsListChanged = (): Notification => ({
	type: "prompts/list_changed",
})

/**
 * Emit a resource updated notification.
 */
export const resourceUpdated = (uri: string): Notification => ({
	type: "resource/updated",
	uri,
})

// ============================================================================
// Cancellation Helper
// ============================================================================

/**
 * Emit a cancelled notification.
 */
export const cancelled = (requestId: string | number, reason?: string): Notification => ({
	type: "cancelled",
	requestId,
	reason,
})
