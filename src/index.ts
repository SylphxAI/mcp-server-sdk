/**
 * @sylphx/mcp-server-sdk
 *
 * Pure functional MCP server SDK for Node.js and Bun.
 *
 * @example
 * ```ts
 * import { createMcpApp, serve, tool, text } from '@sylphx/mcp-server-sdk'
 * import { object, str } from '@sylphx/vex'
 *
 * const greet = tool()
 *   .description('Greet someone')
 *   .input(object({ name: str() }))
 *   .handler(({ input }) => text(`Hello ${input.name}`))
 *
 * const app = createMcpApp({
 *   name: 'my-server',
 *   tools: { greet },
 * })
 *
 * // Option 1: Use with gust serve
 * await serve({ app, port: 3000 })
 *
 * // Option 2: Use with Bun.serve
 * Bun.serve({ fetch: app.fetch, port: 3000 })
 *
 * // Option 3: Direct JSON-RPC handling
 * const result = await app.handle(jsonRpcMessage)
 * ```
 */

// ============================================================================
// Core API - New Architecture
// ============================================================================

// MCP App
export {
	createMcpApp,
	type McpApp,
	type McpAppConfig,
	type McpServer,
	type ServeOptions,
	serve,
} from "./app/index.js"

// ============================================================================
// Builders
// ============================================================================

// Prompt Builder
export {
	assistant,
	interpolate,
	message,
	messages,
	type PromptContext,
	type PromptDefinition,
	type PromptHandler,
	type PromptHandlerArgs,
	prompt,
	promptResult,
	user,
} from "./builders/prompt.js"

// Resource Builder
export {
	type ResourceContext,
	type ResourceDefinition,
	type ResourceHandler,
	type ResourceHandlerArgs,
	type ResourceTemplateDefinition,
	resource,
	resourceBlob,
	resourceContents,
	resourceTemplate,
	resourceText,
	type TemplateHandler,
	type TemplateHandlerArgs,
} from "./builders/resource.js"

// Tool Builder
export {
	audio,
	embedded,
	image,
	json,
	type ToolContext,
	type ToolDefinition,
	type ToolHandler,
	type ToolHandlerArgs,
	type ToolResult,
	text,
	tool,
	toolError,
} from "./builders/tool.js"

// ============================================================================
// Schema
// ============================================================================

export type { Infer } from "./schema/index.js"

// ============================================================================
// Protocol Types
// ============================================================================

export * from "./protocol/index.js"

// ============================================================================
// Advanced Features
// ============================================================================

// Elicitation (Server -> Client user input requests)
export {
	createElicitationClient,
	type ElicitationAction,
	type ElicitationClient,
	type ElicitationContext,
	type ElicitationCreateParams,
	type ElicitationCreateResult,
	type ElicitationProperty,
	type ElicitationRequestSender,
	type ElicitationSchema,
} from "./elicitation/index.js"

// Notifications
export {
	cancelled,
	type LogNotification,
	log,
	type Notification,
	type NotificationEmitter,
	type NotificationSender,
	type ProgressNotification,
	progress,
	promptsListChanged,
	resourcesListChanged,
	resourceUpdated,
	toolsListChanged,
} from "./notifications/index.js"

// Pagination
export {
	type PageResult,
	type PaginationOptions,
	paginate,
} from "./pagination/index.js"

// Sampling (Server -> Client LLM requests)
export {
	createSamplingClient,
	type SamplingClient,
	type SamplingContext,
	type SamplingRequestSender,
} from "./sampling/index.js"

// ============================================================================
// Legacy API (Deprecated)
// ============================================================================

/**
 * @deprecated Use createMcpApp + serve instead.
 *
 * Migration:
 * ```ts
 * // Old
 * const server = createServer({ tools, transport: http({ port: 3000 }) })
 * await server.start()
 *
 * // New
 * const app = createMcpApp({ tools })
 * await serve({ app, port: 3000 })
 * ```
 */
export { createServer, type Server, type ServerConfig } from "./server/server.js"

/**
 * @deprecated Use serve({ app }) instead.
 */
export { type HttpOptions, http } from "./transports/http.js"

/**
 * @deprecated Use stdio transport with createServer for backward compatibility.
 */
export { type StdioOptions, stdio } from "./transports/stdio.js"

export type { Transport, TransportFactory } from "./transports/types.js"
