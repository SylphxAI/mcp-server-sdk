/**
 * @sylphx/mcp-server-sdk
 *
 * Pure functional MCP server SDK for Bun.
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

// Server
export { createServer, type Server, type ServerConfig } from "./server/server.js"

// Transports
export { stdio, type StdioOptions } from "./transports/stdio.js"
export { http, type HttpOptions } from "./transports/http.js"
export type { Transport, TransportFactory } from "./transports/types.js"

// Tool Builder
export {
	tool,
	text,
	image,
	audio,
	embedded,
	json,
	toolError,
	type ToolResult,
	type ToolContext,
	type ToolDefinition,
	type ToolHandler,
	type ToolHandlerArgs,
} from "./builders/tool.js"

// Resource Builder
export {
	resource,
	resourceTemplate,
	resourceText,
	resourceBlob,
	resourceContents,
	type ResourceContext,
	type ResourceDefinition,
	type ResourceTemplateDefinition,
	type ResourceHandler,
	type TemplateHandler,
	type ResourceHandlerArgs,
	type TemplateHandlerArgs,
} from "./builders/resource.js"

// Prompt Builder
export {
	prompt,
	user,
	assistant,
	message,
	messages,
	promptResult,
	interpolate,
	type PromptContext,
	type PromptDefinition,
	type PromptHandler,
	type PromptHandlerArgs,
} from "./builders/prompt.js"

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

// Notifications
export {
	progress,
	log,
	resourcesListChanged,
	toolsListChanged,
	promptsListChanged,
	resourceUpdated,
	cancelled,
	type Notification,
	type ProgressNotification,
	type LogNotification,
	type NotificationEmitter,
	type NotificationSender,
} from "./notifications/index.js"

// Sampling (Server → Client LLM requests)
export {
	createSamplingClient,
	type SamplingClient,
	type SamplingRequestSender,
	type SamplingContext,
} from "./sampling/index.js"

// Elicitation (Server → Client user input requests)
export {
	createElicitationClient,
	type ElicitationClient,
	type ElicitationRequestSender,
	type ElicitationContext,
	type ElicitationSchema,
	type ElicitationProperty,
	type ElicitationCreateParams,
	type ElicitationCreateResult,
	type ElicitationAction,
} from "./elicitation/index.js"

// Pagination
export {
	paginate,
	type PaginationOptions,
	type PageResult,
} from "./pagination/index.js"
