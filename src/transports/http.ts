/**
 * HTTP Transport (Streamable HTTP / SSE)
 *
 * Uses Bun.serve for high-performance HTTP server.
 */

import { createEmitter } from '../notifications/index.js'
import * as Rpc from '../protocol/jsonrpc.js'
import type { Transport, TransportFactory } from './types.js'

// ============================================================================
// Options
// ============================================================================

export interface HttpOptions {
	/** Port to listen on (default: 3000) */
	readonly port?: number
	/** Hostname to bind to (default: localhost) */
	readonly hostname?: string
	/** Path prefix for MCP endpoints (default: /mcp) */
	readonly basePath?: string
	/** CORS origin (set to "*" for all, or specific origin) */
	readonly cors?: string
	/** Error handler */
	readonly onError?: (error: Error) => void
}

// ============================================================================
// HTTP Transport Factory
// ============================================================================

/**
 * Create an HTTP transport.
 *
 * Endpoints:
 * - POST /mcp - JSON-RPC request/response
 * - GET /mcp/sse - Server-Sent Events stream
 * - GET /mcp/health - Health check
 *
 * @example
 * ```ts
 * createServer({
 *   tools: { ping },
 *   transport: http({ port: 3000 })
 * })
 * ```
 */
export const http = (options: HttpOptions = {}): TransportFactory => {
	return (server, notify): Transport => {
		const port = options.port ?? 3000
		const hostname = options.hostname ?? 'localhost'
		const basePath = options.basePath ?? '/mcp'
		const cors = options.cors
		const encoder = new TextEncoder()

		// Session storage for SSE
		const sessions = new Map<
			string,
			{
				controller: ReadableStreamDefaultController<Uint8Array>
			}
		>()

		// CORS headers
		const corsHeaders = cors
			? {
					'Access-Control-Allow-Origin': cors,
					'Access-Control-Allow-Methods': 'GET, POST, OPTIONS',
					'Access-Control-Allow-Headers': 'Content-Type, X-Session-ID',
				}
			: {}

		let bunServer: ReturnType<typeof Bun.serve> | null = null

		const handleRequest = async (req: Request): Promise<Response> => {
			const url = new URL(req.url)
			const path = url.pathname

			// Handle CORS preflight
			if (req.method === 'OPTIONS') {
				return new Response(null, { status: 204, headers: corsHeaders })
			}

			// SSE endpoint - establish stream
			if (path === `${basePath}/sse` && req.method === 'GET') {
				const sessionId = crypto.randomUUID()

				const stream = new ReadableStream<Uint8Array>({
					start(controller) {
						sessions.set(sessionId, { controller })
						controller.enqueue(
							encoder.encode(`event: session\ndata: ${JSON.stringify({ sessionId })}\n\n`)
						)
					},
					cancel() {
						sessions.delete(sessionId)
					},
				})

				return new Response(stream, {
					headers: {
						'Content-Type': 'text/event-stream',
						'Cache-Control': 'no-cache',
						Connection: 'keep-alive',
						...corsHeaders,
					},
				})
			}

			// SSE message endpoint
			if (path === `${basePath}/sse` && req.method === 'POST') {
				const sessionId = req.headers.get('X-Session-ID')
				if (!sessionId) {
					return new Response(JSON.stringify({ error: 'Missing X-Session-ID header' }), {
						status: 400,
						headers: { 'Content-Type': 'application/json', ...corsHeaders },
					})
				}

				const session = sessions.get(sessionId)
				if (!session) {
					return new Response(JSON.stringify({ error: 'Session not found' }), {
						status: 404,
						headers: { 'Content-Type': 'application/json', ...corsHeaders },
					})
				}

				try {
					const body = await req.text()
					const response = await server.handle(body)

					if (response) {
						session.controller.enqueue(encoder.encode(`event: message\ndata: ${response}\n\n`))
					}

					return new Response(JSON.stringify({ ok: true }), {
						headers: { 'Content-Type': 'application/json', ...corsHeaders },
					})
				} catch (error) {
					options.onError?.(error instanceof Error ? error : new Error(String(error)))
					return new Response(JSON.stringify({ error: 'Internal error' }), {
						status: 500,
						headers: { 'Content-Type': 'application/json', ...corsHeaders },
					})
				}
			}

			// Standard JSON-RPC endpoint
			if (path === basePath && req.method === 'POST') {
				try {
					const body = await req.text()
					const response = await server.handle(body)

					return new Response(response ?? '', {
						status: response ? 200 : 204,
						headers: { 'Content-Type': 'application/json', ...corsHeaders },
					})
				} catch (error) {
					options.onError?.(error instanceof Error ? error : new Error(String(error)))
					const errorResponse = Rpc.error(null, Rpc.ErrorCode.InternalError, 'Internal server error')
					return new Response(Rpc.stringify(errorResponse), {
						status: 500,
						headers: { 'Content-Type': 'application/json', ...corsHeaders },
					})
				}
			}

			// Health check
			if (path === `${basePath}/health` && req.method === 'GET') {
				return new Response(
					JSON.stringify({
						status: 'ok',
						server: server.name,
						version: server.version,
					}),
					{ headers: { 'Content-Type': 'application/json', ...corsHeaders } }
				)
			}

			return new Response('Not Found', { status: 404, headers: corsHeaders })
		}

		const start = async (): Promise<void> => {
			bunServer = Bun.serve({
				port,
				hostname,
				fetch: handleRequest,
			})
		}

		const stop = async (): Promise<void> => {
			for (const session of sessions.values()) {
				session.controller.close()
			}
			sessions.clear()
			bunServer?.stop()
			bunServer = null
		}

		return { start, stop }
	}
}
