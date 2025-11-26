/**
 * Tool Builder
 *
 * Builder pattern for creating type-safe MCP tools.
 *
 * @example
 * ```ts
 * const greet = tool()
 *   .description('Greet someone')
 *   .input(z.object({ name: z.string() }))
 *   .handler(({ input }) => text(`Hello ${input.name}`))
 *
 * const ping = tool()
 *   .handler(() => text('pong'))
 * ```
 */

import type { z } from "zod"
import type {
	AudioContent,
	Content,
	ContentAnnotations,
	EmbeddedResource,
	ImageContent,
	JsonSchema,
	ResourceContent,
	TextContent,
	Tool,
	ToolAnnotations,
	ToolsCallResult,
} from "../protocol/mcp.js"
import { toJsonSchema, validate } from "../schema/zod.js"

// ============================================================================
// Context Type
// ============================================================================

/**
 * Context provided to tool handlers.
 */
export interface ToolContext {
	readonly signal?: AbortSignal
}

// ============================================================================
// Handler Types
// ============================================================================

export interface ToolHandlerArgs<TInput = void> {
	readonly input: TInput
	readonly ctx: ToolContext
}

export type ToolHandler<TInput = void> = TInput extends void
	? (args: { ctx: ToolContext }) => ToolsCallResult | Promise<ToolsCallResult>
	: (args: ToolHandlerArgs<TInput>) => ToolsCallResult | Promise<ToolsCallResult>

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
	input<T>(schema: z.ZodType<T>): ToolBuilderWithInput<T>
	handler(
		fn: (args: { ctx: ToolContext }) => ToolsCallResult | Promise<ToolsCallResult>
	): ToolDefinition<void>
}

interface ToolBuilderWithInput<TInput> {
	description(desc: string): ToolBuilderWithInput<TInput>
	annotations(annotations: ToolAnnotations): ToolBuilderWithInput<TInput>
	handler(
		fn: (args: ToolHandlerArgs<TInput>) => ToolsCallResult | Promise<ToolsCallResult>
	): ToolDefinition<TInput>
}

// ============================================================================
// Builder Implementation
// ============================================================================

interface BuilderState {
	description?: string
	annotations?: ToolAnnotations
	inputSchema?: z.ZodType
}

const createBuilder = <TInput = void>(state: BuilderState = {}): ToolBuilderWithoutInput => ({
	description(desc: string) {
		return createBuilder<TInput>({ ...state, description: desc }) as ToolBuilderWithoutInput
	},

	annotations(annotations: ToolAnnotations) {
		return createBuilder<TInput>({ ...state, annotations }) as ToolBuilderWithoutInput
	},

	input<T>(schema: z.ZodType<T>): ToolBuilderWithInput<T> {
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
	fn: (args: { ctx: ToolContext }) => ToolsCallResult | Promise<ToolsCallResult>
): ToolDefinition<void> => ({
	description: state.description,
	inputSchema: { type: "object", properties: {} },
	annotations: state.annotations,
	handler: async ({ ctx }) => fn({ ctx }),
})

const createDefinitionWithInput = <T>(
	state: BuilderState,
	schema: z.ZodType<T>,
	fn: (args: ToolHandlerArgs<T>) => ToolsCallResult | Promise<ToolsCallResult>
): ToolDefinition<T> => ({
	description: state.description,
	inputSchema: toJsonSchema(schema),
	annotations: state.annotations,
	handler: async ({ input, ctx }) => {
		const result = validate(schema, input)
		if (!result.success) {
			return toolError(`Validation error: ${result.error}`)
		}
		return fn({ input: result.data as T, ctx })
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
 *   .input(z.object({ name: z.string() }))
 *   .handler(({ input }) => text(`Hello ${input.name}`))
 *
 * // Without input
 * const ping = tool()
 *   .description('Ping the server')
 *   .handler(() => text('pong'))
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
export const textContent = (
	text: string,
	annotations?: ContentAnnotations
): TextContent => ({
	type: "text",
	text,
	...(annotations && { annotations }),
})

/** Create image content (base64 encoded) */
export const imageContent = (
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
export const audioContent = (
	data: string,
	mimeType: string,
	annotations?: ContentAnnotations
): AudioContent => ({
	type: "audio",
	data,
	mimeType,
	...(annotations && { annotations }),
})

/** Create resource content (embedded resource) */
export const resourceContent = (
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

/** Return text result */
export const text = (content: string): ToolsCallResult => ({
	content: [textContent(content)],
})

/** Return image result */
export const image = (data: string, mimeType: string): ToolsCallResult => ({
	content: [imageContent(data, mimeType)],
})

/** Return audio result */
export const audio = (data: string, mimeType: string): ToolsCallResult => ({
	content: [audioContent(data, mimeType)],
})

/** Return multiple content items */
export const contents = (...items: Content[]): ToolsCallResult => ({
	content: items,
})

/** Return error result */
export const toolError = (message: string): ToolsCallResult => ({
	content: [textContent(message)],
	isError: true,
})

/** Return JSON as formatted text */
export const json = <T>(data: T): ToolsCallResult => ({
	content: [textContent(JSON.stringify(data, null, 2))],
})
