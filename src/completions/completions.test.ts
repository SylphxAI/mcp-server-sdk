import { describe, expect, test } from "bun:test"
import {
	buildCompletionRegistry,
	handleComplete,
	staticCompletions,
	dynamicCompletions,
	mergeCompletions,
} from "./index.js"

describe("Completions", () => {
	describe("buildCompletionRegistry", () => {
		test("builds registry from configs", () => {
			const registry = buildCompletionRegistry([
				{
					type: "prompt",
					name: "greet",
					provider: staticCompletions(["hello", "hi"]),
				},
				{
					type: "resource",
					uriTemplate: "file:///{path}",
					provider: staticCompletions(["readme.md", "package.json"]),
				},
			])

			expect(registry.prompts.size).toBe(1)
			expect(registry.resources.size).toBe(1)
		})

		test("empty configs produces empty registry", () => {
			const registry = buildCompletionRegistry([])
			expect(registry.prompts.size).toBe(0)
			expect(registry.resources.size).toBe(0)
		})
	})

	describe("handleComplete", () => {
		test("completes prompt argument", async () => {
			const registry = buildCompletionRegistry([
				{
					type: "prompt",
					name: "greet",
					provider: staticCompletions(["hello", "hi", "hey"]),
				},
			])

			const result = await handleComplete(registry, {
				ref: { type: "ref/prompt", name: "greet" },
				argument: { name: "greeting", value: "h" },
			})

			expect(result.completion.values).toEqual(["hello", "hi", "hey"])
		})

		test("filters by prefix", async () => {
			const registry = buildCompletionRegistry([
				{
					type: "prompt",
					name: "greet",
					provider: staticCompletions(["hello", "hi", "goodbye"]),
				},
			])

			const result = await handleComplete(registry, {
				ref: { type: "ref/prompt", name: "greet" },
				argument: { name: "greeting", value: "h" },
			})

			expect(result.completion.values).toEqual(["hello", "hi"])
		})

		test("completes resource argument", async () => {
			const registry = buildCompletionRegistry([
				{
					type: "resource",
					uriTemplate: "file:///{path}",
					provider: staticCompletions(["readme.md", "package.json"]),
				},
			])

			const result = await handleComplete(registry, {
				ref: { type: "ref/resource", uri: "file:///test" },
				argument: { name: "path", value: "p" },
			})

			expect(result.completion.values).toEqual(["package.json"])
		})

		test("returns empty for unknown prompt", async () => {
			const registry = buildCompletionRegistry([])

			const result = await handleComplete(registry, {
				ref: { type: "ref/prompt", name: "unknown" },
				argument: { name: "arg", value: "" },
			})

			expect(result.completion.values).toEqual([])
		})
	})

	describe("staticCompletions", () => {
		test("returns filtered values", async () => {
			const provider = staticCompletions(["apple", "apricot", "banana"])
			const result = await provider("fruit", "ap")

			expect(result.values).toEqual(["apple", "apricot"])
			expect(result.hasMore).toBe(false)
		})

		test("case insensitive matching", async () => {
			const provider = staticCompletions(["Apple", "APRICOT", "banana"])
			const result = await provider("fruit", "ap")

			expect(result.values).toEqual(["Apple", "APRICOT"])
		})
	})

	describe("dynamicCompletions", () => {
		test("calls function with prefix", async () => {
			const provider = dynamicCompletions(async (prefix) => {
				return [`${prefix}-1`, `${prefix}-2`]
			})

			const result = await provider("name", "test")

			expect(result.values).toEqual(["test-1", "test-2"])
		})
	})

	describe("mergeCompletions", () => {
		test("combines multiple providers", async () => {
			const provider = mergeCompletions(
				staticCompletions(["a", "b"]),
				staticCompletions(["c", "d"]),
			)

			const result = await provider("name", "")

			expect(result.values).toContain("a")
			expect(result.values).toContain("b")
			expect(result.values).toContain("c")
			expect(result.values).toContain("d")
		})

		test("deduplicates values", async () => {
			const provider = mergeCompletions(
				staticCompletions(["a", "b"]),
				staticCompletions(["b", "c"]),
			)

			const result = await provider("name", "")

			expect(result.values.filter((v) => v === "b").length).toBe(1)
		})
	})
})
