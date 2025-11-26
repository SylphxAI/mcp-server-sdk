/**
 * Notification Emitter
 *
 * Factory for creating notification emitters from raw senders.
 */

import * as Mcp from "../protocol/mcp.js"
import type { Notification, NotificationEmitter, NotificationSender } from "./types.js"

// ============================================================================
// Emitter Factory
// ============================================================================

/**
 * Create a notification emitter from a raw sender.
 *
 * @example
 * ```ts
 * const emitter = createEmitter((method, params) => {
 *   transport.send({ jsonrpc: "2.0", method, params })
 * })
 *
 * emitter.emit({ type: "progress", progressToken: "123", progress: 50 })
 * ```
 */
export const createEmitter = (send: NotificationSender): NotificationEmitter => ({
	emit: (notification) => {
		const [method, params] = toJsonRpc(notification)
		send(method, params)
	},
})

/**
 * No-op emitter for contexts that don't support notifications.
 */
export const noopEmitter: NotificationEmitter = {
	emit: () => {},
}

// ============================================================================
// Notification to JSON-RPC Conversion
// ============================================================================

const toJsonRpc = (notification: Notification): [method: string, params?: unknown] => {
	switch (notification.type) {
		case "progress":
			return [
				Mcp.Method.ProgressNotification,
				{
					progressToken: notification.progressToken,
					progress: notification.progress,
					total: notification.total,
					message: notification.message,
				},
			]

		case "log":
			return [
				Mcp.Method.LogMessage,
				{
					level: notification.level,
					logger: notification.logger,
					data: notification.data,
				},
			]

		case "resources/list_changed":
			return [Mcp.Method.ResourcesListChanged]

		case "tools/list_changed":
			return [Mcp.Method.ToolsListChanged]

		case "prompts/list_changed":
			return [Mcp.Method.PromptsListChanged]

		case "resource/updated":
			return [Mcp.Method.ResourcesUpdated, { uri: notification.uri }]

		case "cancelled":
			return [
				Mcp.Method.ProgressCancelled,
				{
					requestId: notification.requestId,
					reason: notification.reason,
				},
			]
	}
}
