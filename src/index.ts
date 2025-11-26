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

// Middleware
export {
	// Composition
	compose,
	createStack,
	when,
	forType,
	forName,
	// Built-in
	logging,
	timing,
	errorHandler,
	toolErrorHandler,
	timeout,
	retry,
	cache,
	// Types
	type Middleware,
	type MiddlewareStack,
	type RequestInfo,
	type Next,
	type ToolMiddleware,
	type AnyMiddleware,
	type LoggingOptions,
	type TimingContext,
	type ErrorHandlerOptions,
	type TimeoutOptions,
	type RetryOptions,
	type CacheOptions,
} from "./middleware/index.js"

// Notifications
export {
	// Emitter
	createEmitter,
	noopEmitter,
	// Helpers
	progress,
	log,
	createProgressReporter,
	createLogger,
	withProgress,
	resourcesListChanged,
	toolsListChanged,
	promptsListChanged,
	resourceUpdated,
	cancelled,
	// Types
	type Notification,
	type ProgressNotification,
	type LogNotification,
	type NotificationEmitter,
	type NotificationSender,
	type NotificationContext,
	type Logger,
} from "./notifications/index.js"

// Sampling (Server → Client LLM requests)
export {
	createSamplingClient,
	samplingText,
	samplingImage,
	modelPreferences,
	type SamplingClient,
	type SamplingRequestSender,
	type SamplingContext,
} from "./sampling/index.js"

// Completions (Auto-complete)
export {
	buildCompletionRegistry,
	handleComplete,
	staticCompletions,
	dynamicCompletions,
	mergeCompletions,
	type CompletionProvider,
	type CompletionResult,
	type CompletionConfig,
	type PromptCompletionConfig,
	type ResourceCompletionConfig,
	type CompletionRegistry,
} from "./completions/index.js"

// Subscriptions (Resource subscriptions)
export {
	createSubscriptionManager,
	notifySubscribers,
	type SubscriptionManager,
	type SubscriptionEvent,
	type SubscriptionEventHandler,
} from "./subscriptions/index.js"

// Pagination
export {
	paginate,
	encodeCursor,
	decodeCursor,
	createPaginatedHandler,
	iteratePages,
	collectAllPages,
	type PaginationOptions,
	type PageResult,
	type CursorData,
} from "./pagination/index.js"

// Elicitation (Server → Client user input requests)
export {
	createElicitationClient,
	elicitString,
	elicitNumber,
	elicitInteger,
	elicitBoolean,
	elicitEnum,
	elicitSchema,
	type ElicitationClient,
	type ElicitationRequestSender,
	type ElicitationContext,
	type ElicitationSchema,
	type ElicitationProperty,
	type ElicitationCreateParams,
	type ElicitationCreateResult,
	type ElicitationAction,
} from "./elicitation/index.js"
