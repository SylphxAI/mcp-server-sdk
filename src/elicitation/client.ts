/**
 * Elicitation Client
 *
 * Server-side client for requesting user input through the MCP client.
 */

import type {
	ElicitationClient,
	ElicitationCreateParams,
	ElicitationCreateResult,
	ElicitationProperty,
	ElicitationRequestSender,
	ElicitationSchema,
} from "./types.js"

// ============================================================================
// Elicitation Client Factory
// ============================================================================

/**
 * Create an elicitation client for requesting user input.
 *
 * @example
 * ```ts
 * // In tool handler with elicitation context
 * const result = await ctx.elicit?.(
 *   "Please provide your API key",
 *   {
 *     type: "object",
 *     properties: {
 *       apiKey: { type: "string", description: "Your API key" },
 *     },
 *     required: ["apiKey"],
 *   }
 * )
 *
 * if (result?.action === "accept") {
 *   const apiKey = result.content?.apiKey as string
 * }
 * ```
 */
export const createElicitationClient = (sender: ElicitationRequestSender): ElicitationClient => {
	const elicit = async (
		message: string,
		schema: ElicitationSchema,
	): Promise<ElicitationCreateResult> => {
		const params: ElicitationCreateParams = {
			message,
			requestedSchema: schema,
		}
		return sender("elicitation/create", params)
	}

	return { elicit }
}

// ============================================================================
// Schema Helpers
// ============================================================================

/**
 * Create a string property for elicitation schema.
 */
export const elicitString = (options?: {
	description?: string
	default?: string
	format?: "email" | "uri" | "date" | "date-time"
	minLength?: number
	maxLength?: number
}): ElicitationProperty => ({
	type: "string",
	...options,
})

/**
 * Create a number property for elicitation schema.
 */
export const elicitNumber = (options?: {
	description?: string
	default?: number
	minimum?: number
	maximum?: number
}): ElicitationProperty => ({
	type: "number",
	...options,
})

/**
 * Create an integer property for elicitation schema.
 */
export const elicitInteger = (options?: {
	description?: string
	default?: number
	minimum?: number
	maximum?: number
}): ElicitationProperty => ({
	type: "integer",
	...options,
})

/**
 * Create a boolean property for elicitation schema.
 */
export const elicitBoolean = (options?: {
	description?: string
	default?: boolean
}): ElicitationProperty => ({
	type: "boolean",
	...options,
})

/**
 * Create an enum property for elicitation schema.
 */
export const elicitEnum = <T extends string | number>(
	values: readonly T[],
	options?: {
		description?: string
		default?: T
		enumNames?: readonly string[]
	},
): ElicitationProperty => ({
	type: typeof values[0] === "string" ? "string" : "number",
	enum: values,
	...options,
})

/**
 * Create an elicitation schema from properties.
 */
export const elicitSchema = (
	properties: Record<string, ReturnType<typeof elicitString | typeof elicitNumber | typeof elicitBoolean>>,
	required?: readonly string[],
): ElicitationSchema => ({
	type: "object",
	properties,
	required,
})
