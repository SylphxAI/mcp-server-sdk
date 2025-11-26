/**
 * Server Factory
 *
 * Creates an MCP server from tool/resource/prompt definitions.
 * Pure composition - no I/O until transport is connected.
 */

import type { PromptContext, PromptDefinition } from "../builders/prompt.js"
import type {
	AnyResourceDefinition,
	ResourceContext,
	ResourceDefinition,
	ResourceTemplateDefinition,
} from "../builders/resource.js"
import type { ToolContext, ToolDefinition } from "../builders/tool.js"
import { buildCompletionRegistry } from "../completions/handler.js"
import type { CompletionConfig } from "../completions/types.js"
import { compose } from "../middleware/compose.js"
import type { Middleware } from "../middleware/types.js"
import type { PaginationOptions } from "../pagination/index.js"
import * as Rpc from "../protocol/jsonrpc.js"
import type * as Mcp from "../protocol/mcp.js"
import { createSubscriptionManager } from "../subscriptions/manager.js"
import type { SubscriptionManager } from "../subscriptions/types.js"
import type { HandlerContext, NotificationContext, ServerState } from "./handler.js"
import { dispatch } from "./handler.js"

// ============================================================================
// Server Config
// ============================================================================

export interface ServerConfig<
	TToolCtx extends ToolContext = ToolContext,
	TResourceCtx extends ResourceContext = ResourceContext,
	TPromptCtx extends PromptContext = PromptContext,
> {
	readonly name: string
	readonly version: string
	readonly instructions?: string
	readonly tools?: readonly ToolDefinition<unknown, TToolCtx>[]
	readonly resources?: readonly AnyResourceDefinition<TResourceCtx>[]
	readonly prompts?: readonly PromptDefinition<TPromptCtx>[]
	/** Middleware to apply to all tool/resource/prompt handlers */
	readonly middleware?: readonly Middleware<
		HandlerContext<TToolCtx, TResourceCtx, TPromptCtx>,
		unknown
	>[]
	/** Completion providers for auto-complete */
	readonly completions?: readonly CompletionConfig[]
	/** Enable resource subscriptions */
	readonly subscriptions?: boolean | SubscriptionManager
	/** Pagination options */
	readonly pagination?: PaginationOptions
	/** Enable logging capability */
	readonly logging?: boolean
}

// ============================================================================
// Server Instance
// ============================================================================

export interface Server<
	TToolCtx extends ToolContext = ToolContext,
	TResourceCtx extends ResourceContext = ResourceContext,
	TPromptCtx extends PromptContext = PromptContext,
> {
	/** Server metadata */
	readonly name: string
	readonly version: string

	/** Process a single message and return response (if any) */
	readonly handle: (
		message: string,
		ctx: HandlerContext<TToolCtx, TResourceCtx, TPromptCtx>,
		notificationCtx?: NotificationContext
	) => Promise<string | null>

	/** Process parsed message */
	readonly handleMessage: (
		message: Rpc.JsonRpcMessage,
		ctx: HandlerContext<TToolCtx, TResourceCtx, TPromptCtx>,
		notificationCtx?: NotificationContext
	) => Promise<Rpc.JsonRpcResponse | null>

	/** Get server state (for introspection) */
	readonly state: ServerState<TToolCtx, TResourceCtx, TPromptCtx>

	/** Get subscription manager (if enabled) */
	readonly subscriptions?: SubscriptionManager
}

// ============================================================================
// Server Factory
// ============================================================================

/**
 * Create an MCP server from configuration.
 *
 * @example
 * ```ts
 * const server = createServer({
 *   name: "my-server",
 *   version: "1.0.0",
 *   tools: [readFileTool, writeFileTool],
 *   resources: [configResource],
 *   prompts: [codeReviewPrompt],
 * })
 *
 * // Use with transport
 * const transport = stdio(server)
 * await transport.start()
 * ```
 */
export const createServer = <
	TToolCtx extends ToolContext = ToolContext,
	TResourceCtx extends ResourceContext = ResourceContext,
	TPromptCtx extends PromptContext = PromptContext,
>(
	config: ServerConfig<TToolCtx, TResourceCtx, TPromptCtx>
): Server<TToolCtx, TResourceCtx, TPromptCtx> => {
	// Build state from config
	const state = buildState(config)

	// Message handler
	const handleMessage = async (
		message: Rpc.JsonRpcMessage,
		ctx: HandlerContext<TToolCtx, TResourceCtx, TPromptCtx>,
		notificationCtx?: NotificationContext
	): Promise<Rpc.JsonRpcResponse | null> => {
		const result = await dispatch(state, message, ctx, notificationCtx)
		return result.type === "response" ? result.response : null
	}

	// String message handler
	const handle = async (
		input: string,
		ctx: HandlerContext<TToolCtx, TResourceCtx, TPromptCtx>,
		notificationCtx?: NotificationContext
	): Promise<string | null> => {
		const parsed = Rpc.parseMessage(input)

		if (!parsed.ok) {
			const errorResponse = Rpc.error(null, Rpc.ErrorCode.ParseError, parsed.error)
			return Rpc.stringify(errorResponse)
		}

		const response = await handleMessage(parsed.value, ctx, notificationCtx)
		return response ? Rpc.stringify(response) : null
	}

	return {
		name: config.name,
		version: config.version,
		handle,
		handleMessage,
		state,
		subscriptions: state.subscriptions,
	}
}

// ============================================================================
// State Builder (Pure)
// ============================================================================

const buildState = <
	TToolCtx extends ToolContext,
	TResourceCtx extends ResourceContext,
	TPromptCtx extends PromptContext,
>(
	config: ServerConfig<TToolCtx, TResourceCtx, TPromptCtx>
): ServerState<TToolCtx, TResourceCtx, TPromptCtx> => {
	// Build tools map
	const tools = new Map<string, ToolDefinition<unknown, TToolCtx>>()
	for (const tool of config.tools ?? []) {
		tools.set(tool.name, tool)
	}

	// Build resources (separate static and templates)
	const resources = new Map<string, ResourceDefinition<TResourceCtx>>()
	const resourceTemplates: ResourceTemplateDefinition<TResourceCtx>[] = []

	for (const res of config.resources ?? []) {
		if (res.type === "static") {
			resources.set(res.uri, res)
		} else {
			resourceTemplates.push(res)
		}
	}

	// Build prompts map
	const prompts = new Map<string, PromptDefinition<TPromptCtx>>()
	for (const prompt of config.prompts ?? []) {
		prompts.set(prompt.name, prompt)
	}

	// Build completions registry
	const completions = config.completions?.length
		? buildCompletionRegistry(config.completions)
		: undefined

	// Build subscription manager
	const subscriptions =
		config.subscriptions === true
			? createSubscriptionManager()
			: config.subscriptions === false
				? undefined
				: config.subscriptions

	// Build capabilities
	const hasResources = resources.size > 0 || resourceTemplates.length > 0
	const capabilities: Mcp.ServerCapabilities = {
		...(tools.size > 0 && { tools: { listChanged: true } }),
		...(hasResources && {
			resources: {
				subscribe: !!subscriptions,
				listChanged: true,
			},
		}),
		...(prompts.size > 0 && { prompts: { listChanged: true } }),
		...(completions && { completions: {} }),
		...(config.logging && { logging: {} }),
	}

	// Compose middleware
	const middleware = config.middleware?.length ? compose(...config.middleware) : undefined

	return {
		name: config.name,
		version: config.version,
		instructions: config.instructions,
		tools,
		resources,
		resourceTemplates,
		prompts,
		capabilities,
		middleware,
		completions,
		subscriptions,
		pagination: config.pagination,
	}
}

// ============================================================================
// Utility: Default Context Factory
// ============================================================================

/**
 * Create a minimal handler context.
 * Extend this for custom contexts with additional dependencies.
 */
export const createContext = <
	TToolCtx extends ToolContext = ToolContext,
	TResourceCtx extends ResourceContext = ResourceContext,
	TPromptCtx extends PromptContext = PromptContext,
>(overrides?: {
	tool?: Partial<TToolCtx>
	resource?: Partial<TResourceCtx>
	prompt?: Partial<TPromptCtx>
}): HandlerContext<TToolCtx, TResourceCtx, TPromptCtx> => ({
	toolContext: { signal: undefined, ...overrides?.tool } as TToolCtx,
	resourceContext: { signal: undefined, ...overrides?.resource } as TResourceCtx,
	promptContext: { signal: undefined, ...overrides?.prompt } as TPromptCtx,
})
