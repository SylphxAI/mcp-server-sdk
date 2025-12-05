/**
 * Schema Compatibility Layer
 *
 * Allows using both Zod and Vex schemas interchangeably.
 */

import type { Schema as VexSchema } from "@sylphx/vex"
import { safeParse } from "@sylphx/vex"
import { toJsonSchema as vexToJsonSchema } from "@sylphx/vex-json-schema"
import type { JsonSchema } from "../protocol/mcp.js"

// ============================================================================
// Types
// ============================================================================

/** Validation result */
export type ValidationResult<T> =
	| { readonly success: true; readonly data: T }
	| { readonly success: false; readonly error: string }

/** Generic schema type that works with both Zod and Vex */
export type AnySchema<T = unknown> = VexSchema<T> | ZodLike<T>

/** Zod-like schema interface */
interface ZodLike<T = unknown> {
	_def: unknown
	safeParse: (data: unknown) => { success: true; data: T } | { success: false; error: unknown }
}

// ============================================================================
// Detection
// ============================================================================

/** Check if schema is a Zod schema */
const isZodSchema = <T>(schema: unknown): schema is ZodLike<T> => {
	if (!schema || typeof schema !== "object") return false
	const s = schema as Record<string, unknown>
	// biome-ignore lint/complexity/useLiteralKeys: TypeScript requires bracket notation for index signatures
	return "_def" in s && typeof s["safeParse"] === "function"
}

// ============================================================================
// Unified Operations
// ============================================================================

/**
 * Validate input against any schema (Zod or Vex).
 */
export const validate = <T>(schema: AnySchema<T>, input: unknown): ValidationResult<T> => {
	if (isZodSchema<T>(schema)) {
		const result = schema.safeParse(input)
		if (result.success) {
			return { success: true, data: result.data }
		}
		// Extract error message from Zod error
		const err = result.error
		let message = "Validation failed"
		if (err && typeof err === "object") {
			const errObj = err as Record<string, unknown>
			// biome-ignore lint/complexity/useLiteralKeys: TypeScript requires bracket notation for index signatures
			if ("message" in errObj && typeof errObj["message"] === "string") {
				// biome-ignore lint/complexity/useLiteralKeys: TypeScript requires bracket notation for index signatures
				message = errObj["message"]
				// biome-ignore lint/complexity/useLiteralKeys: TypeScript requires bracket notation for index signatures
			} else if ("errors" in errObj && Array.isArray(errObj["errors"])) {
				// biome-ignore lint/complexity/useLiteralKeys: TypeScript requires bracket notation for index signatures
				message = (errObj["errors"] as Array<{ message?: string }>)
					.map((e) => e.message || "Unknown error")
					.join(", ")
				// biome-ignore lint/complexity/useLiteralKeys: TypeScript requires bracket notation for index signatures
			} else if ("issues" in errObj && Array.isArray(errObj["issues"])) {
				// biome-ignore lint/complexity/useLiteralKeys: TypeScript requires bracket notation for index signatures
				message = (errObj["issues"] as Array<{ message?: string; path?: unknown[] }>)
					.map((e) => {
						const path = e.path?.join(".") || ""
						return path ? `${path}: ${e.message}` : e.message || "Unknown error"
					})
					.join("; ")
			}
		}
		return { success: false, error: message }
	}

	// Vex schema
	const result = safeParse(schema as VexSchema<T>)(input)
	if (result.success) {
		return { success: true, data: result.data }
	}
	return { success: false, error: result.error || "Validation failed" }
}

/**
 * Convert any schema (Zod or Vex) to JSON Schema.
 */
export const toJsonSchema = <T>(schema: AnySchema<T>): JsonSchema => {
	if (isZodSchema(schema)) {
		// Zod v4+ has different API, check for zodToJsonSchema function
		try {
			// Dynamic require to avoid bundling
			const zodToJson =
				// biome-ignore lint/suspicious/noExplicitAny: globalThis typing
				(globalThis as any).require?.("zod-to-json-schema") ?? require("zod-to-json-schema")
			if (zodToJson?.zodToJsonSchema) {
				return zodToJson.zodToJsonSchema(schema, { $schema: false }) as JsonSchema
			}
		} catch {
			// Fall through to manual conversion
		}

		// Manual conversion for basic Zod schemas
		return convertZodToJsonSchema(schema)
	}

	// Vex schema
	return vexToJsonSchema(schema as VexSchema<unknown>, { $schema: false }) as JsonSchema
}

// ============================================================================
// Zod to JSON Schema (Manual fallback)
// ============================================================================

interface ZodDef {
	typeName?: string
	type?: unknown
	shape?: () => Record<string, unknown>
	innerType?: { _def: ZodDef }
	schema?: { _def: ZodDef }
	options?: Array<{ _def: ZodDef }>
	description?: string
	checks?: Array<{ kind: string; value?: unknown }>
	values?: unknown[]
	defaultValue?: () => unknown
}

// Mutable JSON Schema for building
type MutableJsonSchema = {
	type?: string
	properties?: Record<string, JsonSchema>
	required?: string[]
	items?: JsonSchema
	anyOf?: JsonSchema[]
	enum?: unknown[]
	description?: string
}

const convertZodToJsonSchema = (schema: ZodLike): JsonSchema => {
	const def = schema._def as ZodDef
	const description = def.description
	const typeName = def.typeName || ""

	if (typeName === "ZodString") {
		return description ? { type: "string", description } : { type: "string" }
	}

	if (typeName === "ZodNumber") {
		return description ? { type: "number", description } : { type: "number" }
	}

	if (typeName === "ZodBoolean") {
		return description ? { type: "boolean", description } : { type: "boolean" }
	}

	if (typeName === "ZodArray") {
		const innerSchema = def.type as ZodLike | undefined
		const items = innerSchema ? convertZodToJsonSchema(innerSchema) : {}
		return description ? { type: "array", items, description } : { type: "array", items }
	}

	if (typeName === "ZodObject") {
		const shape = typeof def.shape === "function" ? def.shape() : {}
		const properties: Record<string, JsonSchema> = {}
		const required: string[] = []

		for (const [key, value] of Object.entries(shape)) {
			const propSchema = value as ZodLike
			const propDef = propSchema._def as ZodDef

			const isOptional =
				propDef.typeName === "ZodOptional" ||
				propDef.typeName === "ZodNullable" ||
				propDef.typeName === "ZodDefault"

			if (!isOptional) {
				required.push(key)
			}

			let innerSchema = propSchema
			if (propDef.innerType) {
				innerSchema = propDef.innerType as ZodLike
			}

			properties[key] = convertZodToJsonSchema(innerSchema)
		}

		const result: MutableJsonSchema = { type: "object", properties }
		if (required.length > 0) result.required = required
		if (description) result.description = description
		return result as JsonSchema
	}

	if (typeName === "ZodOptional" || typeName === "ZodNullable" || typeName === "ZodDefault") {
		const inner = def.innerType as ZodLike | undefined
		return inner ? convertZodToJsonSchema(inner) : ({} as JsonSchema)
	}

	if (typeName === "ZodUnion") {
		const options = def.options as ZodLike[] | undefined
		if (options) {
			const anyOf = options.map((opt) => convertZodToJsonSchema(opt))
			return description ? ({ anyOf, description } as JsonSchema) : ({ anyOf } as JsonSchema)
		}
	}

	if (typeName === "ZodEnum" || typeName === "ZodNativeEnum") {
		const values = def.values as unknown[] | undefined
		if (values) {
			return description
				? ({ type: "string", enum: values, description } as JsonSchema)
				: ({ type: "string", enum: values } as JsonSchema)
		}
	}

	// Default fallback
	return description ? { type: "object", description } : { type: "object" }
}

// ============================================================================
// Type Inference
// ============================================================================

/** Infer the output type from any schema */
export type InferSchema<T extends AnySchema> =
	T extends VexSchema<infer U> ? U : T extends ZodLike<infer U> ? U : unknown
