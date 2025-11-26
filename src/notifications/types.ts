/**
 * Notification Types
 *
 * Types for server-to-client notifications.
 */

import type * as Mcp from "../protocol/mcp.js"

// ============================================================================
// Notification Types
// ============================================================================

export interface ProgressNotification {
	readonly type: "progress"
	readonly progressToken: Mcp.ProgressToken
	readonly progress: number
	readonly total?: number
	readonly message?: string
}

export interface LogNotification {
	readonly type: "log"
	readonly level: Mcp.LogLevel
	readonly logger?: string
	readonly data?: unknown
}

export interface ResourcesListChangedNotification {
	readonly type: "resources/list_changed"
}

export interface ToolsListChangedNotification {
	readonly type: "tools/list_changed"
}

export interface PromptsListChangedNotification {
	readonly type: "prompts/list_changed"
}

export interface ResourceUpdatedNotification {
	readonly type: "resource/updated"
	readonly uri: string
}

export interface CancelledNotification {
	readonly type: "cancelled"
	readonly requestId: string | number
	readonly reason?: string
}

export type Notification =
	| ProgressNotification
	| LogNotification
	| ResourcesListChangedNotification
	| ToolsListChangedNotification
	| PromptsListChangedNotification
	| ResourceUpdatedNotification
	| CancelledNotification

// ============================================================================
// Notification Emitter
// ============================================================================

/**
 * Notification emitter interface.
 * Transports implement this to send notifications to clients.
 */
export interface NotificationEmitter {
	readonly emit: (notification: Notification) => void
}

/**
 * Raw notification sender - sends JSON-RPC notification messages.
 * This is what transports provide.
 */
export type NotificationSender = (method: string, params?: unknown) => void

// ============================================================================
// Context Extensions
// ============================================================================

/**
 * Context extension for handlers that can emit notifications.
 */
export interface NotificationContext {
	readonly notify: NotificationEmitter
}
