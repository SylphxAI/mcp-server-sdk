/**
 * Subscription Types
 *
 * Types for resource subscription management.
 */

// ============================================================================
// Subscription Manager
// ============================================================================

/**
 * Subscription manager for tracking resource subscriptions.
 */
export interface SubscriptionManager {
	/** Subscribe to a resource */
	readonly subscribe: (uri: string, subscriberId: string) => void
	/** Unsubscribe from a resource */
	readonly unsubscribe: (uri: string, subscriberId: string) => void
	/** Unsubscribe all for a subscriber (e.g., on disconnect) */
	readonly unsubscribeAll: (subscriberId: string) => void
	/** Get all subscribers for a resource */
	readonly getSubscribers: (uri: string) => ReadonlySet<string>
	/** Check if a resource has any subscribers */
	readonly hasSubscribers: (uri: string) => boolean
	/** Get all subscribed URIs for a subscriber */
	readonly getSubscriptions: (subscriberId: string) => ReadonlySet<string>
}

/**
 * Subscription event for notifying about resource updates.
 */
export interface SubscriptionEvent {
	readonly type: "subscribed" | "unsubscribed" | "updated"
	readonly uri: string
	readonly subscriberId?: string
}

/**
 * Subscription event handler.
 */
export type SubscriptionEventHandler = (event: SubscriptionEvent) => void
