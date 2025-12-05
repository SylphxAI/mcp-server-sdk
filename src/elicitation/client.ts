/**
 * Elicitation Client
 *
 * Server-side client for requesting user input through the MCP client.
 */

import type {
	ElicitationClient,
	ElicitationCreateParams,
	ElicitationCreateResult,
	ElicitationRequestSender,
	ElicitationSchema,
} from "./types.js"

/**
 * Create an elicitation client for requesting user input.
 *
 * @example
 * ```ts
 * const result = await ctx.elicit?.(
 *   "Please provide your API key",
 *   {
 *     type: "object",
 *     properties: {
 *       apiKey: { type: "string", description: "Your API key" },
 *     },
 *     required: ["apiKey"],
 *   }
 * )
 *
 * if (result?.action === "accept") {
 *   const apiKey = result.content?.apiKey as string
 * }
 * ```
 */
export const createElicitationClient = (sender: ElicitationRequestSender): ElicitationClient => {
	const elicit = async (
		message: string,
		schema: ElicitationSchema,
	): Promise<ElicitationCreateResult> => {
		const params: ElicitationCreateParams = {
			message,
			requestedSchema: schema,
		}
		return sender("elicitation/create", params)
	}

	return { elicit }
}
