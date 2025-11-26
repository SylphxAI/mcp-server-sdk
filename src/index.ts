/**
 * @sylphx/mcp-server
 *
 * Pure functional MCP server library for Bun.
 *
 * @example
 * ```ts
 * import { createServer, tool, text, stdio } from "@sylphx/mcp-server"
 *
 * const greet = tool({
 *   name: "greet",
 *   description: "Greet someone",
 *   input: {
 *     type: "object",
 *     properties: { name: { type: "string" } },
 *     required: ["name"],
 *   },
 *   handler: ({ name }) => () => text(`Hello, ${name}!`),
 * })
 *
 * const server = createServer({
 *   name: "my-server",
 *   version: "1.0.0",
 *   tools: [greet],
 * })
 *
 * const transport = stdio(server)
 * await transport.start()
 * ```
 */

// Protocol types
export * from "./protocol/index.js"

// Builders
export {
	// Tool
	tool,
	defineTool,
	createTool,
	text,
	textContent,
	contents,
	toolError,
	structured,
	sequence,
	guard,
	mapResult,
	toProtocolTool,
	type ToolConfig,
	type ToolContext,
	type ToolDefinition,
	type ToolHandler,
	type TypedToolConfig,
	type TypedToolDefinition,
	// Resource
	resource,
	resourceTemplate,
	resourceText,
	resourceBlob,
	resourceContents,
	matchesTemplate,
	extractParams,
	toProtocolResource,
	toProtocolTemplate,
	type ResourceConfig,
	type ResourceContext,
	type ResourceDefinition,
	type ResourceHandler,
	type ResourceTemplateConfig,
	type ResourceTemplateDefinition,
	type AnyResourceDefinition,
	// Prompt
	prompt,
	definePrompt,
	arg,
	user,
	assistant,
	message,
	messages,
	promptResult,
	interpolate,
	templatePrompt,
	toProtocolPrompt,
	type PromptConfig,
	type PromptContext,
	type PromptDefinition,
	type PromptHandler,
	type PromptArgumentConfig,
	type TypedPromptConfig,
	type TypedPromptDefinition,
	type TypedPromptHandler,
} from "./builders/index.js"

// Schema utilities
export {
	zodToJsonSchema,
	toJsonSchema,
	isZodSchema,
	validate,
	extractObjectFields,
	type Infer,
	type SchemaInput,
	type ValidationResult,
} from "./schema/index.js"

// Server
export {
	createServer,
	createContext,
	type Server,
	type ServerConfig,
	type ServerState,
	type HandlerContext,
	type HandlerResult,
} from "./server/index.js"

// Transports
export { stdio, type StdioTransport, type StdioOptions } from "./transports/stdio.js"
export { http, type HttpTransport, type HttpOptions } from "./transports/http.js"
