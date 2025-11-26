/**
 * Model Context Protocol Types
 * Based on MCP specification 2025-03-26
 */

// ============================================================================
// Protocol Version
// ============================================================================

export const LATEST_PROTOCOL_VERSION = "2025-03-26" as const
export const SUPPORTED_PROTOCOL_VERSIONS: readonly ["2025-03-26", "2024-11-05"] = [
	LATEST_PROTOCOL_VERSION,
	"2024-11-05",
] as const

// ============================================================================
// Content Types
// ============================================================================

export interface TextContent {
	readonly type: "text"
	readonly text: string
	readonly annotations?: ContentAnnotations
}

export interface ImageContent {
	readonly type: "image"
	readonly data: string // base64
	readonly mimeType: string
	readonly annotations?: ContentAnnotations
}

export interface AudioContent {
	readonly type: "audio"
	readonly data: string // base64
	readonly mimeType: string
	readonly annotations?: ContentAnnotations
}

export interface ResourceContent {
	readonly type: "resource"
	readonly resource: EmbeddedResource
	readonly annotations?: ContentAnnotations
}

export type Content = TextContent | ImageContent | AudioContent | ResourceContent

export interface ContentAnnotations {
	readonly audience?: Array<"user" | "assistant">
	readonly priority?: number
}

// ============================================================================
// Resources
// ============================================================================

export interface Resource {
	readonly uri: string
	readonly name: string
	readonly description?: string
	readonly mimeType?: string
	readonly size?: number
}

export interface ResourceTemplate {
	readonly uriTemplate: string
	readonly name: string
	readonly description?: string
	readonly mimeType?: string
}

export interface EmbeddedResource {
	readonly type: "resource"
	readonly uri: string
	readonly mimeType?: string
	readonly text?: string
	readonly blob?: string // base64
}

// ============================================================================
// Tools
// ============================================================================

export interface Tool {
	readonly name: string
	readonly description?: string
	readonly inputSchema: JsonSchema
	readonly annotations?: ToolAnnotations
}

export interface ToolAnnotations {
	readonly title?: string
	readonly readOnlyHint?: boolean
	readonly destructiveHint?: boolean
	readonly idempotentHint?: boolean
	readonly openWorldHint?: boolean
}

export interface JsonSchema {
	readonly type?: string
	readonly properties?: Record<string, JsonSchema>
	readonly required?: readonly string[]
	readonly items?: JsonSchema
	readonly enum?: readonly unknown[]
	readonly description?: string
	readonly default?: unknown
	readonly [key: string]: unknown
}

// ============================================================================
// Prompts
// ============================================================================

export interface Prompt {
	readonly name: string
	readonly description?: string
	readonly arguments?: readonly PromptArgument[]
}

export interface PromptArgument {
	readonly name: string
	readonly description?: string
	readonly required?: boolean
}

export interface PromptMessage {
	readonly role: "user" | "assistant"
	readonly content: Content
}

// ============================================================================
// Capabilities
// ============================================================================

export interface ServerCapabilities {
	readonly experimental?: Record<string, unknown>
	readonly logging?: Record<string, never>
	readonly completions?: Record<string, never>
	readonly prompts?: { readonly listChanged?: boolean }
	readonly resources?: {
		readonly subscribe?: boolean
		readonly listChanged?: boolean
	}
	readonly tools?: {
		readonly listChanged?: boolean
	}
}

export interface ClientCapabilities {
	readonly experimental?: Record<string, unknown>
	readonly roots?: { readonly listChanged?: boolean }
	readonly sampling?: Record<string, never>
	readonly elicitation?: Record<string, never>
}

// ============================================================================
// Implementation Info
// ============================================================================

export interface Implementation {
	readonly name: string
	readonly version: string
}

// ============================================================================
// Roots
// ============================================================================

export interface Root {
	readonly uri: string
	readonly name?: string
}

// ============================================================================
// Progress
// ============================================================================

export type ProgressToken = string | number

export interface Progress {
	readonly progress: number
	readonly total?: number
	readonly message?: string
}

// ============================================================================
// Logging
// ============================================================================

export type LogLevel =
	| "debug"
	| "info"
	| "notice"
	| "warning"
	| "error"
	| "critical"
	| "alert"
	| "emergency"

export interface LogEntry {
	readonly level: LogLevel
	readonly logger?: string
	readonly data?: unknown
}

// ============================================================================
// MCP Method Names
// ============================================================================

export const Method = {
	// Lifecycle
	Initialize: "initialize",
	Initialized: "notifications/initialized",
	Ping: "ping",

	// Resources
	ResourcesList: "resources/list",
	ResourcesTemplatesList: "resources/templates/list",
	ResourcesRead: "resources/read",
	ResourcesSubscribe: "resources/subscribe",
	ResourcesUnsubscribe: "resources/unsubscribe",
	ResourcesUpdated: "notifications/resources/updated",
	ResourcesListChanged: "notifications/resources/list_changed",

	// Prompts
	PromptsList: "prompts/list",
	PromptsGet: "prompts/get",
	PromptsListChanged: "notifications/prompts/list_changed",

	// Tools
	ToolsList: "tools/list",
	ToolsCall: "tools/call",
	ToolsListChanged: "notifications/tools/list_changed",

	// Logging
	LoggingSetLevel: "logging/setLevel",
	LogMessage: "notifications/message",

	// Completions
	CompletionComplete: "completion/complete",

	// Sampling (Server → Client)
	SamplingCreateMessage: "sampling/createMessage",

	// Progress & Cancellation
	ProgressNotification: "notifications/progress",
	CancelledNotification: "notifications/cancelled",

	// Roots
	RootsList: "roots/list",
	RootsListChanged: "notifications/roots/list_changed",
} as const

export type Method = (typeof Method)[keyof typeof Method]

// ============================================================================
// Request/Response Params
// ============================================================================

export interface InitializeParams {
	readonly protocolVersion: string
	readonly capabilities: ClientCapabilities
	readonly clientInfo: Implementation
}

export interface InitializeResult {
	readonly protocolVersion: string
	readonly capabilities: ServerCapabilities
	readonly serverInfo: Implementation
	readonly instructions?: string
}

export interface ListParams {
	readonly cursor?: string
}

export interface ListResult<T> {
	readonly items: readonly T[]
	readonly nextCursor?: string
}

export interface ResourcesListResult extends ListResult<Resource> {}
export interface ResourceTemplatesListResult extends ListResult<ResourceTemplate> {}

export interface ResourcesReadParams {
	readonly uri: string
}

export interface ResourcesReadResult {
	readonly contents: readonly EmbeddedResource[]
}

export interface PromptsListResult extends ListResult<Prompt> {}

export interface PromptsGetParams {
	readonly name: string
	readonly arguments?: Record<string, string>
}

export interface PromptsGetResult {
	readonly description?: string
	readonly messages: readonly PromptMessage[]
}

export interface ToolsListResult extends ListResult<Tool> {}

export interface ToolsCallParams {
	readonly name: string
	readonly arguments?: Record<string, unknown>
}

export interface ToolsCallResult {
	readonly content: readonly Content[]
	readonly isError?: boolean
	readonly structuredContent?: unknown
}

export interface ProgressParams {
	readonly progressToken: ProgressToken
	readonly progress: number
	readonly total?: number
	readonly message?: string
}

export interface LoggingSetLevelParams {
	readonly level: LogLevel
}

export interface RootsListResult {
	readonly roots: readonly Root[]
}

// ============================================================================
// Sampling (Server → Client LLM Request)
// ============================================================================

export interface SamplingMessage {
	readonly role: "user" | "assistant"
	readonly content: TextContent | ImageContent | AudioContent
}

export interface ModelPreferences {
	readonly hints?: readonly ModelHint[]
	readonly costPriority?: number
	readonly speedPriority?: number
	readonly intelligencePriority?: number
}

export interface ModelHint {
	readonly name?: string
}

export interface SamplingCreateParams {
	readonly messages: readonly SamplingMessage[]
	readonly modelPreferences?: ModelPreferences
	readonly systemPrompt?: string
	readonly includeContext?: "none" | "thisServer" | "allServers"
	readonly temperature?: number
	readonly maxTokens: number
	readonly stopSequences?: readonly string[]
	readonly metadata?: Record<string, unknown>
}

export interface SamplingCreateResult {
	readonly role: "user" | "assistant"
	readonly content: TextContent | ImageContent | AudioContent
	readonly model: string
	readonly stopReason?: "endTurn" | "stopSequence" | "maxTokens" | string
}

// ============================================================================
// Completions (Auto-Complete)
// ============================================================================

export interface CompletionReference {
	readonly type: "ref/prompt" | "ref/resource"
	readonly name?: string // for prompts
	readonly uri?: string // for resources
}

export interface CompletionArgument {
	readonly name: string
	readonly value: string
}

export interface CompletionCompleteParams {
	readonly ref: CompletionReference
	readonly argument: CompletionArgument
}

export interface CompletionCompleteResult {
	readonly completion: {
		readonly values: readonly string[]
		readonly total?: number
		readonly hasMore?: boolean
	}
}

// ============================================================================
// Resource Subscriptions
// ============================================================================

export interface ResourcesSubscribeParams {
	readonly uri: string
}

export interface ResourcesUnsubscribeParams {
	readonly uri: string
}

// ============================================================================
// Cancellation
// ============================================================================

export interface CancelledNotificationParams {
	readonly requestId: string | number
	readonly reason?: string
}
