/**
 * Vex Schema Integration
 *
 * Uses @sylphx/vex for ultra-fast schema validation.
 */

import { getMeta, type InferOutput, type Parser, safeParse, toJsonSchema } from "@sylphx/vex"
import type { JsonSchema } from "../protocol/mcp.js"

// ============================================================================
// Type Utilities
// ============================================================================

/** Extract the inferred type from a Vex schema */
export type Infer<T extends Parser<unknown>> = InferOutput<T>

// ============================================================================
// Vex to JSON Schema Conversion
// ============================================================================

/**
 * Convert a Vex schema to JSON Schema.
 */
export const vexToJsonSchema = (schema: Parser<unknown>): JsonSchema => {
	const jsonSchema = toJsonSchema(schema, { $schema: false })
	return jsonSchema as JsonSchema
}

// ============================================================================
// Validation
// ============================================================================

export type ValidationResult<T> =
	| { readonly success: true; readonly data: T }
	| { readonly success: false; readonly error: string }

/**
 * Validate input against a Vex schema.
 */
export const validate = <T>(schema: Parser<T>, input: unknown): ValidationResult<T> => {
	const result = safeParse(schema)(input)
	if (result.success) {
		return { success: true, data: result.data }
	}
	return { success: false, error: result.error || "Validation failed" }
}

// ============================================================================
// Schema Introspection (for prompt arguments)
// ============================================================================

/**
 * Extract field info from a Vex object schema.
 */
export const extractObjectFields = (
	schema: Parser<unknown>
): Array<{ name: string; description?: string; required: boolean }> => {
	const meta = getMeta(schema)
	if (!meta || meta.type !== "object") {
		return []
	}

	const inner = meta.inner as Record<string, Parser<unknown>> | undefined
	if (!inner || typeof inner !== "object") {
		return []
	}

	const fields: Array<{ name: string; description?: string; required: boolean }> = []

	for (const [key, fieldSchema] of Object.entries(inner)) {
		const fieldMeta = getMeta(fieldSchema)
		const isOptional = fieldMeta?.type === "optional" || fieldMeta?.type === "nullish"
		const hasDefault = fieldMeta?.default !== undefined

		fields.push({
			name: key,
			description: fieldMeta?.description,
			required: !isOptional && !hasDefault,
		})
	}

	return fields
}
