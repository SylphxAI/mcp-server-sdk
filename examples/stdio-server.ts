/**
 * MCP Stdio Server Example
 *
 * Simple MCP server running over stdio (stdin/stdout).
 * Use this pattern for CLI tools and local MCP servers.
 *
 * Run with: bun run examples/stdio-server.ts
 * Test with: echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}' | bun run examples/stdio-server.ts
 */

import { object, str } from "@sylphx/vex"
import { createMcpApp, runStdio, text, tool } from "../src/index.js"

// ============================================================================
// Tools
// ============================================================================

const greet = tool()
	.description("Greet someone by name")
	.input(object({ name: str() }))
	.handler(({ input }) => text(`Hello, ${input.name}!`))

const echo = tool()
	.description("Echo a message back")
	.input(object({ message: str() }))
	.handler(({ input }) => text(input.message))

const add = tool()
	.description("Add two numbers")
	.input(object({ a: str(), b: str() }))
	.handler(({ input }) => {
		const a = Number.parseFloat(input.a)
		const b = Number.parseFloat(input.b)
		return text(`${a} + ${b} = ${a + b}`)
	})

// ============================================================================
// Create App and Run
// ============================================================================

const app = createMcpApp({
	name: "stdio-example-server",
	version: "1.0.0",
	instructions: "A simple MCP server running over stdio",
	tools: { greet, echo, add },
})

await runStdio({ app })
