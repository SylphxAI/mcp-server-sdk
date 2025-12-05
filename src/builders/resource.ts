/**
 * Resource Builder
 *
 * Builder pattern for creating MCP resources and resource templates.
 *
 * @example
 * ```ts
 * // Static resource
 * const readme = resource()
 *   .uri('file:///README.md')
 *   .description('README file')
 *   .handler(({ uri }) => resourceText(uri, '# Hello'))
 *
 * // Resource template
 * const file = resourceTemplate()
 *   .uriTemplate('file:///{path}')
 *   .description('Read any file')
 *   .handler(({ uri, params }) => resourceText(uri, `Content of ${params.path}`))
 * ```
 */

import type {
	EmbeddedResource,
	Resource,
	ResourcesReadResult,
	ResourceTemplate,
} from "../protocol/mcp.js"

// ============================================================================
// Context Type
// ============================================================================

export interface ResourceContext {
	readonly signal?: AbortSignal
}

// ============================================================================
// Handler Types
// ============================================================================

export interface ResourceHandlerArgs {
	readonly uri: string
	readonly ctx: ResourceContext
}

export interface TemplateHandlerArgs<TParams = Record<string, string>> {
	readonly uri: string
	readonly params: TParams
	readonly ctx: ResourceContext
}

export type ResourceHandler = (
	args: ResourceHandlerArgs,
) => ResourcesReadResult | Promise<ResourcesReadResult>

export type TemplateHandler<TParams = Record<string, string>> = (
	args: TemplateHandlerArgs<TParams>,
) => ResourcesReadResult | Promise<ResourcesReadResult>

// ============================================================================
// Resource Definition
// ============================================================================

export interface ResourceDefinition {
	readonly uri: string
	readonly description?: string
	readonly mimeType?: string
	readonly handler: (uri: string, ctx: ResourceContext) => Promise<ResourcesReadResult>
}

export interface ResourceTemplateDefinition {
	readonly uriTemplate: string
	readonly description?: string
	readonly mimeType?: string
	readonly handler: (uri: string, ctx: ResourceContext) => Promise<ResourcesReadResult>
}

// ============================================================================
// Static Resource Builder
// ============================================================================

interface ResourceBuilderStart {
	uri(uri: string): ResourceBuilder
}

interface ResourceBuilder {
	description(desc: string): ResourceBuilder
	mimeType(mime: string): ResourceBuilder
	handler(fn: ResourceHandler): ResourceDefinition
}

interface ResourceState {
	uri?: string
	description?: string
	mimeType?: string
}

const createResourceBuilderStart = (): ResourceBuilderStart => ({
	uri(uri: string) {
		return createResourceBuilder({ uri })
	},
})

const createResourceBuilder = (state: ResourceState): ResourceBuilder => ({
	description(desc: string) {
		return createResourceBuilder({ ...state, description: desc })
	},
	mimeType(mime: string) {
		return createResourceBuilder({ ...state, mimeType: mime })
	},
	handler(fn: ResourceHandler): ResourceDefinition {
		if (!state.uri) throw new Error("Resource URI is required")
		return {
			uri: state.uri,
			description: state.description,
			mimeType: state.mimeType,
			handler: async (uri, ctx) => fn({ uri, ctx }),
		}
	},
})

// ============================================================================
// Resource Template Builder
// ============================================================================

interface TemplateBuilderStart {
	uriTemplate(template: string): TemplateBuilder
}

interface TemplateBuilder {
	description(desc: string): TemplateBuilder
	mimeType(mime: string): TemplateBuilder
	handler<TParams = Record<string, string>>(
		fn: TemplateHandler<TParams>,
	): ResourceTemplateDefinition
}

interface TemplateState {
	uriTemplate?: string
	description?: string
	mimeType?: string
}

const createTemplateBuilderStart = (): TemplateBuilderStart => ({
	uriTemplate(template: string) {
		return createTemplateBuilder({ uriTemplate: template })
	},
})

const createTemplateBuilder = (state: TemplateState): TemplateBuilder => ({
	description(desc: string) {
		return createTemplateBuilder({ ...state, description: desc })
	},
	mimeType(mime: string) {
		return createTemplateBuilder({ ...state, mimeType: mime })
	},
	handler<TParams = Record<string, string>>(
		fn: TemplateHandler<TParams>,
	): ResourceTemplateDefinition {
		if (!state.uriTemplate) throw new Error("URI template is required")
		const template = state.uriTemplate
		return {
			uriTemplate: template,
			description: state.description,
			mimeType: state.mimeType,
			handler: async (uri, ctx) => {
				const params = extractParams(template, uri) as TParams
				return fn({ uri, params, ctx })
			},
		}
	},
})

// ============================================================================
// Public API
// ============================================================================

/**
 * Create a static resource.
 *
 * @example
 * ```ts
 * const config = resource()
 *   .uri('config://app')
 *   .description('App configuration')
 *   .mimeType('application/json')
 *   .handler(({ uri }) => resourceText(uri, '{"version": "1.0"}'))
 * ```
 */
export const resource = (): ResourceBuilderStart => createResourceBuilderStart()

/**
 * Create a resource template for dynamic URIs.
 *
 * @example
 * ```ts
 * const file = resourceTemplate()
 *   .uriTemplate('file:///{path}')
 *   .description('Read any file')
 *   .handler(({ uri, params }) => resourceText(uri, `File: ${params.path}`))
 * ```
 */
export const resourceTemplate = (): TemplateBuilderStart => createTemplateBuilderStart()

// ============================================================================
// Protocol Conversion
// ============================================================================

export const toProtocolResource = (name: string, def: ResourceDefinition): Resource => ({
	uri: def.uri,
	name,
	description: def.description,
	mimeType: def.mimeType,
})

export const toProtocolTemplate = (
	name: string,
	def: ResourceTemplateDefinition,
): ResourceTemplate => ({
	uriTemplate: def.uriTemplate,
	name,
	description: def.description,
	mimeType: def.mimeType,
})

// ============================================================================
// URI Template Matching
// ============================================================================

/**
 * Check if a URI matches a template pattern.
 */
export const matchesTemplate = (template: string, uri: string): boolean => {
	const pattern = template.replace(/\{[^}]+\}/g, "[^/]+")
	const regex = new RegExp(`^${pattern}$`)
	return regex.test(uri)
}

/**
 * Extract parameters from a URI based on template.
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
	for (let i = 0; i < paramNames.length; i++) {
		const value = match[i + 1]
		const name = paramNames[i]
		if (value && name) params[name] = value
	}
	return params
}

// ============================================================================
// Content Helpers
// ============================================================================

export const resourceText = (
	uri: string,
	text: string,
	mimeType?: string,
): ResourcesReadResult => ({
	contents: [{ type: "resource", uri, text, mimeType }],
})

export const resourceBlob = (uri: string, blob: string, mimeType: string): ResourcesReadResult => ({
	contents: [{ type: "resource", uri, blob, mimeType }],
})

export const resourceContents = (...items: EmbeddedResource[]): ResourcesReadResult => ({
	contents: items,
})
