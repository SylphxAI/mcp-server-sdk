/**
 * MCP Conformance Test Server
 *
 * Implements all tools, resources, and prompts required by
 * @modelcontextprotocol/conformance test suite.
 *
 * Run with: bun run examples/conformance-server.ts
 */

import { z } from "zod"
import {
	audio,
	createServer,
	embedded,
	http,
	image,
	messages,
	prompt,
	resource,
	resourceBlob,
	resourceTemplate,
	resourceText,
	text,
	tool,
	toolError,
	user,
} from "../src/index.js"

// ============================================================================
// Test Data (base64 encoded)
// ============================================================================

// Minimal valid PNG (1x1 red pixel)
const TEST_PNG_BASE64 =
	"iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8DwHwAFBQIAX8jx0gAAAABJRU5ErkJggg=="

// Minimal valid WAV (silence)
const TEST_WAV_BASE64 = "UklGRiQAAABXQVZFZm10IBAAAAABAAEARKwAAIhYAQACABAAZGF0YQAAAAA="

// ============================================================================
// Tools
// ============================================================================

const test_simple_text = tool()
	.description("Returns a simple text response for testing")
	.handler(() => text("This is a simple text response for testing."))

const test_image_content = tool()
	.description("Returns image content for testing")
	.handler(() => image(TEST_PNG_BASE64, "image/png"))

const test_audio_content = tool()
	.description("Returns audio content for testing")
	.handler(() => audio(TEST_WAV_BASE64, "audio/wav"))

const test_multiple_content_types = tool()
	.description("Returns multiple content types for testing")
	.handler(() => [
		text("This is text content."),
		image(TEST_PNG_BASE64, "image/png"),
		embedded({
			type: "resource",
			uri: "test://embedded",
			mimeType: "text/plain",
			text: "Embedded resource content.",
		}),
	])

const test_error_handling = tool()
	.description("Returns an error for testing error handling")
	.handler(() => toolError("This is a test error message."))

const test_embedded_resource = tool()
	.description("Returns embedded resource content for testing")
	.handler(() =>
		embedded({
			type: "resource",
			uri: "test://embedded-resource",
			mimeType: "text/plain",
			text: "This is embedded resource content for testing.",
		})
	)

// Tools that require notification support
const test_tool_with_logging = tool()
	.description("Returns text with logging notifications")
	.handler(({ ctx }) => {
		ctx.log("info", "Starting tool execution")
		ctx.log("debug", { step: 1, message: "Processing" })
		ctx.log("info", "Tool execution complete")
		return text("Tool completed with logging.")
	})

const test_tool_with_progress = tool()
	.description("Returns text with progress notifications")
	.handler(({ ctx }) => {
		// Uses the progressToken from the request automatically
		ctx.progress(0, { total: 100, message: "Starting" })
		ctx.progress(50, { total: 100, message: "Halfway" })
		ctx.progress(100, { total: 100, message: "Complete" })
		return text("Tool completed with progress tracking.")
	})

// Tools that require client capabilities
const test_sampling = tool()
	.description("Tests sampling capability")
	.input(z.object({ prompt: z.string() }))
	.handler(async ({ input, ctx }) => {
		// Check if sampling is available
		if (!ctx.sampling) {
			return text("Sampling not available - client does not support sampling capability")
		}

		// Request sampling from client
		const result = await ctx.sampling.createMessage({
			messages: [{ role: "user", content: { type: "text", text: input.prompt } }],
			maxTokens: 100,
		})

		return text(
			`Sampling result: ${result.content.type === "text" ? result.content.text : "non-text response"}`
		)
	})

const test_elicitation = tool()
	.description("Tests elicitation capability")
	.input(z.object({ message: z.string() }))
	.handler(async ({ input, ctx }) => {
		// Check if elicitation is available
		if (!ctx.elicit) {
			return text("Elicitation not available - client does not support elicitation capability")
		}

		// Request elicitation from client
		const result = await ctx.elicit(input.message, {
			type: "object",
			properties: {
				response: { type: "string", description: "User response" },
			},
			required: ["response"],
		})

		if (result.action === "accept") {
			return text(`Elicitation accepted: ${JSON.stringify(result.content)}`)
		}
		return text(`Elicitation ${result.action}`)
	})

// SEP-1034: Elicitation with default values for all primitive types
const test_elicitation_sep1034_defaults = tool()
	.description("Tests elicitation with default values for all primitive types (SEP-1034)")
	.handler(async ({ ctx }) => {
		if (!ctx.elicit) {
			return text("Elicitation not available - client does not support elicitation capability")
		}

		const result = await ctx.elicit("Please provide your information", {
			type: "object",
			properties: {
				name: { type: "string", description: "Your name", default: "John Doe" },
				age: { type: "integer", description: "Your age", default: 30 },
				score: { type: "number", description: "Your score", default: 95.5 },
				status: {
					type: "string",
					description: "Your status",
					enum: ["active", "inactive", "pending"],
					default: "active",
				},
				verified: { type: "boolean", description: "Whether verified", default: true },
			},
			required: ["name", "age", "score", "status", "verified"],
		})

		return text(
			`Elicitation completed: action=${result.action}, content=${JSON.stringify(result.content)}`
		)
	})

// ============================================================================
// Resources
// ============================================================================

const staticText = resource()
	.uri("test://static-text")
	.description("Static Text Resource - A static text resource for testing")
	.mimeType("text/plain")
	.handler(({ uri }) =>
		resourceText(uri, "This is the content of the static text resource.", "text/plain")
	)

const staticBinary = resource()
	.uri("test://static-binary")
	.description("Static Binary Resource - A static binary resource for testing")
	.mimeType("image/png")
	.handler(({ uri }) => resourceBlob(uri, TEST_PNG_BASE64, "image/png"))

const watchedResource = resource()
	.uri("test://watched-resource")
	.description("Watched Resource - A resource that can be subscribed to")
	.mimeType("text/plain")
	.handler(({ uri }) => resourceText(uri, "Watched resource content.", "text/plain"))

const templateResource = resourceTemplate()
	.uriTemplate("test://template/{id}/data")
	.description("Template Resource - A parameterized resource template for testing")
	.mimeType("application/json")
	.handler(({ uri, params }) => {
		const data = {
			id: params.id,
			templateTest: true,
			data: `Data for ID: ${params.id}`,
		}
		return resourceText(uri, JSON.stringify(data), "application/json")
	})

// ============================================================================
// Prompts
// ============================================================================

const test_simple_prompt = prompt()
	.description("A simple prompt for testing")
	.handler(() => messages(user("This is a simple prompt for testing.")))

const test_prompt_with_arguments = prompt()
	.description("A prompt with arguments for testing")
	.args(
		z.object({
			arg1: z.string().describe("First argument"),
			arg2: z.string().describe("Second argument"),
		})
	)
	.handler(({ args }) =>
		messages(user(`Prompt with arguments: arg1=${args.arg1}, arg2=${args.arg2}`))
	)

const test_prompt_with_embedded_resource = prompt()
	.description("A prompt with an embedded resource")
	.args(z.object({ resourceUri: z.string().describe("URI of resource to embed") }))
	.handler(({ args }) => ({
		messages: [
			{
				role: "user" as const,
				content: {
					type: "resource" as const,
					resource: {
						type: "resource" as const,
						uri: args.resourceUri,
						mimeType: "text/plain",
						text: "Embedded resource content for testing.",
					},
				},
			},
			{
				role: "user" as const,
				content: {
					type: "text" as const,
					text: "Please process the embedded resource above.",
				},
			},
		],
	}))

const test_prompt_with_image = prompt()
	.description("A prompt containing an image")
	.handler(() => ({
		messages: [
			{
				role: "user" as const,
				content: {
					type: "image" as const,
					data: TEST_PNG_BASE64,
					mimeType: "image/png",
				},
			},
			{
				role: "user" as const,
				content: {
					type: "text" as const,
					text: "Please analyze this test image.",
				},
			},
		],
	}))

// ============================================================================
// Create Server
// ============================================================================

const port = Number(process.env.PORT) || 3456

const server = createServer({
	name: "mcp-conformance-test-server",
	version: "1.0.0",
	instructions: "MCP Conformance Test Server implementing all required test scenarios",
	tools: {
		test_simple_text,
		test_image_content,
		test_audio_content,
		test_multiple_content_types,
		test_error_handling,
		test_embedded_resource,
		test_tool_with_logging,
		test_tool_with_progress,
		test_sampling,
		test_elicitation,
		test_elicitation_sep1034_defaults,
	},
	resources: {
		staticText,
		staticBinary,
		watchedResource,
	},
	resourceTemplates: {
		templateResource,
	},
	prompts: {
		test_simple_prompt,
		test_prompt_with_arguments,
		test_prompt_with_embedded_resource,
		test_prompt_with_image,
	},
	transport: http({ port, cors: "*" }),
})

await server.start()
console.log(`MCP Conformance Test Server running at http://localhost:${port}/mcp`)
