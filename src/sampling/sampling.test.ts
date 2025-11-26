import { describe, expect, test } from "bun:test"
import { createSamplingClient, modelPreferences, samplingImage, samplingText } from "./index.js"

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
				messages: [samplingText("user", "Hi")],
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
				messages: [samplingText("user", "test")],
				maxTokens: 100,
				temperature: 0.5,
				systemPrompt: "You are helpful",
				stopSequences: ["END"],
				modelPreferences: modelPreferences({
					hints: ["claude-3"],
					costPriority: 0.5,
				}),
			})

			expect(receivedParams).toMatchObject({
				maxTokens: 100,
				temperature: 0.5,
				systemPrompt: "You are helpful",
				stopSequences: ["END"],
			})
		})
	})

	describe("samplingText", () => {
		test("creates user text message", () => {
			const msg = samplingText("user", "Hello")

			expect(msg).toEqual({
				role: "user",
				content: { type: "text", text: "Hello" },
			})
		})

		test("creates assistant text message", () => {
			const msg = samplingText("assistant", "Hi there")

			expect(msg).toEqual({
				role: "assistant",
				content: { type: "text", text: "Hi there" },
			})
		})
	})

	describe("samplingImage", () => {
		test("creates image message", () => {
			const msg = samplingImage("user", "base64data", "image/png")

			expect(msg).toEqual({
				role: "user",
				content: {
					type: "image",
					data: "base64data",
					mimeType: "image/png",
				},
			})
		})
	})

	describe("modelPreferences", () => {
		test("creates preferences with hints", () => {
			const prefs = modelPreferences({
				hints: ["claude-3", "gpt-4"],
			})

			expect(prefs.hints).toEqual([{ name: "claude-3" }, { name: "gpt-4" }])
		})

		test("creates preferences with priorities", () => {
			const prefs = modelPreferences({
				costPriority: 0.2,
				speedPriority: 0.3,
				intelligencePriority: 0.5,
			})

			expect(prefs).toEqual({
				hints: undefined,
				costPriority: 0.2,
				speedPriority: 0.3,
				intelligencePriority: 0.5,
			})
		})

		test("handles empty options", () => {
			const prefs = modelPreferences({})

			expect(prefs.hints).toBeUndefined()
			expect(prefs.costPriority).toBeUndefined()
		})
	})
})
