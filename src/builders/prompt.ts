/**
 * Pure Functional Prompt Builder
 *
 * Prompts are templates that generate messages for LLMs.
 */

import type {
	Content,
	Prompt,
	PromptArgument,
	PromptMessage,
	PromptsGetResult,
	TextContent,
} from "../protocol/mcp.js"

// ============================================================================
// Context Type
// ============================================================================

export interface PromptContext {
	readonly signal?: AbortSignal
}

// ============================================================================
// Prompt Definition Types
// ============================================================================

/** Handler for generating prompt messages */
export type PromptHandler<TContext extends PromptContext = PromptContext> = (
	args: Record<string, string>,
) => (ctx: TContext) => Promise<PromptsGetResult> | PromptsGetResult

/** Prompt definition with metadata and handler */
export interface PromptDefinition<TContext extends PromptContext = PromptContext> {
	readonly name: string
	readonly description?: string
	readonly arguments: readonly PromptArgument[]
	readonly handler: PromptHandler<TContext>
}

// ============================================================================
// Builder Config
// ============================================================================

export interface PromptConfig<TContext extends PromptContext = PromptContext> {
	readonly name: string
	readonly description?: string
	readonly arguments?: readonly PromptArgument[]
	readonly handler: PromptHandler<TContext>
}

export interface PromptArgumentConfig {
	readonly name: string
	readonly description?: string
	readonly required?: boolean
}

// ============================================================================
// Pure Builder Functions
// ============================================================================

/**
 * Create a prompt definition.
 *
 * @example
 * ```ts
 * const codeReview = prompt({
 *   name: "code_review",
 *   description: "Review code for issues",
 *   arguments: [
 *     arg({ name: "language", required: true }),
 *     arg({ name: "focus", description: "What to focus on" })
 *   ],
 *   handler: ({ language, focus }) => async (ctx) => ({
 *     messages: [
 *       user(`Review this ${language} code${focus ? ` focusing on ${focus}` : ""}`),
 *     ]
 *   })
 * })
 * ```
 */
export const prompt = <TContext extends PromptContext = PromptContext>(
	config: PromptConfig<TContext>,
): PromptDefinition<TContext> => ({
	name: config.name,
	description: config.description,
	arguments: config.arguments ?? [],
	handler: config.handler,
})

/**
 * Create a prompt argument definition.
 */
export const arg = (config: PromptArgumentConfig): PromptArgument => ({
	name: config.name,
	description: config.description,
	required: config.required,
})

// ============================================================================
// Metadata Extraction
// ============================================================================

export const toProtocolPrompt = (def: PromptDefinition): Prompt => ({
	name: def.name,
	description: def.description,
	arguments: def.arguments,
})

// ============================================================================
// Message Helpers
// ============================================================================

/** Create user message with text */
export const user = (text: string): PromptMessage => ({
	role: "user",
	content: { type: "text", text },
})

/** Create assistant message with text */
export const assistant = (text: string): PromptMessage => ({
	role: "assistant",
	content: { type: "text", text },
})

/** Create message with custom content */
export const message = (role: "user" | "assistant", content: Content): PromptMessage => ({
	role,
	content,
})

/** Create prompt result with messages */
export const messages = (...msgs: PromptMessage[]): PromptsGetResult => ({
	messages: msgs,
})

/** Create prompt result with description */
export const promptResult = (description: string, ...msgs: PromptMessage[]): PromptsGetResult => ({
	description,
	messages: msgs,
})

// ============================================================================
// Template Helpers
// ============================================================================

/**
 * Simple template string interpolation.
 * Replaces {{key}} with values from args.
 */
export const interpolate = (template: string, args: Record<string, string>): string =>
	template.replace(/\{\{(\w+)\}\}/g, (_, key) => args[key] ?? `{{${key}}}`)

/**
 * Create a simple text-based prompt from a template.
 */
export const templatePrompt = <TContext extends PromptContext = PromptContext>(config: {
	readonly name: string
	readonly description?: string
	readonly template: string
	readonly arguments?: readonly PromptArgument[]
}): PromptDefinition<TContext> =>
	prompt({
		name: config.name,
		description: config.description,
		arguments: config.arguments,
		handler: (args) => () => ({
			messages: [user(interpolate(config.template, args))],
		}),
	})
