import { describe, expect, test } from "bun:test"
import { createSamplingClient } from "./index.js"

describe("Sampling", () => {
	describe("createSamplingClient", () => {
		test("creates client with createMessage method", async () => {
			const mockResult = {
				role: "assistant" as const,
				content: { type: "text" as const, text: "Hello!" },
				model: "claude-3",
				stopReason: "endTurn" as const,
			}

			const client = createSamplingClient(async (method, params) => {
				expect(method).toBe("sampling/createMessage")
				expect(params).toEqual({
					messages: [{ role: "user", content: { type: "text", text: "Hi" } }],
					maxTokens: 100,
				})
				return mockResult
			})

			const result = await client.createMessage({
				messages: [{ role: "user", content: { type: "text", text: "Hi" } }],
				maxTokens: 100,
			})

			expect(result).toEqual(mockResult)
		})

		test("passes all parameters", async () => {
			let receivedParams: unknown

			const client = createSamplingClient(async (_, params) => {
				receivedParams = params
				return {
					role: "assistant" as const,
					content: { type: "text" as const, text: "" },
					model: "test",
				}
			})

			await client.createMessage({
				messages: [{ role: "user", content: { type: "text", text: "test" } }],
				maxTokens: 100,
				temperature: 0.5,
				systemPrompt: "You are helpful",
				stopSequences: ["END"],
				modelPreferences: {
					hints: [{ name: "claude-3" }],
					costPriority: 0.5,
				},
			})

			expect(receivedParams).toMatchObject({
				maxTokens: 100,
				temperature: 0.5,
				systemPrompt: "You are helpful",
				stopSequences: ["END"],
			})
		})
	})
})
