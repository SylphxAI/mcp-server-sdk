/**
 * Middleware Types
 *
 * Middleware wraps handlers to add cross-cutting concerns.
 */

import type { ToolsCallResult } from "../protocol/mcp.js"
import type { ToolContext } from "../builders/tool.js"

// ============================================================================
// Core Types
// ============================================================================

/** Information about the current request */
export interface RequestInfo {
	readonly type: "tool" | "resource" | "prompt"
	readonly name: string
	readonly input: unknown
	readonly startTime: number
}

/** Next function to call the next middleware or handler */
export type Next<TResult> = () => Promise<TResult>

/** Middleware function signature */
export type Middleware<TContext, TResult> = (
	ctx: TContext,
	info: RequestInfo,
	next: Next<TResult>,
) => Promise<TResult>

// ============================================================================
// Specialized Middleware Types
// ============================================================================

/** Tool middleware */
export type ToolMiddleware<TContext extends ToolContext = ToolContext> = Middleware<
	TContext,
	ToolsCallResult
>

/** Generic middleware that works with any handler type */
export type AnyMiddleware<TContext = unknown> = Middleware<TContext, unknown>

// ============================================================================
// Middleware Stack
// ============================================================================

/** Composable middleware stack */
export interface MiddlewareStack<TContext, TResult> {
	readonly middlewares: readonly Middleware<TContext, TResult>[]
	readonly use: (mw: Middleware<TContext, TResult>) => MiddlewareStack<TContext, TResult>
	readonly execute: (ctx: TContext, info: RequestInfo, handler: Next<TResult>) => Promise<TResult>
}
