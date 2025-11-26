/**
 * Zod Schema Example - @sylphx/mcp-server-sdk
 *
 * Demonstrates type-safe tool and prompt definitions using Zod.
 * Run with: bun run examples/zod.ts
 */

import { z } from "zod"
import { createServer, messages, prompt, stdio, text, tool, toolError, user } from "../src/index.js"

// ============================================================================
// Define Tools with Zod (Full Type Safety)
// ============================================================================

const calculator = tool()
	.description("Perform basic arithmetic")
	.input(
		z.object({
			operation: z.enum(["add", "subtract", "multiply", "divide"]).describe("Math operation"),
			a: z.number().describe("First number"),
			b: z.number().describe("Second number"),
		})
	)
	.handler(({ input }) => {
		const { operation, a, b } = input
		let result: number

		switch (operation) {
			case "add":
				result = a + b
				break
			case "subtract":
				result = a - b
				break
			case "multiply":
				result = a * b
				break
			case "divide":
				if (b === 0) return toolError("Cannot divide by zero")
				result = a / b
				break
		}

		return text(`${a} ${operation} ${b} = ${result}`)
	})

const fetchUser = tool()
	.description("Fetch user information")
	.input(
		z.object({
			id: z.coerce.number().int().positive().describe("User ID"),
			includeEmail: z.boolean().default(false).describe("Include email in response"),
		})
	)
	.handler(({ input }) => {
		const { id, includeEmail } = input
		// Input is already validated and typed!
		// - id is guaranteed to be a positive integer
		// - includeEmail has a default value
		const user = {
			id,
			name: `User ${id}`,
			...(includeEmail && { email: `user${id}@example.com` }),
		}
		return text(JSON.stringify(user, null, 2))
	})

// ============================================================================
// Define Prompts with Zod
// ============================================================================

const codeReview = prompt()
	.description("Generate a code review prompt")
	.args(
		z.object({
			language: z.string().describe("Programming language"),
			focus: z.string().optional().describe("Specific area to focus on"),
			strict: z.boolean().default(false).describe("Use strict review criteria"),
		})
	)
	.handler(({ args }) => {
		const { language, focus, strict } = args
		return messages(
			user(
				`Review this ${language} code${focus ? ` with focus on ${focus}` : ""}. ` +
					`${strict ? "Apply strict standards and flag all issues." : "Provide constructive feedback."}`
			)
		)
	})

// ============================================================================
// Create Server
// ============================================================================

const server = createServer({
	name: "zod-example",
	version: "1.0.0",
	instructions: "Example server demonstrating Zod schema validation",
	tools: { calculator, fetchUser },
	prompts: { codeReview },
	transport: stdio(),
})

await server.start()
