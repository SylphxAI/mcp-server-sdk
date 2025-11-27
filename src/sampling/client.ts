/**
 * Sampling Client
 *
 * Factory for creating sampling clients from request senders.
 */

import * as Mcp from "../protocol/mcp.js"
import type { SamplingClient, SamplingRequestSender } from "./types.js"

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
