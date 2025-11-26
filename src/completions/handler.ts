/**
 * Completions Handler
 *
 * Handles completion/complete requests.
 */

import type * as Mcp from "../protocol/mcp.js"
import type { CompletionConfig, CompletionProvider } from "./types.js"

// ============================================================================
// Completion Registry
// ============================================================================

export interface CompletionRegistry {
	readonly prompts: ReadonlyMap<string, CompletionProvider>
	readonly resources: ReadonlyMap<string, CompletionProvider>
}

/**
 * Build a completion registry from configs.
 */
export const buildCompletionRegistry = (
	configs: readonly CompletionConfig[]
): CompletionRegistry => {
	const prompts = new Map<string, CompletionProvider>()
	const resources = new Map<string, CompletionProvider>()

	for (const config of configs) {
		if (config.type === "prompt") {
			prompts.set(config.name, config.provider)
		} else {
			resources.set(config.uriTemplate, config.provider)
		}
	}

	return { prompts, resources }
}

// ============================================================================
// Completion Handler
// ============================================================================

/**
 * Handle a completion/complete request.
 */
export const handleComplete = async (
	registry: CompletionRegistry,
	params: Mcp.CompletionCompleteParams
): Promise<Mcp.CompletionCompleteResult> => {
	const { ref, argument } = params

	let provider: CompletionProvider | undefined

	if (ref.type === "ref/prompt" && ref.name) {
		provider = registry.prompts.get(ref.name)
	} else if (ref.type === "ref/resource" && ref.uri) {
		// Find matching template
		for (const [template, p] of registry.resources) {
			if (matchesTemplate(template, ref.uri)) {
				provider = p
				break
			}
		}
	}

	if (!provider) {
		return {
			completion: { values: [] },
		}
	}

	const result = await provider(argument.name, argument.value)

	return {
		completion: {
			values: result.values,
			total: result.total,
			hasMore: result.hasMore,
		},
	}
}

// ============================================================================
// Helpers
// ============================================================================

/**
 * Simple template matching (supports {param} placeholders).
 */
const matchesTemplate = (template: string, uri: string): boolean => {
	const pattern = template.replace(/\{[^}]+\}/g, "[^/]+")
	const regex = new RegExp(`^${pattern}$`)
	return regex.test(uri)
}

/**
 * Create a static completion provider.
 */
export const staticCompletions = (values: readonly string[]): CompletionProvider => {
	return (_, prefix) => {
		const filtered = values.filter((v) => v.toLowerCase().startsWith(prefix.toLowerCase()))
		return {
			values: filtered,
			total: filtered.length,
			hasMore: false,
		}
	}
}

/**
 * Create a completion provider from a function.
 */
export const dynamicCompletions = (
	fn: (prefix: string) => readonly string[] | Promise<readonly string[]>
): CompletionProvider => {
	return async (_, prefix) => {
		const values = await fn(prefix)
		return {
			values,
			total: values.length,
			hasMore: false,
		}
	}
}

/**
 * Combine multiple completion providers.
 */
export const mergeCompletions = (
	...providers: readonly CompletionProvider[]
): CompletionProvider => {
	return async (name, value) => {
		const results = await Promise.all(providers.map((p) => p(name, value)))
		const allValues = results.flatMap((r) => r.values)
		const unique = [...new Set(allValues)]
		return {
			values: unique,
			total: unique.length,
			hasMore: results.some((r) => r.hasMore),
		}
	}
}
