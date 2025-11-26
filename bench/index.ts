/**
 * Benchmarks for @sylphx/mcp-server
 *
 * Run with: bun run bench
 */

import { bench, group, run } from "mitata"
import { prompt, user } from "../src/builders/prompt.js"
import { resource, resourceText } from "../src/builders/resource.js"
import { text, tool } from "../src/builders/tool.js"
import * as Rpc from "../src/protocol/jsonrpc.js"
import * as Mcp from "../src/protocol/mcp.js"
import { createContext, createServer } from "../src/server/server.js"

// =============================================================================
// Setup
// =============================================================================

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

const asyncTool = tool({
	name: "async_op",
	description: "Async operation",
	input: { type: "object" },
	handler: () => async () => {
		// Simulate minimal async work
		await Promise.resolve()
		return text("done")
	},
})

const configResource = resource({
	uri: "config://app",
	name: "Config",
	handler: () => () => resourceText("config://app", '{"version":"1.0"}'),
})

const greetPrompt = prompt({
	name: "greet",
	handler:
		({ name }: Record<string, string>) =>
		() => ({
			messages: [user(`Hello ${name}`)],
		}),
})

const server = createServer({
	name: "bench-server",
	version: "1.0.0",
	tools: [greetTool, asyncTool],
	resources: [configResource],
	prompts: [greetPrompt],
})

const ctx = createContext()

// Pre-built requests
const initializeReq = Rpc.stringify(
	Rpc.request(1, Mcp.Method.Initialize, {
		protocolVersion: Mcp.LATEST_PROTOCOL_VERSION,
		capabilities: {},
		clientInfo: { name: "bench", version: "1.0.0" },
	}),
)

const pingReq = Rpc.stringify(Rpc.request(1, Mcp.Method.Ping))

const toolsListReq = Rpc.stringify(Rpc.request(1, Mcp.Method.ToolsList))

const toolsCallReq = Rpc.stringify(
	Rpc.request(1, Mcp.Method.ToolsCall, {
		name: "greet",
		arguments: { name: "World" },
	}),
)

const asyncToolCallReq = Rpc.stringify(
	Rpc.request(1, Mcp.Method.ToolsCall, {
		name: "async_op",
		arguments: {},
	}),
)

const resourcesListReq = Rpc.stringify(Rpc.request(1, Mcp.Method.ResourcesList))

const resourcesReadReq = Rpc.stringify(
	Rpc.request(1, Mcp.Method.ResourcesRead, { uri: "config://app" }),
)

const promptsListReq = Rpc.stringify(Rpc.request(1, Mcp.Method.PromptsList))

const promptsGetReq = Rpc.stringify(
	Rpc.request(1, Mcp.Method.PromptsGet, {
		name: "greet",
		arguments: { name: "User" },
	}),
)

// =============================================================================
// Benchmarks
// =============================================================================

group("JSON-RPC Parsing", () => {
	const validJson = '{"jsonrpc":"2.0","id":1,"method":"ping"}'
	const invalidJson = "{invalid}"

	bench("parseMessage (valid)", () => {
		Rpc.parseMessage(validJson)
	})

	bench("parseMessage (invalid)", () => {
		Rpc.parseMessage(invalidJson)
	})

	bench("stringify", () => {
		Rpc.stringify(Rpc.request(1, "test", { foo: "bar" }))
	})
})

group("Server Lifecycle", () => {
	bench("initialize", async () => {
		await server.handle(initializeReq, ctx)
	})

	bench("ping", async () => {
		await server.handle(pingReq, ctx)
	})
})

group("Tools", () => {
	bench("tools/list", async () => {
		await server.handle(toolsListReq, ctx)
	})

	bench("tools/call (sync)", async () => {
		await server.handle(toolsCallReq, ctx)
	})

	bench("tools/call (async)", async () => {
		await server.handle(asyncToolCallReq, ctx)
	})
})

group("Resources", () => {
	bench("resources/list", async () => {
		await server.handle(resourcesListReq, ctx)
	})

	bench("resources/read", async () => {
		await server.handle(resourcesReadReq, ctx)
	})
})

group("Prompts", () => {
	bench("prompts/list", async () => {
		await server.handle(promptsListReq, ctx)
	})

	bench("prompts/get", async () => {
		await server.handle(promptsGetReq, ctx)
	})
})

group("Full Request Cycle", () => {
	// Measure complete request/response including parsing and serialization
	bench("ping (full cycle)", async () => {
		const response = await server.handle(pingReq, ctx)
		if (response) JSON.parse(response)
	})

	bench("tools/call (full cycle)", async () => {
		const response = await server.handle(toolsCallReq, ctx)
		if (response) JSON.parse(response)
	})
})

// Run benchmarks
await run({
	avg: true,
	json: false,
	colors: true,
	min_max: true,
	percentiles: true,
})
