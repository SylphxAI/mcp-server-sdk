/**
 * Pure Functional Tool Builder
 *
 * Tools are pure functions that:
 * 1. Take validated input
 * 2. Return a handler that takes context (effects)
 * 3. Produce content output
 */

import type { z } from "zod"
import type {
	Content,
	JsonSchema,
	TextContent,
	Tool,
	ToolAnnotations,
	ToolsCallResult,
} from "../protocol/mcp.js"
import { type Infer, isZodSchema, toJsonSchema, validate } from "../schema/zod.js"

// ============================================================================
// Context Type (Dependency Injection)
// ============================================================================

/**
 * Context provided to tool handlers.
 * Extend this interface to add custom dependencies.
 */
export interface ToolContext {
	/** Signal for cancellation */
	readonly signal?: AbortSignal
	/** Report progress */
	readonly progress?: (current: number, total?: number, message?: string) => void
}

// ============================================================================
// Tool Definition Types
// ============================================================================

/** Handler function type - pure function taking input and returning effect */
export type ToolHandler<TInput, TContext extends ToolContext = ToolContext> = (
	input: TInput,
) => (ctx: TContext) => Promise<ToolsCallResult> | ToolsCallResult

/** Tool definition with metadata and handler */
export interface ToolDefinition<TInput = unknown, TContext extends ToolContext = ToolContext> {
	readonly name: string
	readonly description?: string
	readonly inputSchema: JsonSchema
	readonly annotations?: ToolAnnotations
	readonly handler: ToolHandler<TInput, TContext>
}

// ============================================================================
// Builder Config
// ============================================================================

export interface ToolConfig<TInput, TContext extends ToolContext = ToolContext> {
	readonly name: string
	readonly description?: string
	readonly input: JsonSchema
	readonly annotations?: ToolAnnotations
	readonly handler: ToolHandler<TInput, TContext>
}

// ============================================================================
// Pure Builder Function
// ============================================================================

/**
 * Creates a tool definition from config.
 * Pure function - no side effects.
 *
 * @example
 * ```ts
 * const readFile = tool({
 *   name: "read_file",
 *   description: "Read a file from disk",
 *   input: {
 *     type: "object",
 *     properties: {
 *       path: { type: "string", description: "File path" }
 *     },
 *     required: ["path"]
 *   },
 *   handler: ({ path }) => async (ctx) => {
 *     const content = await ctx.fs.read(path)
 *     return text(content)
 *   }
 * })
 * ```
 */
export const tool = <TInput, TContext extends ToolContext = ToolContext>(
	config: ToolConfig<TInput, TContext>,
): ToolDefinition<TInput, TContext> => ({
	name: config.name,
	description: config.description,
	inputSchema: config.input,
	annotations: config.annotations,
	handler: config.handler,
})

// ============================================================================
// Zod-Typed Tool Builder
// ============================================================================

/**
 * Config for defineTool with Zod schema
 */
export interface TypedToolConfig<
	TSchema extends z.ZodType,
	TContext extends ToolContext = ToolContext,
> {
	readonly name: string
	readonly description?: string
	readonly input: TSchema
	readonly annotations?: ToolAnnotations
	readonly handler: ToolHandler<Infer<TSchema>, TContext>
}

/**
 * Tool definition with Zod schema for validation
 */
export interface TypedToolDefinition<
	TSchema extends z.ZodType,
	TContext extends ToolContext = ToolContext,
> extends ToolDefinition<Infer<TSchema>, TContext> {
	readonly schema: TSchema
}

/**
 * Create a tool with Zod schema validation.
 * Provides full type inference from schema.
 *
 * @example
 * ```ts
 * import { z } from "zod"
 *
 * const greet = defineTool({
 *   name: "greet",
 *   description: "Greet someone",
 *   input: z.object({
 *     name: z.string().describe("Name to greet"),
 *     excited: z.boolean().optional().default(false),
 *   }),
 *   handler: ({ name, excited }) => () =>
 *     text(`Hello, ${name}${excited ? "!" : "."}`),
 * })
 * ```
 */
export const defineTool = <TSchema extends z.ZodType, TContext extends ToolContext = ToolContext>(
	config: TypedToolConfig<TSchema, TContext>,
): TypedToolDefinition<TSchema, TContext> => {
	const jsonSchema = toJsonSchema(config.input)

	// Wrap handler with validation
	const validatedHandler: ToolHandler<Infer<TSchema>, TContext> = (rawInput) => async (ctx) => {
		const result = validate(config.input, rawInput)
		if (!result.success) {
			return toolError(`Validation error: ${result.error}`)
		}
		return config.handler(result.data as Infer<TSchema>)(ctx)
	}

	return {
		name: config.name,
		description: config.description,
		inputSchema: jsonSchema,
		annotations: config.annotations,
		handler: validatedHandler,
		schema: config.input,
	}
}

/**
 * Create a tool that accepts any schema (Zod or JSON Schema).
 * When Zod is used, provides validation; otherwise uses raw JSON Schema.
 */
export const createTool = <TInput, TContext extends ToolContext = ToolContext>(config: {
	readonly name: string
	readonly description?: string
	readonly input: z.ZodType<TInput> | JsonSchema
	readonly annotations?: ToolAnnotations
	readonly handler: ToolHandler<TInput, TContext>
}): ToolDefinition<TInput, TContext> => {
	const jsonSchema = toJsonSchema(config.input)

	// If Zod schema, add validation
	if (isZodSchema(config.input)) {
		const zodSchema = config.input as z.ZodType<TInput>
		const validatedHandler: ToolHandler<TInput, TContext> = (rawInput) => async (ctx) => {
			const result = validate(zodSchema, rawInput)
			if (!result.success) {
				return toolError(`Validation error: ${result.error}`)
			}
			return config.handler(result.data)(ctx)
		}

		return {
			name: config.name,
			description: config.description,
			inputSchema: jsonSchema,
			annotations: config.annotations,
			handler: validatedHandler,
		}
	}

	// Raw JSON Schema - no validation
	return {
		name: config.name,
		description: config.description,
		inputSchema: jsonSchema,
		annotations: config.annotations,
		handler: config.handler,
	}
}

// ============================================================================
// Tool Metadata Extraction (for protocol)
// ============================================================================

/**
 * Extract protocol Tool type from definition.
 * Pure function.
 */
export const toProtocolTool = (def: ToolDefinition): Tool => ({
	name: def.name,
	description: def.description,
	inputSchema: def.inputSchema,
	annotations: def.annotations,
})

// ============================================================================
// Content Helpers (Pure Functions)
// ============================================================================

/** Create text content */
export const textContent = (text: string): TextContent => ({
	type: "text",
	text,
})

/** Create successful tool result with text */
export const text = (content: string): ToolsCallResult => ({
	content: [textContent(content)],
})

/** Create successful tool result with multiple contents */
export const contents = (...items: Content[]): ToolsCallResult => ({
	content: items,
})

/** Create error tool result */
export const toolError = (message: string): ToolsCallResult => ({
	content: [textContent(message)],
	isError: true,
})

/** Create tool result with structured content */
export const structured = <T>(textContent: string, data: T): ToolsCallResult => ({
	content: [{ type: "text", text: textContent }],
	structuredContent: data,
})

// ============================================================================
// Tool Composition (Pure Functions)
// ============================================================================

/**
 * Compose multiple tool handlers sequentially.
 * Each handler's output becomes part of the final result.
 */
export const sequence =
	<TInput, TContext extends ToolContext>(
		...handlers: Array<ToolHandler<TInput, TContext>>
	): ToolHandler<TInput, TContext> =>
	(input) =>
	async (ctx) => {
		const allContent: Content[] = []
		for (const handler of handlers) {
			const result = await handler(input)(ctx)
			allContent.push(...result.content)
			if (result.isError) {
				return { content: allContent, isError: true }
			}
		}
		return { content: allContent }
	}

/**
 * Add precondition check to a handler.
 * If predicate fails, returns error without running handler.
 */
export const guard =
	<TInput, TContext extends ToolContext>(
		predicate: (input: TInput) => boolean | string,
		handler: ToolHandler<TInput, TContext>,
	): ToolHandler<TInput, TContext> =>
	(input) =>
	async (ctx) => {
		const result = predicate(input)
		if (result === false) {
			return toolError("Precondition failed")
		}
		if (typeof result === "string") {
			return toolError(result)
		}
		return handler(input)(ctx)
	}

/**
 * Map over handler output.
 */
export const mapResult =
	<TInput, TContext extends ToolContext>(
		handler: ToolHandler<TInput, TContext>,
		fn: (result: ToolsCallResult) => ToolsCallResult,
	): ToolHandler<TInput, TContext> =>
	(input) =>
	async (ctx) => {
		const result = await handler(input)(ctx)
		return fn(result)
	}
