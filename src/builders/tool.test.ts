import { describe, expect, test } from "bun:test"
import {
	type ToolContext,
	contents,
	guard,
	mapResult,
	sequence,
	structured,
	text,
	textContent,
	toProtocolTool,
	tool,
	toolError,
} from "./tool.js"

describe("Tool Builder", () => {
	describe("tool", () => {
		test("creates tool definition", () => {
			const t = tool({
				name: "greet",
				description: "Greet someone",
				input: {
					type: "object",
					properties: { name: { type: "string" } },
					required: ["name"],
				},
				handler:
					({ name }: { name: string }) =>
					() =>
						text(`Hello, ${name}!`),
			})

			expect(t.name).toBe("greet")
			expect(t.description).toBe("Greet someone")
			expect(t.inputSchema.type).toBe("object")
		})

		test("handler executes correctly", async () => {
			const t = tool({
				name: "add",
				input: { type: "object" },
				handler:
					({ a, b }: { a: number; b: number }) =>
					() =>
						text(`${a + b}`),
			})

			const ctx: ToolContext = {}
			const result = await t.handler({ a: 2, b: 3 })(ctx)

			expect(result.content[0]).toEqual({ type: "text", text: "5" })
		})

		test("async handler works", async () => {
			const t = tool({
				name: "async-test",
				input: { type: "object" },
				handler: () => async () => {
					await new Promise((r) => setTimeout(r, 1))
					return text("done")
				},
			})

			const result = await t.handler({})({} as ToolContext)
			expect(result.content[0]).toEqual({ type: "text", text: "done" })
		})
	})

	describe("toProtocolTool", () => {
		test("extracts protocol tool", () => {
			const t = tool({
				name: "test",
				description: "Test tool",
				input: { type: "object" },
				annotations: { readOnlyHint: true },
				handler: () => () => text("ok"),
			})

			const proto = toProtocolTool(t)

			expect(proto).toEqual({
				name: "test",
				description: "Test tool",
				inputSchema: { type: "object" },
				annotations: { readOnlyHint: true },
			})
		})
	})

	describe("content helpers", () => {
		test("textContent creates text content", () => {
			expect(textContent("hello")).toEqual({ type: "text", text: "hello" })
		})

		test("text creates success result", () => {
			expect(text("hello")).toEqual({
				content: [{ type: "text", text: "hello" }],
			})
		})

		test("contents creates multi-content result", () => {
			const result = contents({ type: "text", text: "a" }, { type: "text", text: "b" })
			expect(result.content).toHaveLength(2)
		})

		test("toolError creates error result", () => {
			const result = toolError("something went wrong")
			expect(result.isError).toBe(true)
			expect(result.content[0]).toEqual({
				type: "text",
				text: "something went wrong",
			})
		})

		test("structured creates result with structured content", () => {
			const result = structured("User data", { id: 1, name: "Test" })
			expect(result.content[0]).toEqual({ type: "text", text: "User data" })
			expect(result.structuredContent).toEqual({ id: 1, name: "Test" })
		})
	})

	describe("composition", () => {
		test("sequence runs handlers in order", async () => {
			const h1 = () => () => text("first")
			const h2 = () => () => text("second")

			const combined = sequence(h1, h2)
			const result = await combined({})({} as ToolContext)

			expect(result.content).toHaveLength(2)
			expect(result.content[0]).toEqual({ type: "text", text: "first" })
			expect(result.content[1]).toEqual({ type: "text", text: "second" })
		})

		test("sequence stops on error", async () => {
			const h1 = () => () => toolError("failed")
			const h2 = () => () => text("should not run")

			const combined = sequence(h1, h2)
			const result = await combined({})({} as ToolContext)

			expect(result.isError).toBe(true)
			expect(result.content).toHaveLength(1)
		})

		test("guard prevents execution on false", async () => {
			const h = () => () => text("ok")
			const guarded = guard(() => false, h)

			const result = await guarded({})({} as ToolContext)
			expect(result.isError).toBe(true)
		})

		test("guard allows execution on true", async () => {
			const h = () => () => text("ok")
			const guarded = guard(() => true, h)

			const result = await guarded({})({} as ToolContext)
			expect(result.isError).toBeUndefined()
		})

		test("guard returns custom error message", async () => {
			const h = () => () => text("ok")
			const guarded = guard(() => "custom error", h)

			const result = await guarded({})({} as ToolContext)
			expect(result.isError).toBe(true)
			expect(result.content[0]).toEqual({ type: "text", text: "custom error" })
		})

		test("mapResult transforms output", async () => {
			const h = () => () => text("original")
			const mapped = mapResult(h, (r) => ({
				...r,
				structuredContent: { transformed: true },
			}))

			const result = await mapped({})({} as ToolContext)
			expect(result.structuredContent).toEqual({ transformed: true })
		})
	})
})
