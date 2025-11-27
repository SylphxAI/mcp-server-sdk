import { describe, expect, test } from "bun:test"
import {
	cancelled,
	createEmitter,
	log,
	noopEmitter,
	progress,
	promptsListChanged,
	resourcesListChanged,
	resourceUpdated,
	toolsListChanged,
} from "./index.js"

describe("Notifications", () => {
	describe("createEmitter", () => {
		test("converts progress notification to JSON-RPC", () => {
			const calls: [string, unknown][] = []
			const emitter = createEmitter((method, params) => {
				calls.push([method, params])
			})

			emitter.emit({
				type: "progress",
				progressToken: "token-123",
				progress: 50,
				total: 100,
				message: "Processing...",
			})

			expect(calls).toHaveLength(1)
			expect(calls[0]?.[0]).toBe("notifications/progress")
			expect(calls[0]?.[1]).toEqual({
				progressToken: "token-123",
				progress: 50,
				total: 100,
				message: "Processing...",
			})
		})

		test("converts log notification to JSON-RPC", () => {
			const calls: [string, unknown][] = []
			const emitter = createEmitter((method, params) => {
				calls.push([method, params])
			})

			emitter.emit({
				type: "log",
				level: "info",
				logger: "my-tool",
				data: { message: "Hello" },
			})

			expect(calls).toHaveLength(1)
			expect(calls[0]?.[0]).toBe("notifications/message")
			expect(calls[0]?.[1]).toEqual({
				level: "info",
				logger: "my-tool",
				data: { message: "Hello" },
			})
		})

		test("converts list changed notifications", () => {
			const calls: [string, unknown][] = []
			const emitter = createEmitter((method, params) => {
				calls.push([method, params])
			})

			emitter.emit({ type: "resources/list_changed" })
			emitter.emit({ type: "tools/list_changed" })
			emitter.emit({ type: "prompts/list_changed" })

			expect(calls).toHaveLength(3)
			expect(calls[0]?.[0]).toBe("notifications/resources/list_changed")
			expect(calls[1]?.[0]).toBe("notifications/tools/list_changed")
			expect(calls[2]?.[0]).toBe("notifications/prompts/list_changed")
		})

		test("converts resource updated notification", () => {
			const calls: [string, unknown][] = []
			const emitter = createEmitter((method, params) => {
				calls.push([method, params])
			})

			emitter.emit({ type: "resource/updated", uri: "file:///test.txt" })

			expect(calls).toHaveLength(1)
			expect(calls[0]?.[0]).toBe("notifications/resources/updated")
			expect(calls[0]?.[1]).toEqual({ uri: "file:///test.txt" })
		})

		test("converts cancelled notification", () => {
			const calls: [string, unknown][] = []
			const emitter = createEmitter((method, params) => {
				calls.push([method, params])
			})

			emitter.emit({ type: "cancelled", requestId: 123, reason: "User cancelled" })

			expect(calls).toHaveLength(1)
			expect(calls[0]?.[0]).toBe("notifications/cancelled")
			expect(calls[0]?.[1]).toEqual({ requestId: 123, reason: "User cancelled" })
		})
	})

	describe("noopEmitter", () => {
		test("does nothing", () => {
			noopEmitter.emit({ type: "progress", progressToken: "t", progress: 0 })
			noopEmitter.emit({ type: "log", level: "info" })
		})
	})

	describe("notification factories", () => {
		test("progress creates notification", () => {
			const notification = progress("token", 50, { total: 100, message: "Working..." })

			expect(notification).toEqual({
				type: "progress",
				progressToken: "token",
				progress: 50,
				total: 100,
				message: "Working...",
			})
		})

		test("log creates notification", () => {
			const notification = log("error", { message: "Failed" }, "my-tool")

			expect(notification).toEqual({
				type: "log",
				level: "error",
				logger: "my-tool",
				data: { message: "Failed" },
			})
		})

		test("resourcesListChanged creates notification", () => {
			expect(resourcesListChanged()).toEqual({ type: "resources/list_changed" })
		})

		test("toolsListChanged creates notification", () => {
			expect(toolsListChanged()).toEqual({ type: "tools/list_changed" })
		})

		test("promptsListChanged creates notification", () => {
			expect(promptsListChanged()).toEqual({ type: "prompts/list_changed" })
		})

		test("resourceUpdated creates notification", () => {
			expect(resourceUpdated("file:///test")).toEqual({
				type: "resource/updated",
				uri: "file:///test",
			})
		})

		test("cancelled creates notification", () => {
			expect(cancelled(123, "Timeout")).toEqual({
				type: "cancelled",
				requestId: 123,
				reason: "Timeout",
			})
		})
	})
})
