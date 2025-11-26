/**
 * Pure Functional Resource Builder
 *
 * Resources expose data to clients.
 * They are application-controlled (client decides when to read).
 */

import type {
	EmbeddedResource,
	Resource,
	ResourceTemplate,
	ResourcesReadResult,
} from "../protocol/mcp.js"

// ============================================================================
// Context Type
// ============================================================================

export interface ResourceContext {
	readonly signal?: AbortSignal
}

// ============================================================================
// Resource Definition Types
// ============================================================================

/** Handler for reading a resource */
export type ResourceHandler<TContext extends ResourceContext = ResourceContext> = (
	uri: string,
) => (ctx: TContext) => Promise<ResourcesReadResult> | ResourcesReadResult

/** Static resource definition */
export interface ResourceDefinition<TContext extends ResourceContext = ResourceContext> {
	readonly type: "static"
	readonly uri: string
	readonly name: string
	readonly description?: string
	readonly mimeType?: string
	readonly handler: ResourceHandler<TContext>
}

/** Template resource definition (dynamic URIs) */
export interface ResourceTemplateDefinition<TContext extends ResourceContext = ResourceContext> {
	readonly type: "template"
	readonly uriTemplate: string
	readonly name: string
	readonly description?: string
	readonly mimeType?: string
	readonly handler: ResourceHandler<TContext>
}

export type AnyResourceDefinition<TContext extends ResourceContext = ResourceContext> =
	| ResourceDefinition<TContext>
	| ResourceTemplateDefinition<TContext>

// ============================================================================
// Builder Configs
// ============================================================================

export interface ResourceConfig<TContext extends ResourceContext = ResourceContext> {
	readonly uri: string
	readonly name: string
	readonly description?: string
	readonly mimeType?: string
	readonly handler: ResourceHandler<TContext>
}

export interface ResourceTemplateConfig<TContext extends ResourceContext = ResourceContext> {
	readonly uriTemplate: string
	readonly name: string
	readonly description?: string
	readonly mimeType?: string
	readonly handler: ResourceHandler<TContext>
}

// ============================================================================
// Pure Builder Functions
// ============================================================================

/**
 * Create a static resource definition.
 *
 * @example
 * ```ts
 * const configResource = resource({
 *   uri: "config://app",
 *   name: "App Configuration",
 *   mimeType: "application/json",
 *   handler: (uri) => async (ctx) => ({
 *     contents: [{
 *       type: "resource",
 *       uri,
 *       mimeType: "application/json",
 *       text: JSON.stringify(ctx.config)
 *     }]
 *   })
 * })
 * ```
 */
export const resource = <TContext extends ResourceContext = ResourceContext>(
	config: ResourceConfig<TContext>,
): ResourceDefinition<TContext> => ({
	type: "static",
	uri: config.uri,
	name: config.name,
	description: config.description,
	mimeType: config.mimeType,
	handler: config.handler,
})

/**
 * Create a template resource definition.
 * URI templates follow RFC 6570.
 *
 * @example
 * ```ts
 * const fileResource = resourceTemplate({
 *   uriTemplate: "file:///{path}",
 *   name: "File",
 *   handler: (uri) => async (ctx) => {
 *     const path = extractPath(uri)
 *     const content = await ctx.fs.read(path)
 *     return resourceText(uri, content)
 *   }
 * })
 * ```
 */
export const resourceTemplate = <TContext extends ResourceContext = ResourceContext>(
	config: ResourceTemplateConfig<TContext>,
): ResourceTemplateDefinition<TContext> => ({
	type: "template",
	uriTemplate: config.uriTemplate,
	name: config.name,
	description: config.description,
	mimeType: config.mimeType,
	handler: config.handler,
})

// ============================================================================
// Metadata Extraction
// ============================================================================

export const toProtocolResource = (def: ResourceDefinition): Resource => ({
	uri: def.uri,
	name: def.name,
	description: def.description,
	mimeType: def.mimeType,
})

export const toProtocolTemplate = (def: ResourceTemplateDefinition): ResourceTemplate => ({
	uriTemplate: def.uriTemplate,
	name: def.name,
	description: def.description,
	mimeType: def.mimeType,
})

// ============================================================================
// Content Helpers
// ============================================================================

/** Create text resource content */
export const resourceText = (
	uri: string,
	text: string,
	mimeType?: string,
): ResourcesReadResult => ({
	contents: [
		{
			type: "resource",
			uri,
			text,
			mimeType,
		},
	],
})

/** Create binary resource content */
export const resourceBlob = (
	uri: string,
	blob: string, // base64
	mimeType: string,
): ResourcesReadResult => ({
	contents: [
		{
			type: "resource",
			uri,
			blob,
			mimeType,
		},
	],
})

/** Create multiple resource contents */
export const resourceContents = (...contents: EmbeddedResource[]): ResourcesReadResult => ({
	contents,
})

// ============================================================================
// URI Matching
// ============================================================================

/**
 * Check if URI matches a template pattern.
 * Simple implementation - supports {param} syntax.
 */
export const matchesTemplate = (template: string, uri: string): boolean => {
	const pattern = template.replace(/\{[^}]+\}/g, "[^/]+")
	const regex = new RegExp(`^${pattern}$`)
	return regex.test(uri)
}

/**
 * Extract parameters from URI based on template.
 */
export const extractParams = (template: string, uri: string): Record<string, string> | null => {
	const paramNames: string[] = []
	const pattern = template.replace(/\{([^}]+)\}/g, (_, name) => {
		paramNames.push(name)
		return "([^/]+)"
	})

	const regex = new RegExp(`^${pattern}$`)
	const match = uri.match(regex)

	if (!match) return null

	const params: Record<string, string> = {}
	paramNames.forEach((name, i) => {
		const value = match[i + 1]
		if (value) params[name] = value
	})
	return params
}
