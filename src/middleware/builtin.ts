/**
 * Built-in Middleware
 *
 * Common middleware implementations.
 */

import type { ToolsCallResult } from "../protocol/mcp.js"
import type { Middleware, RequestInfo } from "./types.js"

// ============================================================================
// Logging Middleware
// ============================================================================

export interface LoggingOptions {
	/** Log function (default: console.log) */
	readonly log?: (message: string) => void
	/** Include input in logs */
	readonly logInput?: boolean
	/** Include result in logs */
	readonly logResult?: boolean
}

/**
 * Logging middleware - logs request/response timing.
 *
 * @example
 * ```ts
 * const server = createServer({
 *   middleware: [logging({ logInput: true })],
 *   tools: [...],
 * })
 * ```
 */
export const logging = <TContext, TResult>(
	options: LoggingOptions = {},
): Middleware<TContext, TResult> => {
	const log = options.log ?? console.log

	return async (ctx, info, next) => {
		const start = Date.now()
		const prefix = `[${info.type}:${info.name}]`

		if (options.logInput) {
			log(`${prefix} input: ${JSON.stringify(info.input)}`)
		} else {
			log(`${prefix} started`)
		}

		try {
			const result = await next()
			const duration = Date.now() - start

			if (options.logResult) {
				log(`${prefix} completed in ${duration}ms: ${JSON.stringify(result)}`)
			} else {
				log(`${prefix} completed in ${duration}ms`)
			}

			return result
		} catch (error) {
			const duration = Date.now() - start
			log(`${prefix} failed in ${duration}ms: ${error}`)
			throw error
		}
	}
}

// ============================================================================
// Timing Middleware
// ============================================================================

export interface TimingContext {
	timing?: {
		start: number
		end?: number
		duration?: number
	}
}

/**
 * Timing middleware - adds timing info to context.
 */
export const timing = <TContext extends TimingContext, TResult>(): Middleware<TContext, TResult> => {
	return async (ctx, _info, next) => {
		const mutableCtx = ctx as TimingContext
		mutableCtx.timing = { start: Date.now() }

		try {
			const result = await next()
			mutableCtx.timing.end = Date.now()
			mutableCtx.timing.duration = mutableCtx.timing.end - mutableCtx.timing.start
			return result
		} catch (error) {
			mutableCtx.timing.end = Date.now()
			mutableCtx.timing.duration = mutableCtx.timing.end - mutableCtx.timing.start
			throw error
		}
	}
}

// ============================================================================
// Error Handling Middleware
// ============================================================================

export interface ErrorHandlerOptions<TResult> {
	/** Transform error into result */
	readonly onError: (error: unknown, info: RequestInfo) => TResult
}

/**
 * Error handling middleware - catches errors and transforms them.
 */
export const errorHandler = <TContext, TResult>(
	options: ErrorHandlerOptions<TResult>,
): Middleware<TContext, TResult> => {
	return async (_ctx, info, next) => {
		try {
			return await next()
		} catch (error) {
			return options.onError(error, info)
		}
	}
}

/**
 * Tool-specific error handler that returns proper ToolsCallResult.
 */
export const toolErrorHandler = <TContext>(): Middleware<TContext, ToolsCallResult> =>
	errorHandler<TContext, ToolsCallResult>({
		onError: (error, _info): ToolsCallResult => ({
			content: [{ type: "text", text: `Error: ${error}` }],
			isError: true,
		}),
	})

// ============================================================================
// Timeout Middleware
// ============================================================================

export interface TimeoutOptions {
	/** Timeout in milliseconds */
	readonly ms: number
	/** Custom timeout error message */
	readonly message?: string
}

/**
 * Timeout middleware - fails if handler takes too long.
 */
export const timeout = <TContext, TResult>(
	options: TimeoutOptions,
): Middleware<TContext, TResult> => {
	return async (_ctx, info, next) => {
		const controller = new AbortController()
		const timeoutId = setTimeout(() => controller.abort(), options.ms)

		try {
			const result = await Promise.race([
				next(),
				new Promise<never>((_, reject) => {
					controller.signal.addEventListener("abort", () => {
						reject(
							new Error(options.message ?? `${info.type}:${info.name} timed out after ${options.ms}ms`),
						)
					})
				}),
			])
			return result
		} finally {
			clearTimeout(timeoutId)
		}
	}
}

// ============================================================================
// Retry Middleware
// ============================================================================

export interface RetryOptions {
	/** Maximum retry attempts */
	readonly maxAttempts: number
	/** Delay between retries in ms (default: 0) */
	readonly delay?: number
	/** Exponential backoff multiplier (default: 1 = no backoff) */
	readonly backoff?: number
	/** Should retry on this error? */
	readonly shouldRetry?: (error: unknown) => boolean
}

/**
 * Retry middleware - retries failed handlers.
 */
export const retry = <TContext, TResult>(options: RetryOptions): Middleware<TContext, TResult> => {
	const { maxAttempts, delay = 0, backoff = 1, shouldRetry = () => true } = options

	return async (_ctx, _info, next) => {
		let lastError: unknown
		let currentDelay = delay

		for (let attempt = 1; attempt <= maxAttempts; attempt++) {
			try {
				return await next()
			} catch (error) {
				lastError = error

				if (attempt === maxAttempts || !shouldRetry(error)) {
					throw error
				}

				if (currentDelay > 0) {
					await new Promise((resolve) => setTimeout(resolve, currentDelay))
					currentDelay *= backoff
				}
			}
		}

		throw lastError
	}
}

// ============================================================================
// Cache Middleware
// ============================================================================

export interface CacheOptions<TResult> {
	/** Cache key generator */
	readonly key: (info: RequestInfo) => string
	/** TTL in milliseconds */
	readonly ttl: number
	/** Custom cache store (default: in-memory Map) */
	readonly store?: {
		get: (key: string) => TResult | undefined
		set: (key: string, value: TResult, ttl: number) => void
	}
}

/**
 * Cache middleware - caches results.
 */
export const cache = <TContext, TResult>(options: CacheOptions<TResult>): Middleware<TContext, TResult> => {
	const defaultStore = new Map<string, { value: TResult; expires: number }>()
	const store = options.store ?? {
		get(key: string) {
			const entry = defaultStore.get(key)
			if (!entry) return undefined
			if (Date.now() > entry.expires) {
				defaultStore.delete(key)
				return undefined
			}
			return entry.value
		},
		set(key: string, value: TResult, ttl: number) {
			defaultStore.set(key, { value, expires: Date.now() + ttl })
		},
	}

	return async (_ctx, info, next) => {
		const cacheKey = options.key(info)
		const cached = store.get(cacheKey)

		if (cached !== undefined) {
			return cached
		}

		const result = await next()
		store.set(cacheKey, result, options.ttl)
		return result
	}
}
