/**
 * Sampling Types
 *
 * Types for server-to-client LLM sampling requests.
 */

import type * as Mcp from "../protocol/mcp.js"

// ============================================================================
// Sampling Client Interface
// ============================================================================

/**
 * Interface for making sampling requests to the client.
 * The client must have sampling capability enabled.
 */
export interface SamplingClient {
	/**
	 * Request the client to sample from an LLM.
	 *
	 * @example
	 * ```ts
	 * const result = await sampling.createMessage({
	 *   messages: [{ role: "user", content: { type: "text", text: "Hello" } }],
	 *   maxTokens: 100,
	 * })
	 * console.log(result.content)
	 * ```
	 */
	readonly createMessage: (params: Mcp.SamplingCreateParams) => Promise<Mcp.SamplingCreateResult>
}

/**
 * Raw request sender for sampling.
 * Transports implement this to send JSON-RPC requests to the client.
 */
export type SamplingRequestSender = (method: string, params: unknown) => Promise<unknown>

// ============================================================================
// Sampling Context Extension
// ============================================================================

/**
 * Context extension for handlers that can make sampling requests.
 */
export interface SamplingContext {
	readonly sampling?: SamplingClient
}
