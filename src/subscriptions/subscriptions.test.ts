import { describe, expect, test } from "bun:test"
import { createSubscriptionManager, notifySubscribers } from "./index.js"

describe("Subscriptions", () => {
	describe("createSubscriptionManager", () => {
		test("subscribes to resource", () => {
			const manager = createSubscriptionManager()

			manager.subscribe("file:///test.txt", "session-1")

			expect(manager.hasSubscribers("file:///test.txt")).toBe(true)
			expect(manager.getSubscribers("file:///test.txt").has("session-1")).toBe(true)
		})

		test("unsubscribes from resource", () => {
			const manager = createSubscriptionManager()

			manager.subscribe("file:///test.txt", "session-1")
			manager.unsubscribe("file:///test.txt", "session-1")

			expect(manager.hasSubscribers("file:///test.txt")).toBe(false)
		})

		test("tracks multiple subscribers", () => {
			const manager = createSubscriptionManager()

			manager.subscribe("file:///test.txt", "session-1")
			manager.subscribe("file:///test.txt", "session-2")

			const subscribers = manager.getSubscribers("file:///test.txt")
			expect(subscribers.size).toBe(2)
			expect(subscribers.has("session-1")).toBe(true)
			expect(subscribers.has("session-2")).toBe(true)
		})

		test("tracks subscriptions per subscriber", () => {
			const manager = createSubscriptionManager()

			manager.subscribe("file:///a.txt", "session-1")
			manager.subscribe("file:///b.txt", "session-1")

			const subscriptions = manager.getSubscriptions("session-1")
			expect(subscriptions.size).toBe(2)
			expect(subscriptions.has("file:///a.txt")).toBe(true)
			expect(subscriptions.has("file:///b.txt")).toBe(true)
		})

		test("unsubscribeAll removes all subscriptions for subscriber", () => {
			const manager = createSubscriptionManager()

			manager.subscribe("file:///a.txt", "session-1")
			manager.subscribe("file:///b.txt", "session-1")
			manager.subscribe("file:///a.txt", "session-2")

			manager.unsubscribeAll("session-1")

			expect(manager.getSubscriptions("session-1").size).toBe(0)
			expect(manager.hasSubscribers("file:///a.txt")).toBe(true) // session-2 still subscribed
			expect(manager.hasSubscribers("file:///b.txt")).toBe(false)
		})

		test("calls onSubscribe callback", () => {
			const calls: [string, string][] = []
			const manager = createSubscriptionManager({
				onSubscribe: (uri, id) => calls.push([uri, id]),
			})

			manager.subscribe("file:///test.txt", "session-1")

			expect(calls).toEqual([["file:///test.txt", "session-1"]])
		})

		test("calls onUnsubscribe callback", () => {
			const calls: [string, string][] = []
			const manager = createSubscriptionManager({
				onUnsubscribe: (uri, id) => calls.push([uri, id]),
			})

			manager.subscribe("file:///test.txt", "session-1")
			manager.unsubscribe("file:///test.txt", "session-1")

			expect(calls).toEqual([["file:///test.txt", "session-1"]])
		})

		test("returns empty set for unknown resource", () => {
			const manager = createSubscriptionManager()

			const subscribers = manager.getSubscribers("file:///unknown.txt")

			expect(subscribers.size).toBe(0)
		})

		test("returns empty set for unknown subscriber", () => {
			const manager = createSubscriptionManager()

			const subscriptions = manager.getSubscriptions("unknown")

			expect(subscriptions.size).toBe(0)
		})
	})

	describe("notifySubscribers", () => {
		test("notifies all subscribers", () => {
			const manager = createSubscriptionManager()
			const notified: string[] = []

			manager.subscribe("file:///test.txt", "session-1")
			manager.subscribe("file:///test.txt", "session-2")

			notifySubscribers(manager, "file:///test.txt", (id) => notified.push(id))

			expect(notified).toContain("session-1")
			expect(notified).toContain("session-2")
		})

		test("does not notify unsubscribed", () => {
			const manager = createSubscriptionManager()
			const notified: string[] = []

			manager.subscribe("file:///test.txt", "session-1")
			manager.unsubscribe("file:///test.txt", "session-1")

			notifySubscribers(manager, "file:///test.txt", (id) => notified.push(id))

			expect(notified).toEqual([])
		})
	})
})
