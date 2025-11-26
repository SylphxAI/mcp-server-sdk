/**
 * Pure Message Handler
 *
 * Core request/notification handler logic.
 * Pure functions that take server state and return responses.
 */

import type { PromptContext, PromptDefinition } from "../builders/prompt.js"
import { toProtocolPrompt } from "../builders/prompt.js"
import type {
	AnyResourceDefinition,
	ResourceContext,
	ResourceDefinition,
	ResourceTemplateDefinition,
} from "../builders/resource.js"
import { matchesTemplate, toProtocolResource, toProtocolTemplate } from "../builders/resource.js"
import type { ToolContext, ToolDefinition } from "../builders/tool.js"
import { toProtocolTool } from "../builders/tool.js"
import type { CompletionRegistry } from "../completions/index.js"
import { handleComplete } from "../completions/index.js"
import type { Middleware, RequestInfo } from "../middleware/types.js"
import { paginate, type PaginationOptions } from "../pagination/index.js"
import * as Rpc from "../protocol/jsonrpc.js"
import * as Mcp from "../protocol/mcp.js"
import type { SubscriptionManager } from "../subscriptions/index.js"

// ============================================================================
// Server State (Immutable)
// ============================================================================

export interface ServerState<
	TToolCtx extends ToolContext = ToolContext,
	TResourceCtx extends ResourceContext = ResourceContext,
	TPromptCtx extends PromptContext = PromptContext,
> {
	readonly name: string
	readonly version: string
	readonly instructions?: string
	readonly tools: ReadonlyMap<string, ToolDefinition<unknown, TToolCtx>>
	readonly resources: ReadonlyMap<string, ResourceDefinition<TResourceCtx>>
	readonly resourceTemplates: readonly ResourceTemplateDefinition<TResourceCtx>[]
	readonly prompts: ReadonlyMap<string, PromptDefinition<TPromptCtx>>
	readonly capabilities: Mcp.ServerCapabilities
	readonly middleware?: Middleware<HandlerContext<TToolCtx, TResourceCtx, TPromptCtx>, unknown>
	readonly completions?: CompletionRegistry
	readonly subscriptions?: SubscriptionManager
	readonly pagination?: PaginationOptions
	readonly logLevel?: Mcp.LogLevel
}

// ============================================================================
// Handler Context
// ============================================================================

export interface HandlerContext<
	TToolCtx extends ToolContext = ToolContext,
	TResourceCtx extends ResourceContext = ResourceContext,
	TPromptCtx extends PromptContext = PromptContext,
> {
	readonly toolContext: TToolCtx
	readonly resourceContext: TResourceCtx
	readonly promptContext: TPromptCtx
}

// ============================================================================
// Handler Result
// ============================================================================

export type HandlerResult =
	| { readonly type: "response"; readonly response: Rpc.JsonRpcResponse }
	| { readonly type: "none" }

// ============================================================================
// Request Handlers (Pure Functions)
// ============================================================================

export const handleInitialize = (
	state: ServerState,
	_params: Mcp.InitializeParams,
): Mcp.InitializeResult => ({
	protocolVersion: Mcp.LATEST_PROTOCOL_VERSION,
	capabilities: state.capabilities,
	serverInfo: {
		name: state.name,
		version: state.version,
	},
	instructions: state.instructions,
})

export const handlePing = (): Record<string, never> => ({})

export const handleToolsList = (
	state: ServerState,
	params?: Mcp.ListParams,
): Mcp.ToolsListResult => {
	const allItems = Array.from(state.tools.values()).map(toProtocolTool)
	const { items, nextCursor } = paginate(allItems, params?.cursor, state.pagination)
	return { items, nextCursor }
}

export const handleToolsCall = async <
	TToolCtx extends ToolContext,
	TResourceCtx extends ResourceContext,
	TPromptCtx extends PromptContext,
>(
	state: ServerState<TToolCtx, TResourceCtx, TPromptCtx>,
	params: Mcp.ToolsCallParams,
	ctx: HandlerContext<TToolCtx, TResourceCtx, TPromptCtx>,
): Promise<Mcp.ToolsCallResult> => {
	const tool = state.tools.get(params.name)
	if (!tool) {
		return {
			content: [{ type: "text", text: `Unknown tool: ${params.name}` }],
			isError: true,
		}
	}

	const info: RequestInfo = {
		type: "tool",
		name: params.name,
		input: params.arguments ?? {},
		startTime: Date.now(),
	}

	const executeHandler = async (): Promise<Mcp.ToolsCallResult> => {
		try {
			const handler = tool.handler as (
				input: unknown,
			) => (ctx: TToolCtx) => Promise<Mcp.ToolsCallResult>
			return await handler(params.arguments ?? {})(ctx.toolContext)
		} catch (error) {
			return {
				content: [{ type: "text", text: `Tool error: ${error}` }],
				isError: true,
			}
		}
	}

	// Apply middleware if present
	if (state.middleware) {
		return (await state.middleware(ctx, info, executeHandler)) as Mcp.ToolsCallResult
	}

	return executeHandler()
}

export const handleResourcesList = (
	state: ServerState,
	params?: Mcp.ListParams,
): Mcp.ResourcesListResult => {
	const allItems = Array.from(state.resources.values()).map(toProtocolResource)
	const { items, nextCursor } = paginate(allItems, params?.cursor, state.pagination)
	return { items, nextCursor }
}

export const handleResourceTemplatesList = (
	state: ServerState,
	params?: Mcp.ListParams,
): Mcp.ResourceTemplatesListResult => {
	const allItems = state.resourceTemplates.map(toProtocolTemplate)
	const { items, nextCursor } = paginate(allItems, params?.cursor, state.pagination)
	return { items, nextCursor }
}

export const handleResourcesRead = async <
	TToolCtx extends ToolContext,
	TResourceCtx extends ResourceContext,
	TPromptCtx extends PromptContext,
>(
	state: ServerState<TToolCtx, TResourceCtx, TPromptCtx>,
	params: Mcp.ResourcesReadParams,
	ctx: HandlerContext<TToolCtx, TResourceCtx, TPromptCtx>,
): Promise<Mcp.ResourcesReadResult> => {
	const info: RequestInfo = {
		type: "resource",
		name: params.uri,
		input: params,
		startTime: Date.now(),
	}

	const executeHandler = async (): Promise<Mcp.ResourcesReadResult> => {
		// Try static resource first
		const staticResource = state.resources.get(params.uri)
		if (staticResource) {
			return await staticResource.handler(params.uri)(ctx.resourceContext)
		}

		// Try templates
		for (const template of state.resourceTemplates) {
			if (matchesTemplate(template.uriTemplate, params.uri)) {
				return await template.handler(params.uri)(ctx.resourceContext)
			}
		}

		throw new Error(`Resource not found: ${params.uri}`)
	}

	// Apply middleware if present
	if (state.middleware) {
		return (await state.middleware(ctx, info, executeHandler)) as Mcp.ResourcesReadResult
	}

	return executeHandler()
}

export const handlePromptsList = (
	state: ServerState,
	params?: Mcp.ListParams,
): Mcp.PromptsListResult => {
	const allItems = Array.from(state.prompts.values()).map(toProtocolPrompt)
	const { items, nextCursor } = paginate(allItems, params?.cursor, state.pagination)
	return { items, nextCursor }
}

export const handlePromptsGet = async <
	TToolCtx extends ToolContext,
	TResourceCtx extends ResourceContext,
	TPromptCtx extends PromptContext,
>(
	state: ServerState<TToolCtx, TResourceCtx, TPromptCtx>,
	params: Mcp.PromptsGetParams,
	ctx: HandlerContext<TToolCtx, TResourceCtx, TPromptCtx>,
): Promise<Mcp.PromptsGetResult> => {
	const prompt = state.prompts.get(params.name)
	if (!prompt) {
		throw new Error(`Unknown prompt: ${params.name}`)
	}

	const info: RequestInfo = {
		type: "prompt",
		name: params.name,
		input: params.arguments ?? {},
		startTime: Date.now(),
	}

	const executeHandler = async (): Promise<Mcp.PromptsGetResult> => {
		return await prompt.handler(params.arguments ?? {})(ctx.promptContext)
	}

	// Apply middleware if present
	if (state.middleware) {
		return (await state.middleware(ctx, info, executeHandler)) as Mcp.PromptsGetResult
	}

	return executeHandler()
}

// ============================================================================
// Additional Handlers
// ============================================================================

export const handleResourcesSubscribe = (
	state: ServerState,
	params: Mcp.ResourcesSubscribeParams,
	subscriberId: string,
): Record<string, never> => {
	state.subscriptions?.subscribe(params.uri, subscriberId)
	return {}
}

export const handleResourcesUnsubscribe = (
	state: ServerState,
	params: Mcp.ResourcesUnsubscribeParams,
	subscriberId: string,
): Record<string, never> => {
	state.subscriptions?.unsubscribe(params.uri, subscriberId)
	return {}
}

export const handleLoggingSetLevel = (
	params: Mcp.LoggingSetLevelParams,
): Record<string, never> => {
	// Note: The caller should update the state.logLevel
	// This handler just validates and returns success
	return {}
}

// ============================================================================
// Notification Handlers
// ============================================================================

export interface NotificationContext {
	readonly subscriberId?: string
	readonly onCancelled?: (requestId: string | number, reason?: string) => void
}

export const handleNotification = (
	notification: Rpc.JsonRpcNotification,
	ctx: NotificationContext,
): void => {
	switch (notification.method) {
		case Mcp.Method.Initialized:
			// Client is ready - nothing to do
			break

		case Mcp.Method.CancelledNotification: {
			const params = notification.params as Mcp.CancelledNotificationParams
			ctx.onCancelled?.(params.requestId, params.reason)
			break
		}

		default:
			// Unknown notification - ignore
			break
	}
}

// ============================================================================
// Main Dispatcher (Pure Function)
// ============================================================================

export const dispatch = async <
	TToolCtx extends ToolContext,
	TResourceCtx extends ResourceContext,
	TPromptCtx extends PromptContext,
>(
	state: ServerState<TToolCtx, TResourceCtx, TPromptCtx>,
	message: Rpc.JsonRpcMessage,
	ctx: HandlerContext<TToolCtx, TResourceCtx, TPromptCtx>,
	notificationCtx?: NotificationContext,
): Promise<HandlerResult> => {
	// Handle notifications (no response)
	if (Rpc.isNotification(message)) {
		handleNotification(message, notificationCtx ?? {})
		return { type: "none" }
	}

	// Handle requests
	if (Rpc.isRequest(message)) {
		try {
			const result = await handleRequest(state, message, ctx, notificationCtx)
			return {
				type: "response",
				response: Rpc.success(message.id, result),
			}
		} catch (error) {
			return {
				type: "response",
				response: Rpc.error(
					message.id,
					Rpc.ErrorCode.InternalError,
					error instanceof Error ? error.message : String(error),
				),
			}
		}
	}

	// Unknown message type
	return { type: "none" }
}

const handleRequest = async <
	TToolCtx extends ToolContext,
	TResourceCtx extends ResourceContext,
	TPromptCtx extends PromptContext,
>(
	state: ServerState<TToolCtx, TResourceCtx, TPromptCtx>,
	req: Rpc.JsonRpcRequest,
	ctx: HandlerContext<TToolCtx, TResourceCtx, TPromptCtx>,
	notificationCtx?: NotificationContext,
): Promise<unknown> => {
	// Cast to base ServerState for handlers that don't need context types
	const baseState = state as ServerState

	switch (req.method) {
		case Mcp.Method.Initialize:
			return handleInitialize(baseState, req.params as Mcp.InitializeParams)

		case Mcp.Method.Ping:
			return handlePing()

		// Tools
		case Mcp.Method.ToolsList:
			return handleToolsList(baseState, req.params as Mcp.ListParams | undefined)

		case Mcp.Method.ToolsCall:
			return handleToolsCall(state, req.params as Mcp.ToolsCallParams, ctx)

		// Resources
		case Mcp.Method.ResourcesList:
			return handleResourcesList(baseState, req.params as Mcp.ListParams | undefined)

		case "resources/templates/list":
			return handleResourceTemplatesList(baseState, req.params as Mcp.ListParams | undefined)

		case Mcp.Method.ResourcesRead:
			return handleResourcesRead(state, req.params as Mcp.ResourcesReadParams, ctx)

		case Mcp.Method.ResourcesSubscribe:
			return handleResourcesSubscribe(
				baseState,
				req.params as Mcp.ResourcesSubscribeParams,
				notificationCtx?.subscriberId ?? "default",
			)

		case Mcp.Method.ResourcesUnsubscribe:
			return handleResourcesUnsubscribe(
				baseState,
				req.params as Mcp.ResourcesUnsubscribeParams,
				notificationCtx?.subscriberId ?? "default",
			)

		// Prompts
		case Mcp.Method.PromptsList:
			return handlePromptsList(baseState, req.params as Mcp.ListParams | undefined)

		case Mcp.Method.PromptsGet:
			return handlePromptsGet(state, req.params as Mcp.PromptsGetParams, ctx)

		// Completions
		case Mcp.Method.CompletionComplete:
			if (!state.completions) {
				return { completion: { values: [] } }
			}
			return handleComplete(state.completions, req.params as Mcp.CompletionCompleteParams)

		// Logging
		case Mcp.Method.LoggingSetLevel:
			return handleLoggingSetLevel(req.params as Mcp.LoggingSetLevelParams)

		default:
			throw new Error(`Unknown method: ${req.method}`)
	}
}
