/**
 * Notification Helpers
 *
 * Pure functions for creating notifications.
 */

import type * as Mcp from "../protocol/mcp.js"
import type { Notification } from "./types.js"

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
 * Create a log notification.
 */
export const log = (level: Mcp.LogLevel, data: unknown, logger?: string): Notification => ({
	type: "log",
	level,
	logger,
	data,
})

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

/**
 * Emit a cancelled notification.
 */
export const cancelled = (requestId: string | number, reason?: string): Notification => ({
	type: "cancelled",
	requestId,
	reason,
})
