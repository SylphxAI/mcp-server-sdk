import { describe, expect, test } from "bun:test"
import { createElicitationClient } from "./index.js"

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
})
