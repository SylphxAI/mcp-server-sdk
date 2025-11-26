/**
 * Transport Types
 *
 * Transports handle the communication layer (stdio, http, etc.)
 */

import type { NotificationEmitter } from '../notifications/index.js'

// ============================================================================
// Context for Handlers
// ============================================================================

export interface HandlerContext {
	readonly signal?: AbortSignal
}

// ============================================================================
// Server Interface (what transport receives)
// ============================================================================

export interface ServerHandler {
	readonly name: string
	readonly version: string
	readonly handle: (message: string) => Promise<string | null>
}

// ============================================================================
// Transport Interface
// ============================================================================

export interface Transport {
	readonly start: () => Promise<void>
	readonly stop: () => Promise<void>
}

export type TransportFactory = (server: ServerHandler, notify: NotificationEmitter) => Transport
