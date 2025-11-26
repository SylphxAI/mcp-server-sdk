/**
 * Completions Types
 *
 * Types for auto-complete functionality.
 */

import type * as Mcp from "../protocol/mcp.js"

// ============================================================================
// Completion Provider
// ============================================================================

/**
 * Completion provider function.
 * Returns completion suggestions for a given argument value.
 */
export type CompletionProvider = (
	argumentName: string,
	argumentValue: string,
) => CompletionResult | Promise<CompletionResult>

/**
 * Completion result.
 */
export interface CompletionResult {
	readonly values: readonly string[]
	readonly total?: number
	readonly hasMore?: boolean
}

// ============================================================================
// Completion Config
// ============================================================================

/**
 * Completion configuration for a prompt.
 */
export interface PromptCompletionConfig {
	readonly type: "prompt"
	readonly name: string
	readonly provider: CompletionProvider
}

/**
 * Completion configuration for a resource.
 */
export interface ResourceCompletionConfig {
	readonly type: "resource"
	readonly uriTemplate: string
	readonly provider: CompletionProvider
}

export type CompletionConfig = PromptCompletionConfig | ResourceCompletionConfig
