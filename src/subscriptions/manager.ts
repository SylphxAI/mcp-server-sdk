/**
 * Subscription Manager
 *
 * Manages resource subscriptions.
 */

import type { SubscriptionManager, SubscriptionEventHandler } from "./types.js"

// ============================================================================
// Subscription Manager Factory
// ============================================================================

/**
 * Create a subscription manager.
 *
 * @example
 * ```ts
 * const subscriptions = createSubscriptionManager({
 *   onSubscribe: (uri, id) => console.log(`${id} subscribed to ${uri}`),
 *   onUnsubscribe: (uri, id) => console.log(`${id} unsubscribed from ${uri}`),
 * })
 *
 * subscriptions.subscribe("file:///config.json", "session-123")
 * ```
 */
export const createSubscriptionManager = (options?: {
	onSubscribe?: (uri: string, subscriberId: string) => void
	onUnsubscribe?: (uri: string, subscriberId: string) => void
}): SubscriptionManager => {
	// uri -> Set<subscriberId>
	const byResource = new Map<string, Set<string>>()
	// subscriberId -> Set<uri>
	const bySubscriber = new Map<string, Set<string>>()

	const subscribe = (uri: string, subscriberId: string): void => {
		// Add to resource map
		let subscribers = byResource.get(uri)
		if (!subscribers) {
			subscribers = new Set()
			byResource.set(uri, subscribers)
		}
		subscribers.add(subscriberId)

		// Add to subscriber map
		let subscriptions = bySubscriber.get(subscriberId)
		if (!subscriptions) {
			subscriptions = new Set()
			bySubscriber.set(subscriberId, subscriptions)
		}
		subscriptions.add(uri)

		options?.onSubscribe?.(uri, subscriberId)
	}

	const unsubscribe = (uri: string, subscriberId: string): void => {
		// Remove from resource map
		const subscribers = byResource.get(uri)
		if (subscribers) {
			subscribers.delete(subscriberId)
			if (subscribers.size === 0) {
				byResource.delete(uri)
			}
		}

		// Remove from subscriber map
		const subscriptions = bySubscriber.get(subscriberId)
		if (subscriptions) {
			subscriptions.delete(uri)
			if (subscriptions.size === 0) {
				bySubscriber.delete(subscriberId)
			}
		}

		options?.onUnsubscribe?.(uri, subscriberId)
	}

	const unsubscribeAll = (subscriberId: string): void => {
		const subscriptions = bySubscriber.get(subscriberId)
		if (!subscriptions) return

		for (const uri of subscriptions) {
			const subscribers = byResource.get(uri)
			if (subscribers) {
				subscribers.delete(subscriberId)
				if (subscribers.size === 0) {
					byResource.delete(uri)
				}
			}
			options?.onUnsubscribe?.(uri, subscriberId)
		}

		bySubscriber.delete(subscriberId)
	}

	const getSubscribers = (uri: string): ReadonlySet<string> => {
		return byResource.get(uri) ?? new Set()
	}

	const hasSubscribers = (uri: string): boolean => {
		const subscribers = byResource.get(uri)
		return subscribers !== undefined && subscribers.size > 0
	}

	const getSubscriptions = (subscriberId: string): ReadonlySet<string> => {
		return bySubscriber.get(subscriberId) ?? new Set()
	}

	return {
		subscribe,
		unsubscribe,
		unsubscribeAll,
		getSubscribers,
		hasSubscribers,
		getSubscriptions,
	}
}

// ============================================================================
// Helpers
// ============================================================================

/**
 * Notify all subscribers of a resource update.
 */
export const notifySubscribers = (
	manager: SubscriptionManager,
	uri: string,
	notify: (subscriberId: string) => void,
): void => {
	const subscribers = manager.getSubscribers(uri)
	for (const subscriberId of subscribers) {
		notify(subscriberId)
	}
}
