/**
 * Message Handler
 *
 * Core request/notification handler logic.
 */

import type { PromptDefinition } from "../builders/prompt.js"
import { toProtocolPrompt } from "../builders/prompt.js"
import type { ResourceDefinition, ResourceTemplateDefinition } from "../builders/resource.js"
import { matchesTemplate, toProtocolResource, toProtocolTemplate } from "../builders/resource.js"
import type { ToolDefinition } from "../builders/tool.js"
import { toProtocolTool } from "../builders/tool.js"
import { type PaginationOptions, paginate } from "../pagination/index.js"
import * as Rpc from "../protocol/jsonrpc.js"
import * as Mcp from "../protocol/mcp.js"

// ============================================================================
// Server State
// ============================================================================

export interface ServerState {
	readonly name: string
	readonly version: string
	readonly instructions?: string
	readonly tools: ReadonlyMap<string, ToolDefinition>
	readonly resources: ReadonlyMap<string, ResourceDefinition>
	readonly resourceTemplates: ReadonlyMap<string, ResourceTemplateDefinition>
	readonly prompts: ReadonlyMap<string, PromptDefinition>
	readonly capabilities: Mcp.ServerCapabilities
	readonly pagination?: PaginationOptions
}

// ============================================================================
// Handler Context
// ============================================================================

export interface HandlerContext {
	readonly signal?: AbortSignal
}

// ============================================================================
// Request Handlers
// ============================================================================

const handleInitialize = (state: ServerState): Mcp.InitializeResult => ({
	protocolVersion: Mcp.LATEST_PROTOCOL_VERSION,
	capabilities: state.capabilities,
	serverInfo: {
		name: state.name,
		version: state.version,
	},
	instructions: state.instructions,
})

const handlePing = (): Record<string, never> => ({})

const handleToolsList = (state: ServerState, params?: Mcp.ListParams): Mcp.ToolsListResult => {
	const allItems = Array.from(state.tools.entries()).map(([name, def]) => toProtocolTool(name, def))
	const { items, nextCursor } = paginate(allItems, params?.cursor, state.pagination)
	return { items, nextCursor }
}

const handleToolsCall = async (
	state: ServerState,
	params: Mcp.ToolsCallParams,
	ctx: HandlerContext
): Promise<Mcp.ToolsCallResult> => {
	const tool = state.tools.get(params.name)
	if (!tool) {
		return {
			content: [{ type: "text", text: `Unknown tool: ${params.name}` }],
			isError: true,
		}
	}

	try {
		return await tool.handler({ input: params.arguments ?? {}, ctx })
	} catch (error) {
		return {
			content: [{ type: "text", text: `Tool error: ${error}` }],
			isError: true,
		}
	}
}

const handleResourcesList = (
	state: ServerState,
	params?: Mcp.ListParams
): Mcp.ResourcesListResult => {
	const allItems = Array.from(state.resources.entries()).map(([name, def]) =>
		toProtocolResource(name, def)
	)
	const { items, nextCursor } = paginate(allItems, params?.cursor, state.pagination)
	return { items, nextCursor }
}

const handleResourceTemplatesList = (
	state: ServerState,
	params?: Mcp.ListParams
): Mcp.ResourceTemplatesListResult => {
	const allItems = Array.from(state.resourceTemplates.entries()).map(([name, def]) =>
		toProtocolTemplate(name, def)
	)
	const { items, nextCursor } = paginate(allItems, params?.cursor, state.pagination)
	return { items, nextCursor }
}

const handleResourcesRead = async (
	state: ServerState,
	params: Mcp.ResourcesReadParams,
	ctx: HandlerContext
): Promise<Mcp.ResourcesReadResult> => {
	// Try static resource first
	for (const [, def] of state.resources) {
		if (def.uri === params.uri) {
			return await def.handler(params.uri, ctx)
		}
	}

	// Try templates
	for (const [, template] of state.resourceTemplates) {
		if (matchesTemplate(template.uriTemplate, params.uri)) {
			return await template.handler(params.uri, ctx)
		}
	}

	throw new Error(`Resource not found: ${params.uri}`)
}

const handlePromptsList = (state: ServerState, params?: Mcp.ListParams): Mcp.PromptsListResult => {
	const allItems = Array.from(state.prompts.entries()).map(([name, def]) =>
		toProtocolPrompt(name, def)
	)
	const { items, nextCursor } = paginate(allItems, params?.cursor, state.pagination)
	return { items, nextCursor }
}

const handlePromptsGet = async (
	state: ServerState,
	params: Mcp.PromptsGetParams,
	ctx: HandlerContext
): Promise<Mcp.PromptsGetResult> => {
	const prompt = state.prompts.get(params.name)
	if (!prompt) {
		throw new Error(`Unknown prompt: ${params.name}`)
	}

	return await prompt.handler({ args: params.arguments ?? {}, ctx })
}

// ============================================================================
// Notification Handler
// ============================================================================

const handleNotification = (notification: Rpc.JsonRpcNotification): void => {
	switch (notification.method) {
		case Mcp.Method.Initialized:
			// Client is ready
			break
		default:
			// Unknown notification - ignore
			break
	}
}

// ============================================================================
// Main Dispatcher
// ============================================================================

export type HandlerResult =
	| { readonly type: "response"; readonly response: Rpc.JsonRpcResponse }
	| { readonly type: "none" }

export const dispatch = async (
	state: ServerState,
	message: Rpc.JsonRpcMessage,
	ctx: HandlerContext
): Promise<HandlerResult> => {
	// Handle notifications
	if (Rpc.isNotification(message)) {
		handleNotification(message)
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
					error instanceof Error ? error.message : String(error)
				),
			}
		}
	}

	return { type: "none" }
}

const handleRequest = async (
	state: ServerState,
	req: Rpc.JsonRpcRequest,
	ctx: HandlerContext
): Promise<unknown> => {
	switch (req.method) {
		case Mcp.Method.Initialize:
			return handleInitialize(state)

		case Mcp.Method.Ping:
			return handlePing()

		case Mcp.Method.ToolsList:
			return handleToolsList(state, req.params as Mcp.ListParams | undefined)

		case Mcp.Method.ToolsCall:
			return handleToolsCall(state, req.params as Mcp.ToolsCallParams, ctx)

		case Mcp.Method.ResourcesList:
			return handleResourcesList(state, req.params as Mcp.ListParams | undefined)

		case Mcp.Method.ResourcesTemplatesList:
			return handleResourceTemplatesList(state, req.params as Mcp.ListParams | undefined)

		case Mcp.Method.ResourcesRead:
			return handleResourcesRead(state, req.params as Mcp.ResourcesReadParams, ctx)

		case Mcp.Method.PromptsList:
			return handlePromptsList(state, req.params as Mcp.ListParams | undefined)

		case Mcp.Method.PromptsGet:
			return handlePromptsGet(state, req.params as Mcp.PromptsGetParams, ctx)

		default:
			throw new Error(`Unknown method: ${req.method}`)
	}
}
