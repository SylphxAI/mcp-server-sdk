/**
 * Elicitation Types
 *
 * Server-initiated requests for user input through the client.
 */

// ============================================================================
// Elicitation Request/Response
// ============================================================================

/**
 * Schema for elicitation - only flat primitive types allowed.
 */
export interface ElicitationSchema {
	readonly type: "object"
	readonly properties: Record<string, ElicitationProperty>
	readonly required?: readonly string[]
}

export interface ElicitationProperty {
	readonly type: "string" | "number" | "integer" | "boolean"
	readonly description?: string
	readonly default?: string | number | boolean
	readonly enum?: readonly (string | number)[]
	readonly enumNames?: readonly string[]
	// String-specific
	readonly format?: "email" | "uri" | "date" | "date-time"
	readonly minLength?: number
	readonly maxLength?: number
	// Number-specific
	readonly minimum?: number
	readonly maximum?: number
}

export interface ElicitationCreateParams {
	/** Message to display to the user */
	readonly message: string
	/** JSON Schema defining expected response structure */
	readonly requestedSchema: ElicitationSchema
}

export type ElicitationAction = "accept" | "decline" | "cancel"

export interface ElicitationCreateResult {
	/** User's action */
	readonly action: ElicitationAction
	/** User's response data (only when action is "accept") */
	readonly content?: Record<string, unknown>
}

// ============================================================================
// Elicitation Client
// ============================================================================

export type ElicitationRequestSender = (
	method: string,
	params: ElicitationCreateParams
) => Promise<ElicitationCreateResult>

export interface ElicitationClient {
	/** Request user input through the client */
	readonly elicit: (message: string, schema: ElicitationSchema) => Promise<ElicitationCreateResult>
}

export interface ElicitationContext {
	/** Request user input (if client supports elicitation) */
	readonly elicit?: ElicitationClient["elicit"]
}
