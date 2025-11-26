import { describe, expect, test } from "bun:test"
import { z } from "zod"
import {
	arg,
	assistant,
	definePrompt,
	interpolate,
	message,
	messages,
	prompt,
	promptResult,
	templatePrompt,
	toProtocolPrompt,
	user,
} from "./prompt.js"

describe("Prompt Builder", () => {
	describe("prompt", () => {
		test("creates prompt with handler", () => {
			const p = prompt({
				name: "test",
				handler: () => () => ({ messages: [user("Hello")] }),
			})

			expect(p.name).toBe("test")
		})

		test("creates prompt with arguments", () => {
			const p = prompt({
				name: "greet",
				description: "Greet user",
				arguments: [arg({ name: "name", description: "User name", required: true })],
				handler: (args) => () => ({
					messages: [user(`Hello ${args.name}`)],
				}),
			})

			expect(p.arguments).toHaveLength(1)
			expect(p.arguments?.[0]).toEqual({
				name: "name",
				description: "User name",
				required: true,
			})
		})
	})

	describe("arg", () => {
		test("creates required argument", () => {
			const a = arg({ name: "name", description: "User name", required: true })

			expect(a).toEqual({
				name: "name",
				description: "User name",
				required: true,
			})
		})

		test("creates optional argument", () => {
			const a = arg({ name: "suffix", description: "Optional suffix", required: false })

			expect(a).toEqual({
				name: "suffix",
				description: "Optional suffix",
				required: false,
			})
		})

		test("creates argument with minimal config", () => {
			const a = arg({ name: "value" })

			expect(a).toEqual({
				name: "value",
				description: undefined,
				required: undefined,
			})
		})
	})

	describe("user", () => {
		test("creates user message with text", () => {
			const msg = user("Hello")

			expect(msg).toEqual({
				role: "user",
				content: { type: "text", text: "Hello" },
			})
		})

		test("creates user message with empty string", () => {
			const msg = user("")

			expect(msg.content.text).toBe("")
		})
	})

	describe("assistant", () => {
		test("creates assistant message with text", () => {
			const msg = assistant("Hi there")

			expect(msg).toEqual({
				role: "assistant",
				content: { type: "text", text: "Hi there" },
			})
		})
	})

	describe("message", () => {
		test("creates message with text content", () => {
			const msg = message("user", { type: "text", text: "Test" })

			expect(msg).toEqual({
				role: "user",
				content: { type: "text", text: "Test" },
			})
		})

		test("creates message with image content", () => {
			const msg = message("user", {
				type: "image",
				data: "base64",
				mimeType: "image/png",
			})

			expect(msg.content.type).toBe("image")
		})
	})

	describe("messages", () => {
		test("wraps multiple messages", () => {
			const result = messages(user("Q"), assistant("A"))

			expect(result.messages).toHaveLength(2)
		})

		test("handles empty args", () => {
			const result = messages()

			expect(result.messages).toHaveLength(0)
		})
	})

	describe("promptResult", () => {
		test("creates result with description", () => {
			const result = promptResult("A test prompt", user("Test"))

			expect(result).toEqual({
				description: "A test prompt",
				messages: [user("Test")],
			})
		})

		test("creates result with multiple messages", () => {
			const result = promptResult("Dialog", user("Q"), assistant("A"))

			expect(result.messages).toHaveLength(2)
		})
	})

	describe("interpolate", () => {
		test("replaces single variable", () => {
			const result = interpolate("Hello {{name}}!", { name: "World" })

			expect(result).toBe("Hello World!")
		})

		test("replaces multiple variables", () => {
			const result = interpolate("{{greeting}} {{name}}!", {
				greeting: "Hi",
				name: "Alice",
			})

			expect(result).toBe("Hi Alice!")
		})

		test("handles missing variables (leaves placeholder)", () => {
			const result = interpolate("Hello {{name}}!", {})

			expect(result).toBe("Hello {{name}}!")
		})

		test("handles special characters in values", () => {
			const result = interpolate("Code: {{code}}", { code: "if (x > 0) { return; }" })

			expect(result).toBe("Code: if (x > 0) { return; }")
		})
	})

	describe("templatePrompt", () => {
		test("creates prompt from template", () => {
			const p = templatePrompt({
				name: "greet",
				template: "Hello {{name}}!",
				arguments: [arg({ name: "name", description: "Name to greet", required: true })],
			})

			expect(p.name).toBe("greet")
			expect(p.arguments).toHaveLength(1)
		})

		test("template prompt generates correct message", async () => {
			const p = templatePrompt({
				name: "greet",
				template: "Hello {{name}}!",
				arguments: [arg({ name: "name", description: "Name", required: true })],
			})

			const result = await p.handler({ name: "World" })({})

			expect(result.messages[0]).toEqual({
				role: "user",
				content: { type: "text", text: "Hello World!" },
			})
		})
	})

	describe("definePrompt", () => {
		test("creates typed prompt with Zod schema", async () => {
			const p = definePrompt({
				name: "review",
				args: z.object({
					language: z.string(),
				}),
				handler:
					({ language }) =>
					() => ({
						messages: [user(`Review ${language} code`)],
					}),
			})

			expect(p.name).toBe("review")
			expect(p.schema).toBeDefined()
		})

		test("validates input and passes to handler", async () => {
			const p = definePrompt({
				name: "test",
				args: z.object({
					count: z.number(),
				}),
				handler:
					({ count }) =>
					() => ({
						messages: [user(`Count: ${count}`)],
					}),
			})

			const result = await p.handler({ count: 5 })({})

			expect(result.messages[0].content).toEqual({
				type: "text",
				text: "Count: 5",
			})
		})

		test("returns error message on validation failure", async () => {
			const p = definePrompt({
				name: "test",
				args: z.object({
					count: z.number(),
				}),
				handler:
					({ count }) =>
					() => ({
						messages: [user(`Count: ${count}`)],
					}),
			})

			const result = await p.handler({ count: "not a number" } as unknown)({})

			expect(result.messages[0].role).toBe("user")
			expect((result.messages[0].content as { text: string }).text).toContain("Error:")
		})

		test("extracts arguments from Zod schema", () => {
			const p = definePrompt({
				name: "test",
				args: z.object({
					required: z.string(),
					optional: z.string().optional(),
				}),
				handler: () => () => ({ messages: [] }),
			})

			expect(p.arguments).toBeDefined()
			const required = p.arguments?.find((a) => a.name === "required")
			const optional = p.arguments?.find((a) => a.name === "optional")

			expect(required?.required).toBe(true)
			expect(optional?.required).toBe(false)
		})
	})

	describe("toProtocolPrompt", () => {
		test("converts to protocol prompt", () => {
			const p = prompt({
				name: "test",
				description: "Test prompt",
				arguments: [arg({ name: "x", description: "X value", required: true })],
				handler: () => () => ({ messages: [] }),
			})

			const proto = toProtocolPrompt(p)

			expect(proto).toEqual({
				name: "test",
				description: "Test prompt",
				arguments: [{ name: "x", description: "X value", required: true }],
			})
		})
	})
})
