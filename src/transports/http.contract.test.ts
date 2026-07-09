/**
 * Streamable HTTP transport contract tests.
 *
 * Golden fixtures in fixtures/streamable-http/ are the fleet SSOT for MCP Web
 * transport parity (Rust rmcp shells must match these scenarios).
 */

import { afterEach, describe, expect, test } from "bun:test"
import { readFileSync } from "node:fs"
import { createServer } from "node:net"
import path from "node:path"
import { createMcpApp } from "../app/app.js"
import { serve } from "../app/serve.js"

// ============================================================================
// Types
// ============================================================================

interface ContractManifest {
	readonly schemaVersion: number
	readonly contract: string
	readonly spec: string
	readonly scenarios: ReadonlyArray<{
		readonly id: string
		readonly fixture: string
		readonly description: string
	}>
}

interface FixtureAuth {
	readonly apiKey?: string
}

interface FixtureRequest {
	readonly method: string
	readonly path: string
	readonly headers?: Record<string, string>
	readonly body?: unknown
}

interface FixtureResponse {
	readonly status: number
	readonly headers?: Record<string, string | { readonly pattern: string }>
	readonly body?: unknown
}

interface TransportFixture {
	readonly request: FixtureRequest
	readonly auth?: FixtureAuth
	readonly response: FixtureResponse
}

// ============================================================================
// Paths + helpers
// ============================================================================

const REPO_ROOT = path.resolve(import.meta.dirname, "../..")
const FIXTURE_DIR = path.join(REPO_ROOT, "fixtures/streamable-http")
const MANIFEST_PATH = path.join(FIXTURE_DIR, "contract-manifest.json")
const SPEC_PATH = path.join(REPO_ROOT, "docs/specs/streamable-http-transport-contract.md")

const SERVER_NAME = "contract-fixture-server"
const SERVER_VERSION = "0.0.0-contract"
const HOST = "127.0.0.1"

const loadManifest = (): ContractManifest => JSON.parse(readFileSync(MANIFEST_PATH, "utf8")) as ContractManifest

const loadFixture = (fileName: string): TransportFixture =>
	JSON.parse(readFileSync(path.join(FIXTURE_DIR, fileName), "utf8")) as TransportFixture

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

const isPattern = (value: unknown): value is { readonly pattern: string } =>
	typeof value === "object" &&
	value !== null &&
	"pattern" in value &&
	typeof (value as { pattern: unknown }).pattern === "string"

const isContains = (value: unknown): value is { readonly contains: string } =>
	typeof value === "object" &&
	value !== null &&
	"contains" in value &&
	typeof (value as { contains: unknown }).contains === "string"

const matchValue = (actual: unknown, expected: unknown, label: string): void => {
	if (isPattern(expected)) {
		expect(String(actual)).toMatch(new RegExp(expected.pattern))
		return
	}
	if (isContains(expected)) {
		expect(String(actual)).toContain(expected.contains)
		return
	}
	if (
		typeof expected === "object" &&
		expected !== null &&
		!Array.isArray(expected) &&
		typeof actual === "object" &&
		actual !== null &&
		!Array.isArray(actual)
	) {
		for (const [key, nested] of Object.entries(expected)) {
			matchValue((actual as Record<string, unknown>)[key], nested, `${label}.${key}`)
		}
		return
	}
	expect(actual).toEqual(expected)
}

const assertResponseMatchesFixture = async (res: Response, fixture: TransportFixture): Promise<void> => {
	expect(res.status).toBe(fixture.response.status)

	if (fixture.response.headers) {
		for (const [name, expected] of Object.entries(fixture.response.headers)) {
			const actual = res.headers.get(name)
			if (isPattern(expected)) {
				expect(actual).not.toBeNull()
				expect(actual as string).toMatch(new RegExp(expected.pattern))
			} else {
				expect(actual?.toLowerCase()).toBe(String(expected).toLowerCase())
			}
		}
	}

	if (fixture.response.body !== undefined) {
		const contentType = res.headers.get("content-type") ?? ""
		if (contentType.includes("application/json")) {
			const body = (await res.json()) as unknown
			matchValue(body, fixture.response.body, "body")
		} else {
			const text = await res.text()
			matchValue(text, fixture.response.body, "body")
		}
	}
}

const runFixtureOnServe = async (fixture: TransportFixture, port: number): Promise<Response> => {
	const url = `http://${HOST}:${port}${fixture.request.path}`
	const headers = { ...fixture.request.headers }
	const init: RequestInit = { method: fixture.request.method, headers }

	if (fixture.request.body !== undefined) {
		init.body = typeof fixture.request.body === "string" ? fixture.request.body : JSON.stringify(fixture.request.body)
	}

	return fetch(url, init)
}

// ============================================================================
// Contract structure
// ============================================================================

describe("streamable HTTP contract structure", () => {
	test("manifest and spec exist with required scenarios", () => {
		const manifest = loadManifest()
		const spec = readFileSync(SPEC_PATH, "utf8")

		expect(manifest.schemaVersion).toBe(1)
		expect(manifest.contract).toBe("streamable-http-transport")
		expect(manifest.scenarios.length).toBeGreaterThanOrEqual(8)

		for (const scenario of manifest.scenarios) {
			expect(() => loadFixture(scenario.fixture)).not.toThrow()
		}

		expect(spec).toContain("## Canonical routes")
		expect(spec).toContain("## Auth contract")
		expect(spec).toContain("## Fleet rmcp implementation target")
		expect(spec).toContain("/mcp/health")
		expect(spec).toContain("Mcp-Session-Id")
		expect(spec).toContain("-32001")
	})
})

// ============================================================================
// Golden parity — serve() entrypoint (TS SSOT)
// ============================================================================

describe("streamable HTTP golden fixtures (serve)", () => {
	let stop: (() => Promise<void>) | null = null

	afterEach(async () => {
		await stop?.()
		stop = null
	})

	const startServer = async (apiKey?: string) => {
		const port = await getFreePort()
		const app = createMcpApp({
			name: SERVER_NAME,
			version: SERVER_VERSION,
		})
		const server = await serve({
			app,
			port,
			hostname: HOST,
			...(apiKey ? { apiKey } : {}),
		})
		stop = server.stop
		return port
	}

	const manifest = loadManifest()

	for (const scenario of manifest.scenarios) {
		test(`${scenario.id}: ${scenario.description}`, async () => {
			const fixture = loadFixture(scenario.fixture)
			const port = await startServer(fixture.auth?.apiKey)
			const res = await runFixtureOnServe(fixture, port)
			await assertResponseMatchesFixture(res, fixture)
		})
	}
})
