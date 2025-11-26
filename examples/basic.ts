/**
 * Basic Example - @sylphx/mcp-server
 *
 * Run with: bun run examples/basic.ts
 */

import {
	http,
	createContext,
	createServer,
	prompt,
	resource,
	resourceText,
	stdio,
	text,
	tool,
	user,
} from "../src/index.js"

// ============================================================================
// Define Tools (Pure Functions)
// ============================================================================

const greet = tool({
	name: "greet",
	description: "Greet someone by name",
	input: {
		type: "object",
		properties: {
			name: { type: "string", description: "Name to greet" },
		},
		required: ["name"],
	},
	// Handler is a pure function: input -> context -> result
	handler:
		({ name }: { name: string }) =>
		() =>
			text(`Hello, ${name}!`),
})

const add = tool({
	name: "add",
	description: "Add two numbers",
	input: {
		type: "object",
		properties: {
			a: { type: "number" },
			b: { type: "number" },
		},
		required: ["a", "b"],
	},
	handler:
		({ a, b }: { a: number; b: number }) =>
		() =>
			text(`${a} + ${b} = ${a + b}`),
})

// ============================================================================
// Define Resources
// ============================================================================

const configResource = resource({
	uri: "config://app",
	name: "Application Config",
	description: "Current application configuration",
	mimeType: "application/json",
	handler: () => () =>
		resourceText(
			"config://app",
			JSON.stringify({ version: "1.0.0", debug: false }, null, 2),
			"application/json",
		),
})

// ============================================================================
// Define Prompts
// ============================================================================

const reviewPrompt = prompt({
	name: "code_review",
	description: "Generate a code review prompt",
	arguments: [
		{ name: "language", required: true },
		{ name: "focus", description: "What to focus on" },
	],
	handler:
		({ language, focus }) =>
		() => ({
			messages: [
				user(
					`Please review this ${language} code${focus ? ` focusing on ${focus}` : ""}. Look for bugs, style issues, and potential improvements.`,
				),
			],
		}),
})

// ============================================================================
// Create Server
// ============================================================================

const server = createServer({
	name: "example-server",
	version: "1.0.0",
	instructions: "A simple example MCP server demonstrating @sylphx/mcp-server",
	tools: [greet, add],
	resources: [configResource],
	prompts: [reviewPrompt],
})

// ============================================================================
// Choose Transport
// ============================================================================

const mode = process.argv[2] ?? "stdio"

if (mode === "http") {
	// HTTP transport with Bun.serve
	const transport = http(server, {
		port: 3000,
		cors: "*",
	})

	await transport.start()
	console.log(`MCP server running at ${transport.url}`)
	console.log(`Health check: ${transport.url}/health`)
} else {
	// Stdio transport (default)
	const transport = stdio(server)
	await transport.start()
}
