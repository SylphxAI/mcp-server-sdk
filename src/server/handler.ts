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
import type { Middleware, RequestInfo } from "../middleware/types.js"
import { compose } from "../middleware/compose.js"
import * as Rpc from "../protocol/jsonrpc.js"
import * as Mcp from "../protocol/mcp.js"

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

export const handleToolsList = (state: ServerState): Mcp.ToolsListResult => ({
	items: Array.from(state.tools.values()).map(toProtocolTool),
})

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

export const handleResourcesList = (state: ServerState): Mcp.ResourcesListResult => ({
	items: Array.from(state.resources.values()).map(toProtocolResource),
})

export const handleResourceTemplatesList = (
	state: ServerState,
): Mcp.ResourceTemplatesListResult => ({
	items: state.resourceTemplates.map(toProtocolTemplate),
})

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

export const handlePromptsList = (state: ServerState): Mcp.PromptsListResult => ({
	items: Array.from(state.prompts.values()).map(toProtocolPrompt),
})

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
): Promise<HandlerResult> => {
	// Handle notifications (no response)
	if (Rpc.isNotification(message)) {
		// notifications/initialized - client is ready
		// notifications/cancelled - cancel in-flight request
		// We don't need to respond to notifications
		return { type: "none" }
	}

	// Handle requests
	if (Rpc.isRequest(message)) {
		try {
			const result = await handleRequest(state, message, ctx)
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
): Promise<unknown> => {
	switch (req.method) {
		case Mcp.Method.Initialize:
			return handleInitialize(state, req.params as Mcp.InitializeParams)

		case Mcp.Method.Ping:
			return handlePing()

		case Mcp.Method.ToolsList:
			return handleToolsList(state)

		case Mcp.Method.ToolsCall:
			return handleToolsCall(state, req.params as Mcp.ToolsCallParams, ctx)

		case Mcp.Method.ResourcesList:
			return handleResourcesList(state)

		case "resources/templates/list":
			return handleResourceTemplatesList(state)

		case Mcp.Method.ResourcesRead:
			return handleResourcesRead(state, req.params as Mcp.ResourcesReadParams, ctx)

		case Mcp.Method.PromptsList:
			return handlePromptsList(state)

		case Mcp.Method.PromptsGet:
			return handlePromptsGet(state, req.params as Mcp.PromptsGetParams, ctx)

		default:
			throw new Error(`Unknown method: ${req.method}`)
	}
}
