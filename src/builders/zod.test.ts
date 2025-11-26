import { describe, expect, test } from "bun:test"
import { z } from "zod"
import * as Rpc from "../protocol/jsonrpc.js"
import * as Mcp from "../protocol/mcp.js"
import { createContext, createServer } from "../server/server.js"
import { definePrompt, user } from "./prompt.js"
import { createTool, defineTool, text, toProtocolTool } from "./tool.js"
import type { ToolContext } from "./tool.js"

describe("Zod Tool Builder", () => {
	describe("defineTool", () => {
		test("creates tool with Zod schema", () => {
			const greet = defineTool({
				name: "greet",
				description: "Greet someone",
				input: z.object({
					name: z.string().describe("Name to greet"),
				}),
				handler:
					({ name }) =>
					() =>
						text(`Hello, ${name}!`),
			})

			expect(greet.name).toBe("greet")
			expect(greet.description).toBe("Greet someone")
			expect(greet.schema).toBeDefined()
		})

		test("converts Zod schema to JSON Schema", () => {
			const tool = defineTool({
				name: "test",
				input: z.object({
					name: z.string(),
					age: z.number().optional(),
				}),
				handler: () => () => text("ok"),
			})

			const proto = toProtocolTool(tool)
			expect(proto.inputSchema.type).toBe("object")
			expect(proto.inputSchema.properties).toHaveProperty("name")
			expect(proto.inputSchema.properties).toHaveProperty("age")
			expect(proto.inputSchema.required).toEqual(["name"])
		})

		test("validates input and returns error on invalid", async () => {
			const tool = defineTool({
				name: "test",
				input: z.object({
					count: z.number().min(0),
				}),
				handler:
					({ count }) =>
					() =>
						text(`Count: ${count}`),
			})

			const ctx: ToolContext = {}
			const result = await tool.handler({ count: -1 } as unknown)(ctx)

			expect(result.isError).toBe(true)
			expect(result.content[0]?.type).toBe("text")
		})

		test("validates input and passes on valid", async () => {
			const tool = defineTool({
				name: "test",
				input: z.object({
					count: z.number().min(0),
				}),
				handler:
					({ count }) =>
					() =>
						text(`Count: ${count}`),
			})

			const ctx: ToolContext = {}
			const result = await tool.handler({ count: 42 })(ctx)

			expect(result.isError).toBeUndefined()
			expect(result.content[0]).toEqual({ type: "text", text: "Count: 42" })
		})

		test("applies transforms during validation", async () => {
			const tool = defineTool({
				name: "test",
				input: z.object({
					value: z.coerce.number(),
				}),
				handler:
					({ value }) =>
					() =>
						text(`Value: ${value}, type: ${typeof value}`),
			})

			const ctx: ToolContext = {}
			const result = await tool.handler({ value: "123" } as unknown)(ctx)

			expect(result.content[0]).toEqual({
				type: "text",
				text: "Value: 123, type: number",
			})
		})

		test("applies defaults during validation", async () => {
			const tool = defineTool({
				name: "test",
				input: z.object({
					name: z.string(),
					greeting: z.string().default("Hello"),
				}),
				handler:
					({ name, greeting }) =>
					() =>
						text(`${greeting}, ${name}!`),
			})

			const ctx: ToolContext = {}
			const result = await tool.handler({ name: "World" } as unknown)(ctx)

			expect(result.content[0]).toEqual({ type: "text", text: "Hello, World!" })
		})
	})

	describe("createTool", () => {
		test("works with Zod schema", async () => {
			const tool = createTool({
				name: "test",
				input: z.object({ x: z.number() }),
				handler:
					({ x }) =>
					() =>
						text(`x = ${x}`),
			})

			const ctx: ToolContext = {}
			const result = await tool.handler({ x: 42 })(ctx)
			expect(result.content[0]).toEqual({ type: "text", text: "x = 42" })
		})

		test("works with JSON Schema", async () => {
			const tool = createTool<{ x: number }>({
				name: "test",
				input: { type: "object", properties: { x: { type: "number" } } },
				handler:
					({ x }) =>
					() =>
						text(`x = ${x}`),
			})

			const ctx: ToolContext = {}
			const result = await tool.handler({ x: 42 })(ctx)
			expect(result.content[0]).toEqual({ type: "text", text: "x = 42" })
		})
	})

	describe("Integration with Server", () => {
		test("server handles Zod-validated tool", async () => {
			const addTool = defineTool({
				name: "add",
				input: z.object({
					a: z.number(),
					b: z.number(),
				}),
				handler:
					({ a, b }) =>
					() =>
						text(`${a + b}`),
			})

			const server = createServer({
				name: "test",
				version: "1.0.0",
				tools: [addTool],
			})

			// Valid call
			const validReq = Rpc.request(1, Mcp.Method.ToolsCall, {
				name: "add",
				arguments: { a: 2, b: 3 },
			})
			const validRes = await server.handleMessage(validReq, createContext())
			expect(Rpc.isSuccess(validRes!)).toBe(true)
			if (Rpc.isSuccess(validRes!)) {
				const result = validRes.result as Mcp.ToolsCallResult
				expect(result.content[0]).toEqual({ type: "text", text: "5" })
			}

			// Invalid call
			const invalidReq = Rpc.request(2, Mcp.Method.ToolsCall, {
				name: "add",
				arguments: { a: "two", b: 3 },
			})
			const invalidRes = await server.handleMessage(invalidReq, createContext())
			expect(Rpc.isSuccess(invalidRes!)).toBe(true)
			if (Rpc.isSuccess(invalidRes!)) {
				const result = invalidRes.result as Mcp.ToolsCallResult
				expect(result.isError).toBe(true)
			}
		})
	})
})

describe("Zod Prompt Builder", () => {
	describe("definePrompt", () => {
		test("creates prompt with Zod schema", () => {
			const prompt = definePrompt({
				name: "review",
				description: "Code review",
				args: z.object({
					language: z.string(),
					focus: z.string().optional(),
				}),
				handler:
					({ language, focus }) =>
					() => ({
						messages: [user(`Review ${language} code${focus ? ` (${focus})` : ""}`)],
					}),
			})

			expect(prompt.name).toBe("review")
			expect(prompt.schema).toBeDefined()
		})

		test("extracts arguments from schema", () => {
			const prompt = definePrompt({
				name: "test",
				args: z.object({
					required: z.string(),
					optional: z.string().optional(),
					withDefault: z.string().default("default"),
				}),
				handler: () => () => ({ messages: [user("test")] }),
			})

			expect(prompt.arguments).toHaveLength(3)

			const reqArg = prompt.arguments.find((a) => a.name === "required")
			expect(reqArg?.required).toBe(true)

			const optArg = prompt.arguments.find((a) => a.name === "optional")
			expect(optArg?.required).toBe(false)

			const defArg = prompt.arguments.find((a) => a.name === "withDefault")
			expect(defArg?.required).toBe(false)
		})

		test("includes description from schema", () => {
			const prompt = definePrompt({
				name: "test",
				args: z.object({
					name: z.string().describe("The user name"),
				}),
				handler: () => () => ({ messages: [user("test")] }),
			})

			const arg = prompt.arguments.find((a) => a.name === "name")
			expect(arg?.description).toBe("The user name")
		})

		test("validates arguments", async () => {
			const prompt = definePrompt({
				name: "test",
				args: z.object({
					count: z.coerce.number().min(1),
				}),
				handler:
					({ count }) =>
					() => ({
						messages: [user(`Count: ${count}`)],
					}),
			})

			// Valid
			const validResult = await prompt.handler({ count: "5" })({})
			expect(validResult.messages[0]?.content).toEqual({ type: "text", text: "Count: 5" })

			// Invalid
			const invalidResult = await prompt.handler({ count: "0" })({})
			expect(invalidResult.messages[0]?.content.type).toBe("text")
			const textContent = invalidResult.messages[0]?.content as { type: "text"; text: string }
			expect(textContent.text).toContain("Error")
		})
	})
})
