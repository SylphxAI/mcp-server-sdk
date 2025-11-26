/**
 * Sampling Client
 *
 * Factory for creating sampling clients from request senders.
 */

import * as Mcp from "../protocol/mcp.js"
import type { SamplingClient, SamplingRequestSender } from "./types.js"

// ============================================================================
// Sampling Client Factory
// ============================================================================

/**
 * Create a sampling client from a request sender.
 *
 * @example
 * ```ts
 * const sampling = createSamplingClient(async (method, params) => {
 *   return await transport.request(method, params)
 * })
 *
 * const result = await sampling.createMessage({
 *   messages: [{ role: "user", content: { type: "text", text: "Hello" } }],
 *   maxTokens: 100,
 * })
 * ```
 */
export const createSamplingClient = (send: SamplingRequestSender): SamplingClient => ({
	createMessage: async (params) => {
		const result = await send(Mcp.Method.SamplingCreateMessage, params)
		return result as Mcp.SamplingCreateResult
	},
})

// ============================================================================
// Helpers
// ============================================================================

/**
 * Create a sampling message with text content.
 */
export const samplingText = (role: "user" | "assistant", text: string): Mcp.SamplingMessage => ({
	role,
	content: { type: "text", text },
})

/**
 * Create a sampling message with image content.
 */
export const samplingImage = (
	role: "user" | "assistant",
	data: string,
	mimeType: string
): Mcp.SamplingMessage => ({
	role,
	content: { type: "image", data, mimeType },
})

/**
 * Create model preferences.
 */
export const modelPreferences = (options: {
	hints?: readonly string[]
	costPriority?: number
	speedPriority?: number
	intelligencePriority?: number
}): Mcp.ModelPreferences => ({
	hints: options.hints?.map((name) => ({ name })),
	costPriority: options.costPriority,
	speedPriority: options.speedPriority,
	intelligencePriority: options.intelligencePriority,
})
