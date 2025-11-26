/**
 * Prompt Builder
 *
 * Builder pattern for creating MCP prompts with type-safe arguments.
 *
 * @example
 * ```ts
 * const review = prompt()
 *   .description('Code review prompt')
 *   .args(z.object({
 *     language: z.string(),
 *     focus: z.string().optional(),
 *   }))
 *   .handler(({ args }) => messages(user(`Review ${args.language} code`)))
 *
 * const simple = prompt()
 *   .description('Simple prompt')
 *   .handler(() => messages(user('Hello!')))
 * ```
 */

import type { z } from 'zod'
import type { Content, Prompt, PromptArgument, PromptMessage, PromptsGetResult } from '../protocol/mcp.js'
import { type Infer, extractObjectFields, validate } from '../schema/zod.js'

// ============================================================================
// Context Type
// ============================================================================

export interface PromptContext {
	readonly signal?: AbortSignal
}

// ============================================================================
// Handler Types
// ============================================================================

export interface PromptHandlerArgs<TArgs = void> {
	readonly args: TArgs
	readonly ctx: PromptContext
}

export type PromptHandler<TArgs = void> = TArgs extends void
	? (args: { ctx: PromptContext }) => PromptsGetResult | Promise<PromptsGetResult>
	: (args: PromptHandlerArgs<TArgs>) => PromptsGetResult | Promise<PromptsGetResult>

// ============================================================================
// Prompt Definition
// ============================================================================

export interface PromptDefinition<TArgs = void> {
	readonly description?: string
	readonly arguments: readonly PromptArgument[]
	readonly handler: (args: { args: unknown; ctx: PromptContext }) => Promise<PromptsGetResult>
}

// ============================================================================
// Builder Types
// ============================================================================

interface PromptBuilderWithoutArgs {
	description(desc: string): PromptBuilderWithoutArgs
	args<T extends z.ZodObject<z.ZodRawShape>>(schema: T): PromptBuilderWithArgs<Infer<T>>
	handler(fn: (args: { ctx: PromptContext }) => PromptsGetResult | Promise<PromptsGetResult>): PromptDefinition<void>
}

interface PromptBuilderWithArgs<TArgs> {
	description(desc: string): PromptBuilderWithArgs<TArgs>
	handler(fn: (args: PromptHandlerArgs<TArgs>) => PromptsGetResult | Promise<PromptsGetResult>): PromptDefinition<TArgs>
}

// ============================================================================
// Builder Implementation
// ============================================================================

interface BuilderState {
	description?: string
	argsSchema?: z.ZodObject<z.ZodRawShape>
}

const createBuilder = <TArgs = void>(state: BuilderState = {}): PromptBuilderWithoutArgs => ({
	description(desc: string) {
		return createBuilder<TArgs>({ ...state, description: desc }) as PromptBuilderWithoutArgs
	},

	args<T extends z.ZodObject<z.ZodRawShape>>(schema: T): PromptBuilderWithArgs<Infer<T>> {
		const newState = { ...state, argsSchema: schema }
		return {
			description(desc: string) {
				return createBuilder({ ...newState, description: desc }) as unknown as PromptBuilderWithArgs<Infer<T>>
			},
			handler(fn) {
				return createDefinitionWithArgs(newState, schema, fn)
			},
		} as PromptBuilderWithArgs<Infer<T>>
	},

	handler(fn) {
		return createDefinitionNoArgs(state, fn)
	},
})

const createDefinitionNoArgs = (
	state: BuilderState,
	fn: (args: { ctx: PromptContext }) => PromptsGetResult | Promise<PromptsGetResult>
): PromptDefinition<void> => ({
	description: state.description,
	arguments: [],
	handler: async ({ ctx }) => fn({ ctx }),
})

const createDefinitionWithArgs = <T>(
	state: BuilderState,
	schema: z.ZodObject<z.ZodRawShape>,
	fn: (args: PromptHandlerArgs<T>) => PromptsGetResult | Promise<PromptsGetResult>
): PromptDefinition<T> => ({
	description: state.description,
	arguments: extractPromptArgs(schema),
	handler: async ({ args, ctx }) => {
		const result = validate(schema, args)
		if (!result.success) {
			return { messages: [user(`Validation error: ${result.error}`)] }
		}
		return fn({ args: result.data as T, ctx })
	},
})

const extractPromptArgs = (schema: z.ZodObject<z.ZodRawShape>): PromptArgument[] => {
	const fields = extractObjectFields(schema)
	return fields.map((field) => ({
		name: field.name,
		description: field.description,
		required: field.required,
	}))
}

// ============================================================================
// Public API
// ============================================================================

/**
 * Create a new prompt using builder pattern.
 *
 * @example
 * ```ts
 * // With arguments
 * const review = prompt()
 *   .description('Code review')
 *   .args(z.object({ language: z.string() }))
 *   .handler(({ args }) => messages(user(`Review ${args.language}`)))
 *
 * // Without arguments
 * const hello = prompt()
 *   .description('Say hello')
 *   .handler(() => messages(user('Hello!')))
 * ```
 */
export const prompt = (): PromptBuilderWithoutArgs => createBuilder()

// ============================================================================
// Protocol Conversion
// ============================================================================

export const toProtocolPrompt = (name: string, def: PromptDefinition): Prompt => ({
	name,
	description: def.description,
	arguments: def.arguments,
})

// ============================================================================
// Message Helpers
// ============================================================================

export const user = (text: string): PromptMessage => ({
	role: 'user',
	content: { type: 'text', text },
})

export const assistant = (text: string): PromptMessage => ({
	role: 'assistant',
	content: { type: 'text', text },
})

export const message = (role: 'user' | 'assistant', content: Content): PromptMessage => ({
	role,
	content,
})

export const messages = (...msgs: PromptMessage[]): PromptsGetResult => ({
	messages: msgs,
})

export const promptResult = (description: string, ...msgs: PromptMessage[]): PromptsGetResult => ({
	description,
	messages: msgs,
})

// ============================================================================
// Template Helpers
// ============================================================================

export const interpolate = (template: string, args: Record<string, string>): string =>
	template.replace(/\{\{(\w+)\}\}/g, (_, key) => args[key] ?? `{{${key}}}`)
