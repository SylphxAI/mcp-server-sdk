/**
 * HTTP Transport Authentication
 *
 * Optional, opt-in authentication for the HTTP / Streamable HTTP transports.
 *
 * The HTTP transport is UNAUTHENTICATED by default. When `apiKey` or
 * `authenticate` is supplied, MCP JSON-RPC requests are gated; the health
 * endpoint and CORS preflight are always left open.
 *
 * This module is the single source of truth for auth logic shared by every
 * HTTP entrypoint (`http()`, `serve()`, and the `fetch` adapter).
 */

import { createHash, timingSafeEqual } from "node:crypto"

// ============================================================================
// Options
// ============================================================================

/**
 * Authentication options for the HTTP transport(s).
 *
 * Precedence: if `authenticate` is provided it is used; otherwise if `apiKey`
 * is provided the `X-API-Key` header is checked; otherwise requests are
 * unauthenticated (the default).
 */
export interface AuthOptions {
	/**
	 * Require a matching `X-API-Key` header on every MCP request.
	 *
	 * The comparison is constant-time (both sides are SHA-256 hashed and
	 * compared with `timingSafeEqual`). A missing or empty header is rejected.
	 *
	 * Ignored when {@link AuthOptions.authenticate} is also provided.
	 */
	readonly apiKey?: string

	/**
	 * Custom authentication hook for Bearer / OAuth / arbitrary schemes.
	 *
	 * Receives the incoming request as exposed by the active entrypoint:
	 * a gust request context (which carries `.headers`) for `http()` / `serve()`,
	 * or a Fetch {@link Request} for the `fetch` adapter. Return `false` (or a
	 * promise resolving to `false`) to reject the request with `401`.
	 *
	 * When provided, this takes precedence over {@link AuthOptions.apiKey}.
	 */
	readonly authenticate?: (request: AuthRequest) => boolean | Promise<boolean>
}

/**
 * The request shape passed to {@link AuthOptions.authenticate}.
 *
 * - `http()` / `serve()` pass the gust request context.
 * - The `fetch` adapter passes the Fetch {@link Request}.
 *
 * Both expose request headers, which {@link readHeader} can read uniformly.
 */
export type AuthRequest = HeaderCarrier | Request

/** Minimal structural type for anything that exposes request headers. */
export interface HeaderCarrier {
	readonly headers: Headers | Readonly<Record<string, string | string[] | undefined>>
}

// ============================================================================
// Header Access
// ============================================================================

/**
 * Read a single header value from any supported request shape.
 *
 * Handles Fetch `Headers`, plain header records, and the `string | string[] |
 * undefined` values that Node's `IncomingMessage.headers` can produce (the
 * first entry of an array is used). Header lookup is case-insensitive.
 */
export const readHeader = (request: AuthRequest, name: string): string | undefined => {
	const headers = (request as HeaderCarrier).headers
	if (headers instanceof Headers) {
		return headers.get(name) ?? undefined
	}
	const lower = name.toLowerCase()
	const record = headers as Readonly<Record<string, string | string[] | undefined>>
	const value = record[lower] ?? record[name]
	if (Array.isArray(value)) return value[0]
	return value ?? undefined
}

// ============================================================================
// Constant-Time API Key Comparison
// ============================================================================

const sha256 = (value: string): Buffer => createHash("sha256").update(value, "utf8").digest()

/**
 * Constant-time comparison of a provided API key against the expected value.
 *
 * Both sides are SHA-256 hashed first so the buffers always share a length
 * (`timingSafeEqual` throws on length mismatch) and the raw key bytes never
 * drive timing. An empty or missing provided key is always rejected.
 */
export const verifyApiKey = (expected: string, provided: string | undefined): boolean => {
	if (!provided) return false
	return timingSafeEqual(sha256(expected), sha256(provided))
}

// ============================================================================
// Authorization Decision
// ============================================================================

/**
 * Decide whether a request is authorized.
 *
 * Precedence: `authenticate` hook > `apiKey` check > allow (unauthenticated).
 */
export const isAuthorized = async (
	options: AuthOptions,
	request: AuthRequest,
): Promise<boolean> => {
	if (options.authenticate) {
		return options.authenticate(request)
	}
	if (options.apiKey !== undefined) {
		return verifyApiKey(options.apiKey, readHeader(request, "x-api-key"))
	}
	return true
}

/** True when any authentication is configured. */
export const isAuthEnabled = (options: AuthOptions): boolean =>
	options.authenticate !== undefined || options.apiKey !== undefined

// ============================================================================
// 401 Response Payload
// ============================================================================

/** JSON-RPC error code for an unauthorized request (server-defined range). */
export const UNAUTHORIZED_CODE = -32001

/** Canonical JSON-RPC error body returned on auth failure. */
export const unauthorizedBody = (): {
	jsonrpc: "2.0"
	id: null
	error: { code: number; message: string }
} => ({
	jsonrpc: "2.0",
	id: null,
	error: { code: UNAUTHORIZED_CODE, message: "Unauthorized" },
})

/** Header advertised on a 401 so clients know how to authenticate. */
export const WWW_AUTHENTICATE = 'Bearer realm="mcp", charset="UTF-8"'
