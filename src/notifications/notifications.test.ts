import { describe, expect, test } from "bun:test"
import {
	cancelled,
	createEmitter,
	createLogger,
	createProgressReporter,
	log,
	noopEmitter,
	progress,
	promptsListChanged,
	resourceUpdated,
	resourcesListChanged,
	toolsListChanged,
	withProgress,
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
			// Should not throw
			noopEmitter.emit({ type: "progress", progressToken: "t", progress: 0 })
			noopEmitter.emit({ type: "log", level: "info" })
		})
	})

	describe("progress helpers", () => {
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

		test("createProgressReporter emits progress", () => {
			const calls: unknown[] = []
			const emitter = createEmitter((_, params) => calls.push(params))

			const report = createProgressReporter(emitter, "token", 100)
			report(25, "25% done")
			report(75, "75% done")

			expect(calls).toEqual([
				{ progressToken: "token", progress: 25, total: 100, message: "25% done" },
				{ progressToken: "token", progress: 75, total: 100, message: "75% done" },
			])
		})

		test("withProgress tracks async processing", async () => {
			const calls: unknown[] = []
			const emitter = createEmitter((_, params) => calls.push(params))

			const items = ["a", "b", "c"]
			const results = await withProgress(emitter, "token", items, async (item, report) => {
				report(`Processing ${item}`)
				return item.toUpperCase()
			})

			expect(results).toEqual(["A", "B", "C"])
			expect(calls).toEqual([
				{ progressToken: "token", progress: 1, total: 3, message: "Processing a" },
				{ progressToken: "token", progress: 2, total: 3, message: "Processing b" },
				{ progressToken: "token", progress: 3, total: 3, message: "Processing c" },
			])
		})
	})

	describe("logging helpers", () => {
		test("log creates notification", () => {
			const notification = log("error", { message: "Failed" }, "my-tool")

			expect(notification).toEqual({
				type: "log",
				level: "error",
				logger: "my-tool",
				data: { message: "Failed" },
			})
		})

		test("createLogger emits at all levels", () => {
			const calls: unknown[] = []
			const emitter = createEmitter((_, params) => calls.push(params))

			const logger = createLogger(emitter, "test")
			logger.debug("d")
			logger.info("i")
			logger.notice("n")
			logger.warning("w")
			logger.error("e")
			logger.critical("c")
			logger.alert("a")
			logger.emergency("em")

			expect(calls).toEqual([
				{ level: "debug", logger: "test", data: "d" },
				{ level: "info", logger: "test", data: "i" },
				{ level: "notice", logger: "test", data: "n" },
				{ level: "warning", logger: "test", data: "w" },
				{ level: "error", logger: "test", data: "e" },
				{ level: "critical", logger: "test", data: "c" },
				{ level: "alert", logger: "test", data: "a" },
				{ level: "emergency", logger: "test", data: "em" },
			])
		})
	})

	describe("list change helpers", () => {
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
	})

	describe("cancellation helper", () => {
		test("cancelled creates notification", () => {
			expect(cancelled(123, "Timeout")).toEqual({
				type: "cancelled",
				requestId: 123,
				reason: "Timeout",
			})
		})
	})
})
