import { describe, expect, test } from "bun:test"
import {
	array,
	bool,
	coerceNumber,
	description,
	enum_,
	gte,
	int,
	lte,
	max,
	min,
	num,
	object,
	optional,
	str,
	union,
	withDefault,
} from "@sylphx/vex"
import { extractObjectFields, validate, vexToJsonSchema } from "./vex.js"

describe("Vex Schema Integration", () => {
	describe("vexToJsonSchema", () => {
		test("converts string schema", () => {
			const schema = str()
			const json = vexToJsonSchema(schema)
			expect(json.type).toBe("string")
		})

		test("converts string with constraints", () => {
			const schema = str(min(1), max(100))
			const json = vexToJsonSchema(schema)
			expect(json.type).toBe("string")
			expect(json.minLength).toBe(1)
			expect(json.maxLength).toBe(100)
		})

		test("converts number schema", () => {
			const schema = num()
			const json = vexToJsonSchema(schema)
			expect(json.type).toBe("number")
		})

		test("converts number with constraints", () => {
			const schema = num(int, gte(0), lte(100))
			const json = vexToJsonSchema(schema)
			expect(json.type).toBe("integer")
			expect(json.minimum).toBe(0)
			expect(json.maximum).toBe(100)
		})

		test("converts boolean schema", () => {
			const schema = bool()
			const json = vexToJsonSchema(schema)
			expect(json.type).toBe("boolean")
		})

		test("converts array schema", () => {
			const schema = array(str())
			const json = vexToJsonSchema(schema)
			expect(json.type).toBe("array")
			expect(json.items).toBeDefined()
		})

		test("converts object schema", () => {
			const schema = object({
				name: str(),
				age: num(),
			})
			const json = vexToJsonSchema(schema)
			expect(json.type).toBe("object")
			expect(json.properties).toHaveProperty("name")
			expect(json.properties).toHaveProperty("age")
			expect(json.required).toContain("name")
			expect(json.required).toContain("age")
		})

		test("handles optional fields", () => {
			const schema = object({
				name: str(),
				nickname: optional(str()),
			})
			const json = vexToJsonSchema(schema)
			expect(json.required).toContain("name")
			expect(json.required).not.toContain("nickname")
		})

		test("converts enum schema", () => {
			const schema = enum_(["a", "b", "c"] as const)
			const json = vexToJsonSchema(schema)
			expect(json.enum).toEqual(["a", "b", "c"])
		})

		test("converts union schema", () => {
			const schema = union(str(), num())
			const json = vexToJsonSchema(schema)
			expect(json.anyOf || json.oneOf).toBeDefined()
		})
	})

	describe("validate", () => {
		test("validates valid input", () => {
			const schema = object({
				name: str(),
				age: num(),
			})
			const result = validate(schema, { name: "John", age: 30 })
			expect(result.success).toBe(true)
			if (result.success) {
				expect(result.data).toEqual({ name: "John", age: 30 })
			}
		})

		test("returns error for invalid input", () => {
			const schema = object({
				name: str(),
				age: num(),
			})
			const result = validate(schema, { name: "John", age: "thirty" })
			expect(result.success).toBe(false)
			if (!result.success) {
				expect(result.error.length).toBeGreaterThan(0)
			}
		})

		test("applies coercion", () => {
			const schema = object({
				count: coerceNumber,
			})
			const result = validate(schema, { count: "42" })
			expect(result.success).toBe(true)
			if (result.success) {
				expect(result.data.count).toBe(42)
			}
		})

		test("applies defaults", () => {
			const schema = object({
				name: str(),
				active: withDefault(bool(), true),
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
			const schema = object({
				name: str(),
				age: optional(num()),
			})
			const fields = extractObjectFields(schema)
			expect(fields).toHaveLength(2)

			const nameField = fields.find((f) => f.name === "name")
			expect(nameField?.required).toBe(true)

			const ageField = fields.find((f) => f.name === "age")
			expect(ageField?.required).toBe(false)
		})

		test("includes description", () => {
			const schema = object({
				name: str(description("The name")),
			})
			const fields = extractObjectFields(schema)
			expect(fields[0]?.description).toBe("The name")
		})

		test("returns empty array for non-object", () => {
			const schema = str()
			const fields = extractObjectFields(schema)
			expect(fields).toEqual([])
		})
	})
})
