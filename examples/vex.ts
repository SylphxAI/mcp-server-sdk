/**
 * Vex Schema Example - @sylphx/mcp-server-sdk
 *
 * Demonstrates type-safe tool and prompt definitions using Vex.
 * Run with: bun run examples/vex.ts
 */

import {
	bool,
	coerceNumber,
	description,
	enum_,
	int,
	num,
	object,
	optional,
	pipe,
	positive,
	str,
	withDefault,
} from "@sylphx/vex"
import { createServer, messages, prompt, stdio, text, tool, toolError, user } from "../src/index.js"

// ============================================================================
// Define Tools with Vex (Full Type Safety)
// ============================================================================

const calculator = tool()
	.description("Perform basic arithmetic")
	.input(
		object({
			operation: enum_(["add", "subtract", "multiply", "divide"] as const),
			a: num(description("First number")),
			b: num(description("Second number")),
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
		object({
			id: pipe(coerceNumber, int, positive, description("User ID")),
			includeEmail: withDefault(bool(description("Include email in response")), false),
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
// Define Prompts with Vex
// ============================================================================

const codeReview = prompt()
	.description("Generate a code review prompt")
	.args(
		object({
			language: str(description("Programming language")),
			focus: optional(str(description("Specific area to focus on"))),
			strict: withDefault(bool(description("Use strict review criteria")), false),
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
	name: "vex-example",
	version: "1.0.0",
	instructions: "Example server demonstrating Vex schema validation",
	tools: { calculator, fetchUser },
	prompts: { codeReview },
	transport: stdio(),
})

await server.start()
