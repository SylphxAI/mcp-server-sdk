/**
 * Middleware Composition
 *
 * Pure functions for composing middleware.
 */

import type { Middleware, MiddlewareStack, Next, RequestInfo } from "./types.js"

// ============================================================================
// Compose Function
// ============================================================================

/**
 * Compose multiple middleware into a single function.
 * Executes in order: first middleware wraps second, etc.
 *
 * @example
 * ```ts
 * const composed = compose(loggingMiddleware, authMiddleware)
 * await composed(ctx, info, handler)
 * ```
 */
export const compose = <TContext, TResult>(
	...middlewares: Middleware<TContext, TResult>[]
): Middleware<TContext, TResult> => {
	if (middlewares.length === 0) {
		return (_ctx, _info, next) => next()
	}

	return (ctx, info, next) => {
		let index = -1

		const dispatch = (i: number): Promise<TResult> => {
			if (i <= index) {
				return Promise.reject(new Error("next() called multiple times"))
			}
			index = i

			const middleware = middlewares[i]
			if (!middleware) {
				return next()
			}

			return middleware(ctx, info, () => dispatch(i + 1))
		}

		return dispatch(0)
	}
}

// ============================================================================
// Middleware Stack Builder
// ============================================================================

/**
 * Create a middleware stack.
 * Immutable - each `use` returns a new stack.
 *
 * @example
 * ```ts
 * const stack = createStack<MyContext, MyResult>()
 *   .use(loggingMiddleware)
 *   .use(authMiddleware)
 *
 * await stack.execute(ctx, info, handler)
 * ```
 */
export const createStack = <TContext, TResult>(
	initial: readonly Middleware<TContext, TResult>[] = [],
): MiddlewareStack<TContext, TResult> => ({
	middlewares: initial,

	use(mw) {
		return createStack([...this.middlewares, mw])
	},

	async execute(ctx, info, handler) {
		const composed = compose(...this.middlewares)
		return composed(ctx, info, handler)
	},
})

// ============================================================================
// Utility: Conditional Middleware
// ============================================================================

/**
 * Apply middleware only when condition is true.
 */
export const when = <TContext, TResult>(
	predicate: (info: RequestInfo) => boolean,
	middleware: Middleware<TContext, TResult>,
): Middleware<TContext, TResult> => {
	return (ctx, info, next) => {
		if (predicate(info)) {
			return middleware(ctx, info, next)
		}
		return next()
	}
}

/**
 * Apply middleware only for specific request types.
 */
export const forType = <TContext, TResult>(
	type: "tool" | "resource" | "prompt",
	middleware: Middleware<TContext, TResult>,
): Middleware<TContext, TResult> => when((info) => info.type === type, middleware)

/**
 * Apply middleware only for specific names (supports glob patterns).
 */
export const forName = <TContext, TResult>(
	pattern: string | RegExp,
	middleware: Middleware<TContext, TResult>,
): Middleware<TContext, TResult> => {
	const regex = typeof pattern === "string" ? globToRegex(pattern) : pattern
	return when((info) => regex.test(info.name), middleware)
}

// ============================================================================
// Helpers
// ============================================================================

const globToRegex = (glob: string): RegExp => {
	const escaped = glob
		.replace(/[.+^${}()|[\]\\]/g, "\\$&")
		.replace(/\*/g, ".*")
		.replace(/\?/g, ".")
	return new RegExp(`^${escaped}$`)
}
