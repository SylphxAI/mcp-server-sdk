import { describe, expect, test } from 'bun:test'
import { z } from 'zod'
import { messages, prompt, user } from '../builders/prompt.js'
import { resource, resourceTemplate, resourceText } from '../builders/resource.js'
import { text, tool } from '../builders/tool.js'
import * as Rpc from '../protocol/jsonrpc.js'
import * as Mcp from '../protocol/mcp.js'
import { type ServerState, dispatch } from './handler.js'

// Helper to create minimal state for testing
const createTestState = (overrides: Partial<ServerState> = {}): ServerState => ({
	name: 'test-server',
	version: '1.0.0',
	tools: new Map(),
	resources: new Map(),
	resourceTemplates: new Map(),
	prompts: new Map(),
	capabilities: {},
	...overrides,
})

// Helper context
const ctx = { signal: undefined }

describe('Server Handler', () => {
	describe('initialize', () => {
		test('returns server info and capabilities', async () => {
			const state = createTestState({
				instructions: 'Test instructions',
				capabilities: { tools: { listChanged: true } },
			})

			const req = Rpc.request(1, Mcp.Method.Initialize, {
				protocolVersion: Mcp.LATEST_PROTOCOL_VERSION,
				capabilities: {},
				clientInfo: { name: 'test-client', version: '1.0.0' },
			})

			const result = await dispatch(state, req, ctx)

			expect(result.type).toBe('response')
			if (result.type === 'response') {
				expect(Rpc.isSuccess(result.response)).toBe(true)
				if (Rpc.isSuccess(result.response)) {
					const init = result.response.result as Mcp.InitializeResult
					expect(init.serverInfo.name).toBe('test-server')
					expect(init.protocolVersion).toBe(Mcp.LATEST_PROTOCOL_VERSION)
					expect(init.instructions).toBe('Test instructions')
				}
			}
		})
	})

	describe('ping', () => {
		test('returns empty object', async () => {
			const state = createTestState()
			const req = Rpc.request(1, Mcp.Method.Ping)

			const result = await dispatch(state, req, ctx)

			expect(result.type).toBe('response')
			if (result.type === 'response' && Rpc.isSuccess(result.response)) {
				expect(result.response.result).toEqual({})
			}
		})
	})

	describe('tools/list', () => {
		test('returns tool definitions', async () => {
			const greet = tool()
				.description('Greet someone')
				.input(z.object({ name: z.string() }))
				.handler(({ input }) => text(`Hello ${input.name}`))

			const state = createTestState({
				tools: new Map([['greet', greet]]),
				capabilities: { tools: { listChanged: true } },
			})

			const req = Rpc.request(1, Mcp.Method.ToolsList)
			const result = await dispatch(state, req, ctx)

			expect(result.type).toBe('response')
			if (result.type === 'response' && Rpc.isSuccess(result.response)) {
				const list = result.response.result as Mcp.ToolsListResult
				expect(list.items).toHaveLength(1)
				expect(list.items[0]?.name).toBe('greet')
				expect(list.items[0]?.description).toBe('Greet someone')
			}
		})
	})

	describe('tools/call', () => {
		test('executes tool with input', async () => {
			const greet = tool()
				.input(z.object({ name: z.string() }))
				.handler(({ input }) => text(`Hello ${input.name}`))

			const state = createTestState({
				tools: new Map([['greet', greet]]),
			})

			const req = Rpc.request(1, Mcp.Method.ToolsCall, {
				name: 'greet',
				arguments: { name: 'World' },
			})

			const result = await dispatch(state, req, ctx)

			expect(result.type).toBe('response')
			if (result.type === 'response' && Rpc.isSuccess(result.response)) {
				const call = result.response.result as Mcp.ToolsCallResult
				expect(call.content[0]).toEqual({ type: 'text', text: 'Hello World' })
			}
		})

		test('executes tool without input', async () => {
			const ping = tool().handler(() => text('pong'))

			const state = createTestState({
				tools: new Map([['ping', ping]]),
			})

			const req = Rpc.request(1, Mcp.Method.ToolsCall, { name: 'ping' })
			const result = await dispatch(state, req, ctx)

			expect(result.type).toBe('response')
			if (result.type === 'response' && Rpc.isSuccess(result.response)) {
				const call = result.response.result as Mcp.ToolsCallResult
				expect(call.content[0]).toEqual({ type: 'text', text: 'pong' })
			}
		})

		test('returns error for unknown tool', async () => {
			const state = createTestState()
			const req = Rpc.request(1, Mcp.Method.ToolsCall, { name: 'unknown' })

			const result = await dispatch(state, req, ctx)

			expect(result.type).toBe('response')
			if (result.type === 'response' && Rpc.isSuccess(result.response)) {
				const call = result.response.result as Mcp.ToolsCallResult
				expect(call.isError).toBe(true)
			}
		})

		test('returns validation error for invalid input', async () => {
			const greet = tool()
				.input(z.object({ name: z.string() }))
				.handler(({ input }) => text(`Hello ${input.name}`))

			const state = createTestState({
				tools: new Map([['greet', greet]]),
			})

			const req = Rpc.request(1, Mcp.Method.ToolsCall, {
				name: 'greet',
				arguments: { name: 123 }, // Invalid - should be string
			})

			const result = await dispatch(state, req, ctx)

			expect(result.type).toBe('response')
			if (result.type === 'response' && Rpc.isSuccess(result.response)) {
				const call = result.response.result as Mcp.ToolsCallResult
				expect(call.isError).toBe(true)
				expect((call.content[0] as Mcp.TextContent).text).toContain('Validation error')
			}
		})
	})

	describe('resources/list', () => {
		test('returns resource definitions', async () => {
			const config = resource()
				.uri('config://app')
				.description('App config')
				.mimeType('application/json')
				.handler(({ uri }) => resourceText(uri, '{}'))

			const state = createTestState({
				resources: new Map([['config', config]]),
			})

			const req = Rpc.request(1, Mcp.Method.ResourcesList)
			const result = await dispatch(state, req, ctx)

			expect(result.type).toBe('response')
			if (result.type === 'response' && Rpc.isSuccess(result.response)) {
				const list = result.response.result as Mcp.ResourcesListResult
				expect(list.items).toHaveLength(1)
				expect(list.items[0]?.name).toBe('config')
				expect(list.items[0]?.uri).toBe('config://app')
			}
		})
	})

	describe('resources/templates/list', () => {
		test('returns template definitions', async () => {
			const file = resourceTemplate()
				.uriTemplate('file:///{path}')
				.description('Read file')
				.handler(({ uri }) => resourceText(uri, 'content'))

			const state = createTestState({
				resourceTemplates: new Map([['file', file]]),
			})

			const req = Rpc.request(1, Mcp.Method.ResourcesTemplatesList)
			const result = await dispatch(state, req, ctx)

			expect(result.type).toBe('response')
			if (result.type === 'response' && Rpc.isSuccess(result.response)) {
				const list = result.response.result as Mcp.ResourceTemplatesListResult
				expect(list.items).toHaveLength(1)
				expect(list.items[0]?.uriTemplate).toBe('file:///{path}')
			}
		})
	})

	describe('resources/read', () => {
		test('reads static resource', async () => {
			const config = resource()
				.uri('config://app')
				.handler(({ uri }) => resourceText(uri, '{"version":"1.0"}'))

			const state = createTestState({
				resources: new Map([['config', config]]),
			})

			const req = Rpc.request(1, Mcp.Method.ResourcesRead, { uri: 'config://app' })
			const result = await dispatch(state, req, ctx)

			expect(result.type).toBe('response')
			if (result.type === 'response' && Rpc.isSuccess(result.response)) {
				const read = result.response.result as Mcp.ResourcesReadResult
				expect(read.contents[0]?.text).toBe('{"version":"1.0"}')
			}
		})

		test('reads from template', async () => {
			const file = resourceTemplate()
				.uriTemplate('file:///{path}')
				.handler(({ uri, params }) => resourceText(uri, `File: ${params.path}`))

			const state = createTestState({
				resourceTemplates: new Map([['file', file]]),
			})

			const req = Rpc.request(1, Mcp.Method.ResourcesRead, { uri: 'file:///test.txt' })
			const result = await dispatch(state, req, ctx)

			expect(result.type).toBe('response')
			if (result.type === 'response' && Rpc.isSuccess(result.response)) {
				const read = result.response.result as Mcp.ResourcesReadResult
				expect(read.contents[0]?.text).toBe('File: test.txt')
			}
		})

		test('returns error for unknown resource', async () => {
			const state = createTestState()
			const req = Rpc.request(1, Mcp.Method.ResourcesRead, { uri: 'unknown://foo' })

			const result = await dispatch(state, req, ctx)

			expect(result.type).toBe('response')
			if (result.type === 'response') {
				expect(Rpc.isError(result.response)).toBe(true)
			}
		})
	})

	describe('prompts/list', () => {
		test('returns prompt definitions', async () => {
			const review = prompt()
				.description('Code review')
				.args(z.object({ code: z.string() }))
				.handler(({ args }) => messages(user(`Review: ${args.code}`)))

			const state = createTestState({
				prompts: new Map([['review', review]]),
			})

			const req = Rpc.request(1, Mcp.Method.PromptsList)
			const result = await dispatch(state, req, ctx)

			expect(result.type).toBe('response')
			if (result.type === 'response' && Rpc.isSuccess(result.response)) {
				const list = result.response.result as Mcp.PromptsListResult
				expect(list.items).toHaveLength(1)
				expect(list.items[0]?.name).toBe('review')
			}
		})
	})

	describe('prompts/get', () => {
		test('generates prompt messages', async () => {
			const review = prompt()
				.args(z.object({ code: z.string() }))
				.handler(({ args }) => messages(user(`Review: ${args.code}`)))

			const state = createTestState({
				prompts: new Map([['review', review]]),
			})

			const req = Rpc.request(1, Mcp.Method.PromptsGet, {
				name: 'review',
				arguments: { code: 'console.log("hi")' },
			})

			const result = await dispatch(state, req, ctx)

			expect(result.type).toBe('response')
			if (result.type === 'response' && Rpc.isSuccess(result.response)) {
				const get = result.response.result as Mcp.PromptsGetResult
				expect(get.messages).toHaveLength(1)
				expect(get.messages[0]?.role).toBe('user')
			}
		})

		test('prompt without args', async () => {
			const hello = prompt().handler(() => messages(user('Hello!')))

			const state = createTestState({
				prompts: new Map([['hello', hello]]),
			})

			const req = Rpc.request(1, Mcp.Method.PromptsGet, { name: 'hello' })
			const result = await dispatch(state, req, ctx)

			expect(result.type).toBe('response')
			if (result.type === 'response' && Rpc.isSuccess(result.response)) {
				const get = result.response.result as Mcp.PromptsGetResult
				expect(get.messages[0]?.content).toEqual({ type: 'text', text: 'Hello!' })
			}
		})
	})

	describe('notifications', () => {
		test('handles initialized notification', async () => {
			const state = createTestState()
			const notification = Rpc.notification(Mcp.Method.Initialized)

			const result = await dispatch(state, notification, ctx)

			expect(result.type).toBe('none')
		})

		test('handles unknown notification gracefully', async () => {
			const state = createTestState()
			const notification = Rpc.notification('unknown/notification')

			const result = await dispatch(state, notification, ctx)

			expect(result.type).toBe('none')
		})
	})

	describe('unknown method', () => {
		test('returns error', async () => {
			const state = createTestState()
			const req = Rpc.request(1, 'unknown/method')

			const result = await dispatch(state, req, ctx)

			expect(result.type).toBe('response')
			if (result.type === 'response') {
				expect(Rpc.isError(result.response)).toBe(true)
			}
		})
	})
})
