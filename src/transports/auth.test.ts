import { afterEach, describe, expect, test } from "bun:test"
import { createServer } from "node:net"
import { createMcpApp } from "../app/app.js"
import { serve } from "../app/serve.js"
import { text, tool } from "../builders/tool.js"
import { isAuthorized, readHeader, UNAUTHORIZED_CODE, verifyApiKey, WWW_AUTHENTICATE } from "./auth.js"
import { http } from "./http.js"

// ============================================================================
// Helpers
// ============================================================================

const HOST = "127.0.0.1"
const API_KEY = "s3cret-key"

/** Reserve a free port on the loopback interface, then release it. */
const getFreePort = (): Promise<number> =>
	new Promise((resolve, reject) => {
		const srv = createServer()
		srv.once("error", reject)
		srv.listen(0, HOST, () => {
			const address = srv.address()
			if (address && typeof address === "object") {
				const { port } = address
				srv.close(() => resolve(port))
			} else {
				srv.close(() => reject(new Error("no port")))
			}
		})
	})

const ping = tool().handler(() => text("pong"))

const initBody = JSON.stringify({
	jsonrpc: "2.0",
	id: 1,
	method: "initialize",
	params: {
		protocolVersion: "2025-03-26",
		capabilities: {},
		clientInfo: { name: "test", version: "1.0.0" },
	},
})

const postMcp = (url: string, headers: Record<string, string> = {}): Promise<Response> =>
	fetch(url, {
		method: "POST",
		headers: { "Content-Type": "application/json", ...headers },
		body: initBody,
	})

// ============================================================================
// Pure auth logic
// ============================================================================

describe("auth logic", () => {
	test("verifyApiKey accepts a matching key", () => {
		expect(verifyApiKey(API_KEY, API_KEY)).toBe(true)
	})

	test("verifyApiKey rejects a wrong key", () => {
		expect(verifyApiKey(API_KEY, "wrong")).toBe(false)
	})

	test("verifyApiKey rejects missing or empty keys", () => {
		expect(verifyApiKey(API_KEY, undefined)).toBe(false)
		expect(verifyApiKey(API_KEY, "")).toBe(false)
	})

	test("readHeader reads Fetch Headers case-insensitively", () => {
		const req = new Request("http://x", { headers: { "X-API-Key": API_KEY } })
		expect(readHeader(req, "x-api-key")).toBe(API_KEY)
	})

	test("readHeader handles record + string[] values", () => {
		const carrier = { headers: { "x-api-key": [API_KEY, "other"] } }
		expect(readHeader(carrier, "x-api-key")).toBe(API_KEY)
		expect(readHeader({ headers: {} }, "x-api-key")).toBeUndefined()
	})

	test("isAuthorized precedence: authenticate wins over apiKey", async () => {
		const req = new Request("http://x", { headers: { "x-api-key": "wrong" } })
		expect(await isAuthorized({ apiKey: API_KEY, authenticate: () => true }, req)).toBe(true)
		expect(await isAuthorized({ apiKey: API_KEY }, req)).toBe(false)
		expect(await isAuthorized({}, req)).toBe(true)
	})
})

// ============================================================================
// http() transport — apiKey
// ============================================================================

describe("http() transport auth", () => {
	let stop: (() => Promise<void>) | null = null

	afterEach(async () => {
		await stop?.()
		stop = null
	})

	const start = async (port: number, apiKey?: string) => {
		const factory = http({ port, hostname: HOST, ...(apiKey && { apiKey }) })
		// Minimal ServerHandler stub: only `handle` is exercised by the route.
		const transport = factory(
			{
				name: "test",
				version: "1.0.0",
				handle: async () => JSON.stringify({ jsonrpc: "2.0", id: 1, result: {} }),
			},
			(() => {}) as never,
		)
		await transport.start()
		stop = transport.stop
		return `http://${HOST}:${port}`
	}

	test("401 with no key", async () => {
		const url = await start(await getFreePort(), API_KEY)
		const res = await postMcp(`${url}/mcp`)
		expect(res.status).toBe(401)
		expect(res.headers.get("WWW-Authenticate")).toBe(WWW_AUTHENTICATE)
		const body = (await res.json()) as { error: { code: number; message: string }; id: null }
		expect(body.error.code).toBe(UNAUTHORIZED_CODE)
		expect(body.error.message).toBe("Unauthorized")
		expect(body.id).toBeNull()
	})

	test("401 with wrong key", async () => {
		const url = await start(await getFreePort(), API_KEY)
		const res = await postMcp(`${url}/mcp`, { "X-API-Key": "nope" })
		expect(res.status).toBe(401)
	})

	test("200 with correct key", async () => {
		const url = await start(await getFreePort(), API_KEY)
		const res = await postMcp(`${url}/mcp`, { "X-API-Key": API_KEY })
		expect(res.status).toBe(200)
	})

	test("health stays open when auth is enabled", async () => {
		const url = await start(await getFreePort(), API_KEY)
		const res = await fetch(`${url}/mcp/health`)
		expect(res.status).toBe(200)
		const body = (await res.json()) as { status: string }
		expect(body.status).toBe("ok")
	})

	test("unauthenticated by default (no auth configured)", async () => {
		const url = await start(await getFreePort())
		const res = await postMcp(`${url}/mcp`)
		expect(res.status).toBe(200)
	})
})

// ============================================================================
// serve() entrypoint — authenticate hook
// ============================================================================

describe("serve() entrypoint auth", () => {
	let stop: (() => Promise<void>) | null = null

	afterEach(async () => {
		await stop?.()
		stop = null
	})

	test("authenticate hook accepts and rejects", async () => {
		const port = await getFreePort()
		const app = createMcpApp({ name: "test", tools: { ping } })
		const server = await serve({
			app,
			port,
			hostname: HOST,
			authenticate: (req) => readHeader(req, "authorization") === "Bearer good",
		})
		stop = server.stop
		const url = `http://${HOST}:${port}/mcp`

		const rejected = await postMcp(url, { Authorization: "Bearer bad" })
		expect(rejected.status).toBe(401)
		expect(rejected.headers.get("WWW-Authenticate")).toBe(WWW_AUTHENTICATE)

		const accepted = await postMcp(url, { Authorization: "Bearer good" })
		expect(accepted.status).toBe(200)
	})

	test("apiKey check + open health", async () => {
		const port = await getFreePort()
		const app = createMcpApp({ name: "test", tools: { ping } })
		const server = await serve({ app, port, hostname: HOST, apiKey: API_KEY })
		stop = server.stop
		const base = `http://${HOST}:${port}`

		expect((await postMcp(`${base}/mcp`)).status).toBe(401)
		expect((await postMcp(`${base}/mcp`, { "X-API-Key": API_KEY })).status).toBe(200)
		expect((await fetch(`${base}/mcp/health`)).status).toBe(200)
	})
})

// ============================================================================
// fetch adapter (app.fetch) — apiKey + hook
// ============================================================================

describe("fetch adapter auth", () => {
	test("apiKey gating on app.fetch", async () => {
		const app = createMcpApp({ name: "test", tools: { ping }, apiKey: API_KEY })

		const noKey = await app.fetch(
			new Request("http://local/mcp", {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: initBody,
			}),
		)
		expect(noKey.status).toBe(401)
		expect(noKey.headers.get("WWW-Authenticate")).toBe(WWW_AUTHENTICATE)
		const errBody = (await noKey.json()) as { error: { code: number } }
		expect(errBody.error.code).toBe(UNAUTHORIZED_CODE)

		const withKey = await app.fetch(
			new Request("http://local/mcp", {
				method: "POST",
				headers: { "Content-Type": "application/json", "X-API-Key": API_KEY },
				body: initBody,
			}),
		)
		expect(withKey.status).toBe(200)
	})

	test("health stays open on app.fetch", async () => {
		const app = createMcpApp({ name: "test", tools: { ping }, apiKey: API_KEY })
		const res = await app.fetch(new Request("http://local/mcp/health", { method: "GET" }))
		expect(res.status).toBe(200)
	})

	test("authenticate hook on app.fetch", async () => {
		const app = createMcpApp({
			name: "test",
			tools: { ping },
			authenticate: (req) => readHeader(req, "authorization") === "Bearer good",
		})

		const rejected = await app.fetch(
			new Request("http://local/mcp", {
				method: "POST",
				headers: { "Content-Type": "application/json", Authorization: "Bearer bad" },
				body: initBody,
			}),
		)
		expect(rejected.status).toBe(401)

		const accepted = await app.fetch(
			new Request("http://local/mcp", {
				method: "POST",
				headers: { "Content-Type": "application/json", Authorization: "Bearer good" },
				body: initBody,
			}),
		)
		expect(accepted.status).toBe(200)
	})

	test("unauthenticated by default on app.fetch", async () => {
		const app = createMcpApp({ name: "test", tools: { ping } })
		const res = await app.fetch(
			new Request("http://local/mcp", {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: initBody,
			}),
		)
		expect(res.status).toBe(200)
	})
})
