/**
 * Server Factory
 *
 * Creates and starts an MCP server.
 *
 * @example
 * ```ts
 * import { createServer, tool, text, stdio } from '@sylphx/mcp-server-sdk'
 * import { z } from 'zod'
 *
 * const greet = tool()
 *   .description('Greet someone')
 *   .input(z.object({ name: z.string() }))
 *   .handler(({ input }) => text(`Hello ${input.name}`))
 *
 * const ping = tool()
 *   .handler(() => text('pong'))
 *
 * const server = createServer({
 *   tools: { greet, ping },
 *   transport: stdio()
 * })
 *
 * await server.start()
 * ```
 */

import type { PromptDefinition } from "../builders/prompt.js"
import type { ResourceDefinition, ResourceTemplateDefinition } from "../builders/resource.js"
import type { ToolDefinition } from "../builders/tool.js"
import { noopEmitter } from "../notifications/index.js"
import type { PaginationOptions } from "../pagination/index.js"
import * as Rpc from "../protocol/jsonrpc.js"
import type * as Mcp from "../protocol/mcp.js"
import type { Transport, TransportFactory } from "../transports/types.js"
import { type HandlerContext, type ServerState, dispatch } from "./handler.js"

// ============================================================================
// Server Config
// ============================================================================

export interface ServerConfig {
	/** Server name (default: "mcp-server") */
	readonly name?: string
	/** Server version (default: "1.0.0") */
	readonly version?: string
	/** Instructions for the LLM */
	readonly instructions?: string
	/** Tool definitions (key = tool name) */
	readonly tools?: Record<string, ToolDefinition>
	/** Resource definitions (key = resource name) */
	readonly resources?: Record<string, ResourceDefinition>
	/** Resource template definitions (key = template name) */
	readonly resourceTemplates?: Record<string, ResourceTemplateDefinition>
	/** Prompt definitions (key = prompt name) */
	readonly prompts?: Record<string, PromptDefinition>
	/** Pagination options */
	readonly pagination?: PaginationOptions
	/** Transport factory */
	readonly transport: TransportFactory
}

// ============================================================================
// Server Instance
// ============================================================================

export interface Server {
	readonly name: string
	readonly version: string
	readonly start: () => Promise<void>
	readonly stop: () => Promise<void>
}

// ============================================================================
// Server Factory
// ============================================================================

/**
 * Create an MCP server.
 *
 * @example
 * ```ts
 * const server = createServer({
 *   tools: { greet, ping },
 *   transport: stdio()
 * })
 *
 * await server.start()
 * ```
 */
export const createServer = (config: ServerConfig): Server => {
	const name = config.name ?? "mcp-server"
	const version = config.version ?? "1.0.0"

	// Build state
	const state = buildState(config, name, version)

	// Create handler context
	const ctx: HandlerContext = {
		signal: undefined,
	}

	// Message handler
	const handle = async (input: string): Promise<string | null> => {
		const parsed = Rpc.parseMessage(input)

		if (!parsed.ok) {
			const errorResponse = Rpc.error(null, Rpc.ErrorCode.ParseError, parsed.error)
			return Rpc.stringify(errorResponse)
		}

		const result = await dispatch(state, parsed.value, ctx)
		return result.type === "response" ? Rpc.stringify(result.response) : null
	}

	// Create transport
	const transport: Transport = config.transport({ name, version, handle }, noopEmitter)

	return {
		name,
		version,
		start: transport.start,
		stop: transport.stop,
	}
}

// ============================================================================
// State Builder
// ============================================================================

const buildState = (config: ServerConfig, name: string, version: string): ServerState => {
	const tools = new Map(Object.entries(config.tools ?? {}))
	const resources = new Map(Object.entries(config.resources ?? {}))
	const resourceTemplates = new Map(Object.entries(config.resourceTemplates ?? {}))
	const prompts = new Map(Object.entries(config.prompts ?? {}))

	// Build capabilities
	const capabilities: Mcp.ServerCapabilities = {
		...(tools.size > 0 && { tools: { listChanged: true } }),
		...((resources.size > 0 || resourceTemplates.size > 0) && {
			resources: { subscribe: false, listChanged: true },
		}),
		...(prompts.size > 0 && { prompts: { listChanged: true } }),
	}

	return {
		name,
		version,
		instructions: config.instructions,
		tools,
		resources,
		resourceTemplates,
		prompts,
		capabilities,
		pagination: config.pagination,
	}
}
