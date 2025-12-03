/**
 * Tool Builder
 *
 * Builder pattern for creating type-safe MCP tools.
 *
 * @example
 * ```ts
 * const greet = tool()
 *   .description('Greet someone')
 *   .input(object({ name: str() }))
 *   .handler(({ input }) => text(`Hello ${input.name}`))
 *
 * const ping = tool()
 *   .handler(() => text('pong'))
 *
 * // Multiple content items
 * const analyze = tool()
 *   .handler(() => [text("Result:"), image(data, "image/png")])
 * ```
 */

import type { Parser } from "@sylphx/vex"
import type { NotificationEmitter } from "../notifications/types.js"
import type {
	AudioContent,
	Content,
	ContentAnnotations,
	EmbeddedResource,
	ImageContent,
	JsonSchema,
	LogLevel,
	ProgressToken,
	ResourceContent,
	TextContent,
	Tool,
	ToolAnnotations,
	ToolsCallResult,
} from "../protocol/mcp.js"
import { validate, vexToJsonSchema } from "../schema/vex.js"

// ============================================================================
// Context Type
// ============================================================================

/**
 * Context provided to tool handlers.
 */
export interface ToolContext {
	readonly signal?: AbortSignal
	/** Progress token from the request (if client requested progress updates) */
	readonly progressToken?: ProgressToken
	/** Send a log message to the client */
	readonly log: (level: LogLevel, data: unknown, logger?: string) => void
	/**
	 * Send progress notification to the client.
	 * Only sends if client provided a progressToken in the request.
	 */
	readonly progress: (current: number, options?: { total?: number; message?: string }) => void
	/** Raw notification emitter for advanced use */
	readonly notify?: NotificationEmitter
	/**
	 * Request LLM completion from the client.
	 * Only available if client declared sampling capability.
	 */
	readonly sampling?: {
		readonly createMessage: (params: {
			messages: ReadonlyArray<{
				role: "user" | "assistant"
				content:
					| { type: "text"; text: string }
					| { type: "image"; data: string; mimeType: string }
					| { type: "audio"; data: string; mimeType: string }
			}>
			maxTokens: number
			systemPrompt?: string
			temperature?: number
			stopSequences?: readonly string[]
			modelPreferences?: {
				hints?: ReadonlyArray<{ name?: string }>
				costPriority?: number
				speedPriority?: number
				intelligencePriority?: number
			}
		}) => Promise<{
			role: "user" | "assistant"
			content:
				| { type: "text"; text: string }
				| { type: "image"; data: string; mimeType: string }
				| { type: "audio"; data: string; mimeType: string }
			model: string
			stopReason?: string
		}>
	}
	/**
	 * Request user input from the client.
	 * Only available if client declared elicitation capability.
	 */
	readonly elicit?: (
		message: string,
		schema: {
			type: "object"
			properties: Record<
				string,
				{
					type: "string" | "number" | "integer" | "boolean"
					description?: string
					default?: string | number | boolean
					enum?: readonly (string | number)[]
				}
			>
			required?: readonly string[]
		}
	) => Promise<{
		action: "accept" | "decline" | "cancel"
		content?: Record<string, unknown>
	}>
}

// ============================================================================
// Handler Types
// ============================================================================

/** Handler can return single content, array, or full result */
export type ToolResult = Content | Content[] | ToolsCallResult

export interface ToolHandlerArgs<TInput = void> {
	readonly input: TInput
	readonly ctx: ToolContext
}

export type ToolHandler<TInput = void> = TInput extends void
	? (args: { ctx: ToolContext }) => ToolResult | Promise<ToolResult>
	: (args: ToolHandlerArgs<TInput>) => ToolResult | Promise<ToolResult>

// ============================================================================
// Tool Definition
// ============================================================================

export interface ToolDefinition<_TInput = void> {
	readonly name?: string
	readonly description?: string
	readonly inputSchema: JsonSchema
	readonly annotations?: ToolAnnotations
	readonly handler: (args: { input: unknown; ctx: ToolContext }) => Promise<ToolsCallResult>
}

// ============================================================================
// Builder Types
// ============================================================================

interface ToolBuilderWithoutInput {
	description(desc: string): ToolBuilderWithoutInput
	annotations(annotations: ToolAnnotations): ToolBuilderWithoutInput
	input<T>(schema: Parser<T>): ToolBuilderWithInput<T>
	handler(
		fn: (args: { ctx: ToolContext }) => ToolResult | Promise<ToolResult>
	): ToolDefinition<void>
}

interface ToolBuilderWithInput<TInput> {
	description(desc: string): ToolBuilderWithInput<TInput>
	annotations(annotations: ToolAnnotations): ToolBuilderWithInput<TInput>
	handler(
		fn: (args: ToolHandlerArgs<TInput>) => ToolResult | Promise<ToolResult>
	): ToolDefinition<TInput>
}

// ============================================================================
// Result Normalization
// ============================================================================

/** Convert handler result to ToolsCallResult */
const normalizeResult = (result: ToolResult): ToolsCallResult => {
	// Already a full result (has content array)
	if ("content" in result && Array.isArray(result.content)) {
		return result as ToolsCallResult
	}
	// Array of content items
	if (Array.isArray(result)) {
		return { content: result }
	}
	// Single content item
	return { content: [result as Content] }
}

// ============================================================================
// Builder Implementation
// ============================================================================

interface BuilderState {
	description?: string
	annotations?: ToolAnnotations
	inputSchema?: Parser<unknown>
}

const createBuilder = <TInput = void>(state: BuilderState = {}): ToolBuilderWithoutInput => ({
	description(desc: string) {
		return createBuilder<TInput>({ ...state, description: desc }) as ToolBuilderWithoutInput
	},

	annotations(annotations: ToolAnnotations) {
		return createBuilder<TInput>({ ...state, annotations }) as ToolBuilderWithoutInput
	},

	input<T>(schema: Parser<T>): ToolBuilderWithInput<T> {
		const newState = { ...state, inputSchema: schema }
		return {
			description(desc: string) {
				return createBuilder({
					...newState,
					description: desc,
				}) as unknown as ToolBuilderWithInput<T>
			},
			annotations(annotations: ToolAnnotations) {
				return createBuilder({ ...newState, annotations }) as unknown as ToolBuilderWithInput<T>
			},
			handler(fn) {
				return createDefinitionWithInput(newState, schema, fn)
			},
		} as ToolBuilderWithInput<T>
	},

	handler(fn) {
		return createDefinitionNoInput(state, fn)
	},
})

const createDefinitionNoInput = (
	state: BuilderState,
	fn: (args: { ctx: ToolContext }) => ToolResult | Promise<ToolResult>
): ToolDefinition<void> => ({
	description: state.description,
	inputSchema: { type: "object", properties: {} },
	annotations: state.annotations,
	handler: async ({ ctx }) => normalizeResult(await fn({ ctx })),
})

const createDefinitionWithInput = <T>(
	state: BuilderState,
	schema: Parser<T>,
	fn: (args: ToolHandlerArgs<T>) => ToolResult | Promise<ToolResult>
): ToolDefinition<T> => ({
	description: state.description,
	inputSchema: vexToJsonSchema(schema),
	annotations: state.annotations,
	handler: async ({ input, ctx }) => {
		const result = validate(schema, input)
		if (!result.success) {
			return { content: [text(`Validation error: ${result.error}`)], isError: true }
		}
		return normalizeResult(await fn({ input: result.data as T, ctx }))
	},
})

// ============================================================================
// Public API
// ============================================================================

/**
 * Create a new tool using builder pattern.
 *
 * @example
 * ```ts
 * // With input
 * const greet = tool()
 *   .description('Greet someone')
 *   .input(object({ name: str() }))
 *   .handler(({ input }) => text(`Hello ${input.name}`))
 *
 * // Without input
 * const ping = tool()
 *   .description('Ping the server')
 *   .handler(() => text('pong'))
 *
 * // Multiple content
 * const multi = tool()
 *   .handler(() => [text("Hello"), image(data, "image/png")])
 * ```
 */
export const tool = (): ToolBuilderWithoutInput => createBuilder()

// ============================================================================
// Protocol Conversion
// ============================================================================

/**
 * Convert tool definition to MCP protocol format.
 */
export const toProtocolTool = (name: string, def: ToolDefinition): Tool => ({
	name,
	description: def.description,
	inputSchema: def.inputSchema,
	annotations: def.annotations,
})

// ============================================================================
// Content Helpers
// ============================================================================

/** Create text content */
export const text = (content: string, annotations?: ContentAnnotations): TextContent => ({
	type: "text",
	text: content,
	...(annotations && { annotations }),
})

/** Create image content (base64 encoded) */
export const image = (
	data: string,
	mimeType: string,
	annotations?: ContentAnnotations
): ImageContent => ({
	type: "image",
	data,
	mimeType,
	...(annotations && { annotations }),
})

/** Create audio content (base64 encoded) */
export const audio = (
	data: string,
	mimeType: string,
	annotations?: ContentAnnotations
): AudioContent => ({
	type: "audio",
	data,
	mimeType,
	...(annotations && { annotations }),
})

/** Create embedded resource content */
export const embedded = (
	resource: EmbeddedResource,
	annotations?: ContentAnnotations
): ResourceContent => ({
	type: "resource",
	resource,
	...(annotations && { annotations }),
})

// ============================================================================
// Result Helpers
// ============================================================================

/** Return error result */
export const toolError = (message: string): ToolsCallResult => ({
	content: [text(message)],
	isError: true,
})

/** Return JSON as formatted text */
export const json = <T>(data: T): TextContent => text(JSON.stringify(data, null, 2))
