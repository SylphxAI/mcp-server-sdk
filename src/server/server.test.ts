import { describe, expect, test } from "bun:test"
import { prompt, user } from "../builders/prompt.js"
import { resource, resourceText } from "../builders/resource.js"
import { text, tool } from "../builders/tool.js"
import * as Rpc from "../protocol/jsonrpc.js"
import * as Mcp from "../protocol/mcp.js"
import { createContext, createServer } from "./server.js"

describe("Server", () => {
	const greetTool = tool({
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

	const configResource = resource({
		uri: "config://app",
		name: "App Config",
		mimeType: "application/json",
		handler: () => () => resourceText("config://app", '{"version":"1.0"}', "application/json"),
	})

	const greetPrompt = prompt({
		name: "greet",
		description: "Generate greeting",
		handler:
			({ name }: Record<string, string>) =>
			() => ({
				messages: [user(`Please greet ${name}`)],
			}),
	})

	const server = createServer({
		name: "test-server",
		version: "1.0.0",
		instructions: "Test server",
		tools: [greetTool],
		resources: [configResource],
		prompts: [greetPrompt],
	})

	describe("createServer", () => {
		test("creates server with correct metadata", () => {
			expect(server.name).toBe("test-server")
			expect(server.version).toBe("1.0.0")
		})

		test("builds capabilities from definitions", () => {
			const caps = server.state.capabilities
			expect(caps.tools).toBeDefined()
			expect(caps.resources).toBeDefined()
			expect(caps.prompts).toBeDefined()
		})
	})

	describe("handle", () => {
		test("returns parse error for invalid JSON", async () => {
			const response = await server.handle("{invalid}", createContext())
			const parsed = JSON.parse(response!)

			expect(parsed.error.code).toBe(Rpc.ErrorCode.ParseError)
		})

		test("returns null for notifications", async () => {
			const notif = Rpc.stringify(Rpc.notification("notifications/initialized"))
			const response = await server.handle(notif, createContext())

			expect(response).toBeNull()
		})
	})

	describe("initialize", () => {
		test("returns server info and capabilities", async () => {
			const req = Rpc.request(1, Mcp.Method.Initialize, {
				protocolVersion: Mcp.LATEST_PROTOCOL_VERSION,
				capabilities: {},
				clientInfo: { name: "test-client", version: "1.0.0" },
			})

			const response = await server.handleMessage(req, createContext())

			expect(response).not.toBeNull()
			expect(Rpc.isSuccess(response!)).toBe(true)

			if (Rpc.isSuccess(response!)) {
				const result = response.result as Mcp.InitializeResult
				expect(result.serverInfo.name).toBe("test-server")
				expect(result.protocolVersion).toBe(Mcp.LATEST_PROTOCOL_VERSION)
				expect(result.capabilities.tools).toBeDefined()
			}
		})
	})

	describe("ping", () => {
		test("returns empty object", async () => {
			const req = Rpc.request(1, Mcp.Method.Ping)
			const response = await server.handleMessage(req, createContext())

			expect(Rpc.isSuccess(response!)).toBe(true)
			if (Rpc.isSuccess(response!)) {
				expect(response.result).toEqual({})
			}
		})
	})

	describe("tools/list", () => {
		test("returns tool definitions", async () => {
			const req = Rpc.request(1, Mcp.Method.ToolsList)
			const response = await server.handleMessage(req, createContext())

			expect(Rpc.isSuccess(response!)).toBe(true)
			if (Rpc.isSuccess(response!)) {
				const result = response.result as Mcp.ToolsListResult
				expect(result.items).toHaveLength(1)
				expect(result.items[0]?.name).toBe("greet")
			}
		})
	})

	describe("tools/call", () => {
		test("executes tool handler", async () => {
			const req = Rpc.request(1, Mcp.Method.ToolsCall, {
				name: "greet",
				arguments: { name: "World" },
			})

			const response = await server.handleMessage(req, createContext())

			expect(Rpc.isSuccess(response!)).toBe(true)
			if (Rpc.isSuccess(response!)) {
				const result = response.result as Mcp.ToolsCallResult
				expect(result.content[0]).toEqual({ type: "text", text: "Hello, World!" })
			}
		})

		test("returns error for unknown tool", async () => {
			const req = Rpc.request(1, Mcp.Method.ToolsCall, {
				name: "unknown",
				arguments: {},
			})

			const response = await server.handleMessage(req, createContext())

			expect(Rpc.isSuccess(response!)).toBe(true)
			if (Rpc.isSuccess(response!)) {
				const result = response.result as Mcp.ToolsCallResult
				expect(result.isError).toBe(true)
			}
		})
	})

	describe("resources/list", () => {
		test("returns resource definitions", async () => {
			const req = Rpc.request(1, Mcp.Method.ResourcesList)
			const response = await server.handleMessage(req, createContext())

			expect(Rpc.isSuccess(response!)).toBe(true)
			if (Rpc.isSuccess(response!)) {
				const result = response.result as Mcp.ResourcesListResult
				expect(result.items).toHaveLength(1)
				expect(result.items[0]?.uri).toBe("config://app")
			}
		})
	})

	describe("resources/read", () => {
		test("reads resource content", async () => {
			const req = Rpc.request(1, Mcp.Method.ResourcesRead, {
				uri: "config://app",
			})

			const response = await server.handleMessage(req, createContext())

			expect(Rpc.isSuccess(response!)).toBe(true)
			if (Rpc.isSuccess(response!)) {
				const result = response.result as Mcp.ResourcesReadResult
				expect(result.contents[0]?.text).toBe('{"version":"1.0"}')
			}
		})

		test("returns error for unknown resource", async () => {
			const req = Rpc.request(1, Mcp.Method.ResourcesRead, {
				uri: "unknown://resource",
			})

			const response = await server.handleMessage(req, createContext())

			expect(Rpc.isError(response!)).toBe(true)
		})
	})

	describe("prompts/list", () => {
		test("returns prompt definitions", async () => {
			const req = Rpc.request(1, Mcp.Method.PromptsList)
			const response = await server.handleMessage(req, createContext())

			expect(Rpc.isSuccess(response!)).toBe(true)
			if (Rpc.isSuccess(response!)) {
				const result = response.result as Mcp.PromptsListResult
				expect(result.items).toHaveLength(1)
				expect(result.items[0]?.name).toBe("greet")
			}
		})
	})

	describe("prompts/get", () => {
		test("generates prompt messages", async () => {
			const req = Rpc.request(1, Mcp.Method.PromptsGet, {
				name: "greet",
				arguments: { name: "User" },
			})

			const response = await server.handleMessage(req, createContext())

			expect(Rpc.isSuccess(response!)).toBe(true)
			if (Rpc.isSuccess(response!)) {
				const result = response.result as Mcp.PromptsGetResult
				expect(result.messages).toHaveLength(1)
				expect(result.messages[0]?.role).toBe("user")
			}
		})

		test("returns error for unknown prompt", async () => {
			const req = Rpc.request(1, Mcp.Method.PromptsGet, {
				name: "unknown",
			})

			const response = await server.handleMessage(req, createContext())

			expect(Rpc.isError(response!)).toBe(true)
		})
	})

	describe("unknown method", () => {
		test("returns method not found error", async () => {
			const req = Rpc.request(1, "unknown/method")
			const response = await server.handleMessage(req, createContext())

			expect(Rpc.isError(response!)).toBe(true)
			if (Rpc.isError(response!)) {
				expect(response.error.message).toContain("Unknown method")
			}
		})
	})
})
