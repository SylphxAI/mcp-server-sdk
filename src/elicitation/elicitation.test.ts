import { describe, expect, test } from "bun:test"
import {
	createElicitationClient,
	elicitString,
	elicitNumber,
	elicitInteger,
	elicitBoolean,
	elicitEnum,
	elicitSchema,
} from "./index.js"

describe("Elicitation", () => {
	describe("createElicitationClient", () => {
		test("creates client with elicit method", async () => {
			const mockResult = {
				action: "accept" as const,
				content: { apiKey: "test-key" },
			}

			const client = createElicitationClient(async (method, params) => {
				expect(method).toBe("elicitation/create")
				expect(params.message).toBe("Enter API key")
				expect(params.requestedSchema.type).toBe("object")
				return mockResult
			})

			const result = await client.elicit("Enter API key", {
				type: "object",
				properties: {
					apiKey: { type: "string" },
				},
				required: ["apiKey"],
			})

			expect(result).toEqual(mockResult)
		})

		test("handles decline action", async () => {
			const client = createElicitationClient(async () => ({
				action: "decline" as const,
			}))

			const result = await client.elicit("Enter data", {
				type: "object",
				properties: {},
			})

			expect(result.action).toBe("decline")
			expect(result.content).toBeUndefined()
		})

		test("handles cancel action", async () => {
			const client = createElicitationClient(async () => ({
				action: "cancel" as const,
			}))

			const result = await client.elicit("Enter data", {
				type: "object",
				properties: {},
			})

			expect(result.action).toBe("cancel")
		})
	})

	describe("elicitString", () => {
		test("creates string property", () => {
			const prop = elicitString({ description: "Name" })

			expect(prop).toEqual({
				type: "string",
				description: "Name",
			})
		})

		test("creates string with format", () => {
			const prop = elicitString({ format: "email" })

			expect(prop.format).toBe("email")
		})

		test("creates string with length constraints", () => {
			const prop = elicitString({ minLength: 1, maxLength: 100 })

			expect(prop.minLength).toBe(1)
			expect(prop.maxLength).toBe(100)
		})
	})

	describe("elicitNumber", () => {
		test("creates number property", () => {
			const prop = elicitNumber({ description: "Age", minimum: 0, maximum: 150 })

			expect(prop).toEqual({
				type: "number",
				description: "Age",
				minimum: 0,
				maximum: 150,
			})
		})
	})

	describe("elicitInteger", () => {
		test("creates integer property", () => {
			const prop = elicitInteger({ default: 10 })

			expect(prop).toEqual({
				type: "integer",
				default: 10,
			})
		})
	})

	describe("elicitBoolean", () => {
		test("creates boolean property", () => {
			const prop = elicitBoolean({ description: "Confirm?", default: false })

			expect(prop).toEqual({
				type: "boolean",
				description: "Confirm?",
				default: false,
			})
		})
	})

	describe("elicitEnum", () => {
		test("creates string enum", () => {
			const prop = elicitEnum(["red", "green", "blue"], { description: "Color" })

			expect(prop.type).toBe("string")
			expect(prop.enum).toEqual(["red", "green", "blue"])
		})

		test("creates enum with names", () => {
			const prop = elicitEnum(["a", "b"], { enumNames: ["Option A", "Option B"] })

			expect(prop.enumNames).toEqual(["Option A", "Option B"])
		})
	})

	describe("elicitSchema", () => {
		test("creates schema from properties", () => {
			const schema = elicitSchema(
				{
					name: elicitString({ description: "Name" }),
					age: elicitNumber({ minimum: 0 }),
				},
				["name"],
			)

			expect(schema.type).toBe("object")
			expect(schema.properties.name.type).toBe("string")
			expect(schema.properties.age.type).toBe("number")
			expect(schema.required).toEqual(["name"])
		})

		test("creates schema without required", () => {
			const schema = elicitSchema({
				optional: elicitString(),
			})

			expect(schema.required).toBeUndefined()
		})
	})
})
