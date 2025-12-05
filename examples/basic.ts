/**
 * Basic Example - @sylphx/mcp-server-sdk
 *
 * Run with: bun run examples/basic.ts
 */

import { description, num, object, optional, str } from "@sylphx/vex"
import { createServer, http, messages, prompt, resource, resourceText, stdio, text, tool, user } from "../src/index.js"

// ============================================================================
// Define Tools (Builder Pattern)
// ============================================================================

const greet = tool()
	.description("Greet someone by name")
	.input(object({ name: str(description("Name to greet")) }))
	.handler(({ input }) => text(`Hello, ${input.name}!`))

const add = tool()
	.description("Add two numbers")
	.input(object({ a: num(), b: num() }))
	.handler(({ input }) => text(`${input.a} + ${input.b} = ${input.a + input.b}`))

const ping = tool()
	.description("Health check")
	.handler(() => text("pong"))

// ============================================================================
// Define Resources
// ============================================================================

const config = resource()
	.uri("config://app")
	.description("Current application configuration")
	.mimeType("application/json")
	.handler(({ uri }) =>
		resourceText(uri, JSON.stringify({ version: "1.0.0", debug: false }, null, 2), "application/json"),
	)

// ============================================================================
// Define Prompts
// ============================================================================

const codeReview = prompt()
	.description("Generate a code review prompt")
	.args(
		object({
			language: str(description("Programming language")),
			focus: optional(str(description("What to focus on"))),
		}),
	)
	.handler(({ args }) =>
		messages(
			user(
				`Please review this ${args.language} code${args.focus ? ` focusing on ${args.focus}` : ""}. Look for bugs, style issues, and potential improvements.`,
			),
		),
	)

// ============================================================================
// Create Server
// ============================================================================

const mode = process.argv[2] ?? "stdio"

if (mode === "http") {
	const server = createServer({
		name: "example-server",
		version: "1.0.0",
		instructions: "A simple example MCP server demonstrating @sylphx/mcp-server-sdk",
		tools: { greet, add, ping },
		resources: { config },
		prompts: { codeReview },
		transport: http({ port: 3000 }),
	})

	await server.start()
	console.log("MCP server running at http://localhost:3000")
} else {
	const server = createServer({
		name: "example-server",
		version: "1.0.0",
		instructions: "A simple example MCP server demonstrating @sylphx/mcp-server-sdk",
		tools: { greet, add, ping },
		resources: { config },
		prompts: { codeReview },
		transport: stdio(),
	})

	await server.start()
}
