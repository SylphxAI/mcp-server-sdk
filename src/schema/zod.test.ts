import { describe, expect, test } from "bun:test"
import { z } from "zod"
import { extractObjectFields, isZodSchema, toJsonSchema, validate, zodToJsonSchema } from "./zod.js"

describe("Zod Schema Integration", () => {
	describe("zodToJsonSchema", () => {
		test("converts string schema", () => {
			const schema = z.string()
			const json = zodToJsonSchema(schema)
			expect(json.type).toBe("string")
		})

		test("converts string with constraints", () => {
			const schema = z.string().min(1).max(100)
			const json = zodToJsonSchema(schema)
			expect(json.type).toBe("string")
			expect(json.minLength).toBe(1)
			expect(json.maxLength).toBe(100)
		})

		test("converts number schema", () => {
			const schema = z.number()
			const json = zodToJsonSchema(schema)
			expect(json.type).toBe("number")
		})

		test("converts number with constraints", () => {
			const schema = z.number().int().min(0).max(100)
			const json = zodToJsonSchema(schema)
			expect(json.type).toBe("integer")
			expect(json.minimum).toBe(0)
			expect(json.maximum).toBe(100)
		})

		test("converts boolean schema", () => {
			const schema = z.boolean()
			const json = zodToJsonSchema(schema)
			expect(json.type).toBe("boolean")
		})

		test("converts array schema", () => {
			const schema = z.array(z.string())
			const json = zodToJsonSchema(schema)
			expect(json.type).toBe("array")
			expect(json.items).toBeDefined()
		})

		test("converts object schema", () => {
			const schema = z.object({
				name: z.string(),
				age: z.number(),
			})
			const json = zodToJsonSchema(schema)
			expect(json.type).toBe("object")
			expect(json.properties).toHaveProperty("name")
			expect(json.properties).toHaveProperty("age")
			expect(json.required).toContain("name")
			expect(json.required).toContain("age")
		})

		test("handles optional fields", () => {
			const schema = z.object({
				name: z.string(),
				nickname: z.string().optional(),
			})
			const json = zodToJsonSchema(schema)
			expect(json.required).toContain("name")
			expect(json.required).not.toContain("nickname")
		})

		test("converts enum schema", () => {
			const schema = z.enum(["a", "b", "c"])
			const json = zodToJsonSchema(schema)
			expect(json.enum).toEqual(["a", "b", "c"])
		})

		test("converts union schema", () => {
			const schema = z.union([z.string(), z.number()])
			const json = zodToJsonSchema(schema)
			// Zod 4 may use anyOf or oneOf
			expect(json.anyOf || json.oneOf).toBeDefined()
		})
	})

	describe("isZodSchema", () => {
		test("returns true for Zod schemas", () => {
			expect(isZodSchema(z.string())).toBe(true)
			expect(isZodSchema(z.object({}))).toBe(true)
			expect(isZodSchema(z.array(z.string()))).toBe(true)
		})

		test("returns false for non-Zod values", () => {
			expect(isZodSchema(null)).toBe(false)
			expect(isZodSchema(undefined)).toBe(false)
			expect(isZodSchema({})).toBe(false)
			expect(isZodSchema({ type: "string" })).toBe(false)
			expect(isZodSchema("string")).toBe(false)
		})
	})

	describe("toJsonSchema", () => {
		test("converts Zod schema", () => {
			const schema = z.string()
			const json = toJsonSchema(schema)
			expect(json.type).toBe("string")
		})

		test("passes through JSON Schema", () => {
			const jsonSchema = { type: "string", minLength: 1 }
			expect(toJsonSchema(jsonSchema)).toEqual(jsonSchema)
		})
	})

	describe("validate", () => {
		test("validates valid input", () => {
			const schema = z.object({
				name: z.string(),
				age: z.number(),
			})
			const result = validate(schema, { name: "John", age: 30 })
			expect(result.success).toBe(true)
			if (result.success) {
				expect(result.data).toEqual({ name: "John", age: 30 })
			}
		})

		test("returns error for invalid input", () => {
			const schema = z.object({
				name: z.string(),
				age: z.number(),
			})
			const result = validate(schema, { name: "John", age: "thirty" })
			expect(result.success).toBe(false)
			if (!result.success) {
				expect(result.error.length).toBeGreaterThan(0)
			}
		})

		test("applies coercion", () => {
			const schema = z.object({
				count: z.coerce.number(),
			})
			const result = validate(schema, { count: "42" })
			expect(result.success).toBe(true)
			if (result.success) {
				expect(result.data.count).toBe(42)
			}
		})

		test("applies defaults", () => {
			const schema = z.object({
				name: z.string(),
				active: z.boolean().default(true),
			})
			const result = validate(schema, { name: "Test" })
			expect(result.success).toBe(true)
			if (result.success) {
				expect(result.data.active).toBe(true)
			}
		})
	})

	describe("extractObjectFields", () => {
		test("extracts fields from object schema", () => {
			const schema = z.object({
				name: z.string(),
				age: z.number().optional(),
			})
			const fields = extractObjectFields(schema)
			expect(fields).toHaveLength(2)

			const nameField = fields.find((f) => f.name === "name")
			expect(nameField?.required).toBe(true)

			const ageField = fields.find((f) => f.name === "age")
			expect(ageField?.required).toBe(false)
		})

		test("includes description", () => {
			const schema = z.object({
				name: z.string().describe("The name"),
			})
			const fields = extractObjectFields(schema)
			expect(fields[0]?.description).toBe("The name")
		})

		test("returns empty array for non-object", () => {
			const schema = z.string()
			const fields = extractObjectFields(schema)
			expect(fields).toEqual([])
		})
	})
})
