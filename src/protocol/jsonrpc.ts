/**
 * JSON-RPC 2.0 Protocol Types
 * Pure type definitions - no runtime dependencies
 */

export const JSONRPC_VERSION = "2.0" as const

export type RequestId = string | number

// ============================================================================
// Base Message Types
// ============================================================================

export interface JsonRpcRequest<M extends string = string, P = unknown> {
	readonly jsonrpc: typeof JSONRPC_VERSION
	readonly id: RequestId
	readonly method: M
	readonly params?: P
}

export interface JsonRpcNotification<M extends string = string, P = unknown> {
	readonly jsonrpc: typeof JSONRPC_VERSION
	readonly method: M
	readonly params?: P
}

export interface JsonRpcSuccess<R = unknown> {
	readonly jsonrpc: typeof JSONRPC_VERSION
	readonly id: RequestId
	readonly result: R
}

export interface JsonRpcError<D = unknown> {
	readonly jsonrpc: typeof JSONRPC_VERSION
	readonly id: RequestId | null
	readonly error: {
		readonly code: number
		readonly message: string
		readonly data?: D
	}
}

export type JsonRpcResponse<R = unknown, D = unknown> = JsonRpcSuccess<R> | JsonRpcError<D>

export type JsonRpcMessage = JsonRpcRequest | JsonRpcNotification | JsonRpcResponse

// ============================================================================
// Standard Error Codes
// ============================================================================

export const ErrorCode = {
	// JSON-RPC 2.0 standard errors
	ParseError: -32700,
	InvalidRequest: -32600,
	MethodNotFound: -32601,
	InvalidParams: -32602,
	InternalError: -32603,
} as const

export type ErrorCode = (typeof ErrorCode)[keyof typeof ErrorCode]

// ============================================================================
// Type Guards (Pure Functions)
// ============================================================================

export const isRequest = (msg: JsonRpcMessage): msg is JsonRpcRequest =>
	"id" in msg && "method" in msg

export const isNotification = (msg: JsonRpcMessage): msg is JsonRpcNotification =>
	!("id" in msg) && "method" in msg

export const isResponse = (msg: JsonRpcMessage): msg is JsonRpcResponse =>
	"id" in msg && ("result" in msg || "error" in msg)

export const isSuccess = <R>(msg: JsonRpcResponse<R>): msg is JsonRpcSuccess<R> => "result" in msg

export const isError = <D>(msg: JsonRpcResponse<unknown, D>): msg is JsonRpcError<D> =>
	"error" in msg

// ============================================================================
// Constructors (Pure Functions)
// ============================================================================

export const request = <M extends string, P>(
	id: RequestId,
	method: M,
	params?: P
): JsonRpcRequest<M, P> => ({
	jsonrpc: JSONRPC_VERSION,
	id,
	method,
	...(params !== undefined && { params }),
})

export const notification = <M extends string, P>(
	method: M,
	params?: P
): JsonRpcNotification<M, P> => ({
	jsonrpc: JSONRPC_VERSION,
	method,
	...(params !== undefined && { params }),
})

export const success = <R>(id: RequestId, result: R): JsonRpcSuccess<R> => ({
	jsonrpc: JSONRPC_VERSION,
	id,
	result,
})

export const error = <D>(
	id: RequestId | null,
	code: number,
	message: string,
	data?: D
): JsonRpcError<D> => ({
	jsonrpc: JSONRPC_VERSION,
	id,
	error: {
		code,
		message,
		...(data !== undefined && { data }),
	},
})

// ============================================================================
// Parse (Pure Function)
// ============================================================================

export type ParseResult<T> =
	| { readonly ok: true; readonly value: T }
	| { readonly ok: false; readonly error: string }

export const parseMessage = (input: string): ParseResult<JsonRpcMessage> => {
	try {
		const data = JSON.parse(input)
		if (typeof data !== "object" || data === null) {
			return { ok: false, error: "Message must be an object" }
		}
		if (data.jsonrpc !== JSONRPC_VERSION) {
			return { ok: false, error: `Invalid jsonrpc version: ${data.jsonrpc}` }
		}
		return { ok: true, value: data as JsonRpcMessage }
	} catch (e) {
		return { ok: false, error: `JSON parse error: ${e}` }
	}
}

export const stringify = (msg: JsonRpcMessage): string => JSON.stringify(msg)
