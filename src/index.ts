/**
 * @sylphx/mcp-server-sdk
 *
 * Pure functional MCP server SDK for Node.js and Bun.
 *
 * @example
 * ```ts
 * import { createServer, tool, text, stdio } from '@sylphx/mcp-server-sdk'
 * import { z } from 'zod'
 *
 * const greet = tool()
 *   .description('Greet someone')
 *   .input(z.object({ name: z.string() }))
 *   .handler(({ input }) => text(`Hello ${input.name}`))
 *
 * const ping = tool()
 *   .handler(() => text('pong'))
 *
 * const server = createServer({
 *   tools: { greet, ping },
 *   transport: stdio()
 * })
 *
 * await server.start()
 * ```
 */

// ============================================================================
// Core API
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
// Server
export { createServer, type Server, type ServerConfig } from "./server/server.js"
export { type HttpOptions, http } from "./transports/http.js"
// Transports
export { type StdioOptions, stdio } from "./transports/stdio.js"
export type { Transport, TransportFactory } from "./transports/types.js"

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

// Elicitation (Server → Client user input requests)
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
// Sampling (Server → Client LLM requests)
export {
	createSamplingClient,
	type SamplingClient,
	type SamplingContext,
	type SamplingRequestSender,
} from "./sampling/index.js"
