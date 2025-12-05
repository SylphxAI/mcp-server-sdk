/**
 * MCP App
 *
 * Stateless MCP application that can be used with any transport.
 *
 * @example
 * ```ts
 * const app = createMcpApp({
 *   name: 'my-server',
 *   tools: { greet, ping },
 * })
 *
 * // Use with gust serve
 * await serve({ app, port: 3000 })
 *
 * // Or use standalone
 * const result = await app.handle(jsonRpcMessage)
 *
 * // Or use with Bun.serve
 * Bun.serve({ fetch: app.fetch, port: 3000 })
 * ```
 */

import type { PromptDefinition } from "../builders/prompt.js"
import type { ResourceDefinition, ResourceTemplateDefinition } from "../builders/resource.js"
import type { ToolDefinition } from "../builders/tool.js"
import type { PaginationOptions } from "../pagination/index.js"
import type * as Rpc from "../protocol/jsonrpc.js"
import type * as Mcp from "../protocol/mcp.js"
import {
	dispatch,
	type HandlerContext,
	type HandlerResult,
	type ServerState,
} from "../server/handler.js"
import { createFetchHandler } from "./http.js"

// ============================================================================
// Config
// ============================================================================

export interface McpAppConfig {
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
}

// ============================================================================
// McpApp Interface
// ============================================================================

export interface McpApp {
	/** Server name */
	readonly name: string
	/** Server version */
	readonly version: string
	/** Internal state (for serve/transports) */
	readonly state: ServerState

	/**
	 * Handle a JSON-RPC message (stateless).
	 *
	 * For session-aware handling, use the fetch adapter.
	 *
	 * @example
	 * ```ts
	 * const result = await app.handle(message)
	 * if (result.type === 'response') {
	 *   send(result.response)
	 * }
	 * ```
	 */
	readonly handle: (message: Rpc.JsonRpcMessage, ctx?: HandlerContext) => Promise<HandlerResult>

	/**
	 * HTTP fetch handler (MCP Streamable HTTP spec).
	 *
	 * Handles session management, SSE streaming, and bidirectional RPC.
	 * Compatible with Bun.serve, Deno.serve, Cloudflare Workers.
	 *
	 * @example
	 * ```ts
	 * // Bun
	 * Bun.serve({ fetch: app.fetch, port: 3000 })
	 *
	 * // Deno
	 * Deno.serve(app.fetch)
	 *
	 * // Cloudflare Workers
	 * export default { fetch: app.fetch }
	 * ```
	 */
	readonly fetch: (request: Request) => Promise<Response>
}

// ============================================================================
// Build Server State
// ============================================================================

const buildServerState = (config: McpAppConfig): ServerState => {
	const name = config.name ?? "mcp-server"
	const version = config.version ?? "1.0.0"

	const tools = new Map(Object.entries(config.tools ?? {}))
	const resources = new Map(Object.entries(config.resources ?? {}))
	const resourceTemplates = new Map(Object.entries(config.resourceTemplates ?? {}))
	const prompts = new Map(Object.entries(config.prompts ?? {}))

	const capabilities: Mcp.ServerCapabilities = {
		...(tools.size > 0 && { tools: {} }),
		...(resources.size > 0 && { resources: { subscribe: true } }),
		...(resourceTemplates.size > 0 && { resources: { subscribe: true } }),
		...(prompts.size > 0 && { prompts: {} }),
		logging: {},
		completions: {},
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
		subscriptions: new Set(),
	}
}

// ============================================================================
// Factory
// ============================================================================

/**
 * Create an MCP application.
 *
 * Returns a stateless app that can be used with any transport:
 * - `app.handle()` for direct JSON-RPC handling
 * - `app.fetch` for HTTP (Bun.serve, Deno.serve, Cloudflare Workers)
 * - `serve({ app })` for gust server
 *
 * @example
 * ```ts
 * import { createMcpApp, tool, text } from '@sylphx/mcp-server-sdk'
 *
 * const greet = tool()
 *   .description('Greet someone')
 *   .input(z.object({ name: z.string() }))
 *   .handler(({ input }) => text(`Hello ${input.name}`))
 *
 * const app = createMcpApp({
 *   name: 'my-server',
 *   tools: { greet },
 * })
 *
 * // Use with Bun.serve
 * Bun.serve({ fetch: app.fetch, port: 3000 })
 * ```
 */
export const createMcpApp = (config: McpAppConfig): McpApp => {
	const state = buildServerState(config)

	const handle = async (
		message: Rpc.JsonRpcMessage,
		ctx: HandlerContext = {},
	): Promise<HandlerResult> => {
		return dispatch(state, message, ctx)
	}

	const fetch = createFetchHandler(state)

	return {
		name: state.name,
		version: state.version,
		state,
		handle,
		fetch,
	}
}
