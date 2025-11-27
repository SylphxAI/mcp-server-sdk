/**
 * Benchmarks for @sylphx/mcp-server-sdk
 *
 * Run with: bun run bench
 */

import { bench, group, run } from "mitata"
import { z } from "zod"
import { messages, prompt, user } from "../src/builders/prompt.js"
import { resource, resourceText } from "../src/builders/resource.js"
import { text, tool } from "../src/builders/tool.js"
import * as Rpc from "../src/protocol/jsonrpc.js"
import * as Mcp from "../src/protocol/mcp.js"
import { dispatch, type ServerState } from "../src/server/handler.js"

// =============================================================================
// Setup
// =============================================================================

const greetTool = tool()
	.description("Greet someone")
	.input(z.object({ name: z.string() }))
	.handler(({ input }) => text(`Hello, ${input.name}!`))

const asyncTool = tool()
	.description("Async operation")
	.handler(async () => {
		await Promise.resolve()
		return text("done")
	})

const configResource = resource()
	.uri("config://app")
	.handler(({ uri }) => resourceText(uri, '{"version":"1.0"}'))

const greetPrompt = prompt()
	.args(z.object({ name: z.string() }))
	.handler(({ args }) => messages(user(`Hello ${args.name}`)))

// Create server state directly for benchmarking
const state: ServerState = {
	name: "bench-server",
	version: "1.0.0",
	tools: new Map([
		["greet", greetTool],
		["async_op", asyncTool],
	]),
	resources: new Map([["config", configResource]]),
	resourceTemplates: new Map(),
	prompts: new Map([["greet", greetPrompt]]),
	capabilities: {
		tools: { listChanged: true },
		resources: { subscribe: false, listChanged: true },
		prompts: { listChanged: true },
	},
}

const ctx = { signal: undefined }

// Pre-built requests
const initializeReq = Rpc.request(1, Mcp.Method.Initialize, {
	protocolVersion: Mcp.LATEST_PROTOCOL_VERSION,
	capabilities: {},
	clientInfo: { name: "bench", version: "1.0.0" },
})

const pingReq = Rpc.request(1, Mcp.Method.Ping)
const toolsListReq = Rpc.request(1, Mcp.Method.ToolsList)

const toolsCallReq = Rpc.request(1, Mcp.Method.ToolsCall, {
	name: "greet",
	arguments: { name: "World" },
})

const asyncToolCallReq = Rpc.request(1, Mcp.Method.ToolsCall, {
	name: "async_op",
	arguments: {},
})

const resourcesListReq = Rpc.request(1, Mcp.Method.ResourcesList)

const resourcesReadReq = Rpc.request(1, Mcp.Method.ResourcesRead, {
	uri: "config://app",
})

const promptsListReq = Rpc.request(1, Mcp.Method.PromptsList)

const promptsGetReq = Rpc.request(1, Mcp.Method.PromptsGet, {
	name: "greet",
	arguments: { name: "User" },
})

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
		await dispatch(state, initializeReq, ctx)
	})

	bench("ping", async () => {
		await dispatch(state, pingReq, ctx)
	})
})

group("Tools", () => {
	bench("tools/list", async () => {
		await dispatch(state, toolsListReq, ctx)
	})

	bench("tools/call (sync)", async () => {
		await dispatch(state, toolsCallReq, ctx)
	})

	bench("tools/call (async)", async () => {
		await dispatch(state, asyncToolCallReq, ctx)
	})
})

group("Resources", () => {
	bench("resources/list", async () => {
		await dispatch(state, resourcesListReq, ctx)
	})

	bench("resources/read", async () => {
		await dispatch(state, resourcesReadReq, ctx)
	})
})

group("Prompts", () => {
	bench("prompts/list", async () => {
		await dispatch(state, promptsListReq, ctx)
	})

	bench("prompts/get", async () => {
		await dispatch(state, promptsGetReq, ctx)
	})
})

group("Full Request Cycle", () => {
	bench("ping (full cycle)", async () => {
		const result = await dispatch(state, pingReq, ctx)
		if (result.type === "response") Rpc.stringify(result.response)
	})

	bench("tools/call (full cycle)", async () => {
		const result = await dispatch(state, toolsCallReq, ctx)
		if (result.type === "response") Rpc.stringify(result.response)
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
