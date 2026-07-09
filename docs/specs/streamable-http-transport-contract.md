# Streamable HTTP Transport Contract

Status: **contracted** (fleet SSOT for MCP Web transport parity)
Owner: `@sylphx/mcp-server-sdk`
Protocol: MCP Streamable HTTP (`2025-03-26`)
Decision gate: Rust-First D-006 — MCP repos must prove parity against this contract before deleting TS stdio adapters.

## Purpose

This contract is the cross-repo boundary for Web MCP HTTP transport. The TypeScript
implementations in this package (`serve()`, `http()`, `createMcpApp().fetch`) are the
**baseline** until a schema-derived Rust bridge ships. Fleet MCP servers (rmcp
`StreamableHttpService` shells) must match the golden fixtures in
`fixtures/streamable-http/`.

## Canonical routes

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `{basePath}/health` | Open | Liveness probe; never gated by `apiKey` / `authenticate`. |
| `POST` | `{basePath}` | Opt-in | JSON-RPC ingress (JSON or SSE response). |
| `OPTIONS` | `{basePath}`, `{basePath}/health` | Open | CORS preflight when `cors` is configured. |

Defaults: `basePath = /mcp`, `port = 3000`, `hostname = localhost`.

## Request headers

| Header | Required | Notes |
|--------|----------|-------|
| `Content-Type` | POST | `application/json` |
| `Accept` | POST | `application/json` (default) or `text/event-stream` (SSE + bidirectional RPC). |
| `Mcp-Session-Id` | Optional | Required after `initialize` for session-scoped bidirectional RPC. |
| `X-API-Key` | When `apiKey` set | Constant-time SHA-256 digest comparison (see Auth). |

## Response headers

| Header | When | Notes |
|--------|------|-------|
| `Mcp-Session-Id` | `initialize` | New UUID session created on first `initialize` in a transport context. |
| `WWW-Authenticate` | `401` | `Bearer realm="mcp", charset="UTF-8"` |
| `Content-Type` | Always | `application/json` or `text/event-stream` |

## Status codes

| Code | Scenario |
|------|----------|
| `200` | Successful JSON-RPC response |
| `202` | Client JSON-RPC **response** ack for a pending server-initiated request (bidirectional RPC) |
| `204` | Notification handled (no response body) |
| `400` | JSON parse / protocol parse error (`serve()` / `fetch` adapters) |
| `401` | Auth configured and request rejected |
| `404` | Unknown `Mcp-Session-Id` |
| `405` | Non-POST to JSON-RPC path (`fetch` adapter) |
| `500` | Unhandled server error |

## Auth contract

Authentication is **opt-in**. When disabled, all MCP POST requests are allowed.

Precedence: `authenticate` hook > `apiKey` > allow.

### `apiKey` mode

- Header: `X-API-Key`
- Comparison: SHA-256 digest of expected and provided keys, compared with constant-time equality.
- Missing, empty, or wrong keys → `401` with body:

```json
{
  "jsonrpc": "2.0",
  "id": null,
  "error": { "code": -32001, "message": "Unauthorized" }
}
```

### Health bypass

`GET {basePath}/health` is always open, even when auth is enabled.

## Health response shape

```json
{
  "status": "ok",
  "server": "<server.name>",
  "version": "<server.version>"
}
```

## Session lifecycle

1. Client sends `initialize` without `Mcp-Session-Id`.
2. Server responds with `Mcp-Session-Id: <uuid>` header and JSON-RPC result.
3. Subsequent requests in the same session include `Mcp-Session-Id`.
4. Unknown session id → `404` `{ "error": "Session not found" }`.
5. Bidirectional RPC (SSE path): server may emit JSON-RPC requests on the SSE stream; client answers via POST with the same session id → `202` empty body.

## SSE contract (Accept: text/event-stream)

- Response `Content-Type`: `text/event-stream`
- Events use `event: message` with JSON-RPC payloads in `data`
- Final event carries the JSON-RPC response for the inbound request
- Notifications may be interleaved before the final response

## CORS contract

When `cors` option is set:

- `Access-Control-Allow-Origin`: configured origin
- `Access-Control-Allow-Headers`: `Content-Type`, `Accept`, `Mcp-Session-Id` (+ `X-API-Key` on fleet rmcp shells)
- `Access-Control-Expose-Headers`: `Mcp-Session-Id`
- Preflight `OPTIONS` → `204`

## Fleet rmcp implementation target

MCP product repos should implement this contract with:

- `rmcp::transport::streamable_http_server::tower::StreamableHttpService`
- Axum router nesting at `/mcp`
- `GET /mcp/health` returning `{ "status": "ok" }` (server/version optional until parity slice)
- Env routing: `MCP_TRANSPORT=http` (+ repo-prefixed alias, e.g. `FILESYSTEM_MCP_TRANSPORT=http`)

## Parity proof

| Artifact | Role |
|----------|------|
| `fixtures/streamable-http/contract-manifest.json` | Machine-readable scenario index |
| `fixtures/streamable-http/*.json` | Golden request/response fixtures |
| `src/transports/http.contract.test.ts` | TS SSOT conformance tests |
| `src/transports/auth.test.ts` | Auth unit + integration tests |

Rust implementations prove parity by matching golden fixtures (same inputs → same status, headers, and body shapes). Dynamic fields (`Mcp-Session-Id` values) are validated by pattern, not literal equality.

## TS implementation surfaces (current authority)

- `src/app/serve.ts` — canonical `createMcpApp` + gust server
- `src/app/http.ts` — Fetch adapter (`app.fetch`)
- `src/transports/http.ts` — legacy `http()` factory
- `src/transports/auth.ts` — shared auth module