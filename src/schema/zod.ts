/**
 * Zod Schema Integration
 *
 * Uses Zod 4's built-in JSON Schema conversion.
 */

import { z } from "zod"
import type { JsonSchema } from "../protocol/mcp.js"

// ============================================================================
// Type Utilities
// ============================================================================

/** Extract the inferred type from a Zod schema */
export type Infer<T extends z.ZodType> = z.infer<T>

/** Schema that can be either Zod or raw JSON Schema */
export type SchemaInput<T = unknown> = z.ZodType<T> | JsonSchema

// ============================================================================
// Zod to JSON Schema Conversion
// ============================================================================

/**
 * Convert a Zod schema to JSON Schema using Zod 4's native conversion.
 */
export const zodToJsonSchema = (schema: z.ZodType): JsonSchema => {
	const jsonSchema = z.toJSONSchema(schema, {
		unrepresentable: "any",
		target: "draft-7",
	})
	// Remove $schema as it's not needed for MCP
	const { $schema, ...rest } = jsonSchema as { $schema?: string } & JsonSchema
	return rest
}

// ============================================================================
// Helpers
// ============================================================================

/**
 * Check if a value is a Zod schema.
 */
export const isZodSchema = (value: unknown): value is z.ZodType => {
	return (
		value !== null &&
		typeof value === "object" &&
		"_zod" in value &&
		typeof (value as z.ZodType).parse === "function"
	)
}

/**
 * Convert SchemaInput to JSON Schema.
 */
export const toJsonSchema = (schema: SchemaInput): JsonSchema => {
	if (isZodSchema(schema)) {
		return zodToJsonSchema(schema)
	}
	return schema
}

// ============================================================================
// Validation
// ============================================================================

export type ValidationResult<T> =
	| { readonly success: true; readonly data: T }
	| { readonly success: false; readonly error: string }

/**
 * Validate input against a Zod schema.
 */
export const validate = <T>(schema: z.ZodType<T>, input: unknown): ValidationResult<T> => {
	const result = schema.safeParse(input)
	if (result.success) {
		return { success: true, data: result.data }
	}
	// Zod 4 uses result.error.issues
	const issues = result.error.issues ?? result.error.errors ?? []
	const errorMsg = issues
		.map((e: { path: (string | number)[]; message: string }) => `${e.path.join(".")}: ${e.message}`)
		.join("; ")
	return {
		success: false,
		error: errorMsg || "Validation failed",
	}
}

// ============================================================================
// Schema Introspection (for prompt arguments)
// ============================================================================

/**
 * Extract field info from a Zod object schema.
 * Works with Zod 4's internal structure.
 */
export const extractObjectFields = (
	schema: z.ZodType,
): Array<{ name: string; description?: string; required: boolean }> => {
	// Check if it's an object schema by trying to get shape
	if (!("shape" in schema) || typeof schema.shape !== "object") {
		return []
	}

	const shape = schema.shape as Record<string, z.ZodType>
	const fields: Array<{ name: string; description?: string; required: boolean }> = []

	for (const [key, value] of Object.entries(shape)) {
		const isOptional = value.isOptional?.() ?? false
		const hasDefault =
			"~standard" in value &&
			typeof value["~standard"] === "object" &&
			value["~standard"] !== null &&
			"default" in (value["~standard"] as object)

		fields.push({
			name: key,
			description: value.description,
			required: !isOptional && !hasDefault,
		})
	}

	return fields
}
