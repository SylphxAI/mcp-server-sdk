# MCP Server SDK

- Lifecycle: active
- Layer: foundation
- Runtime authority: Rust workspace

## Boundary

This repository owns reusable MCP protocol, builder, client, and transport
libraries. Product-specific tools and business logic belong in consumers.

## Public surfaces

- `crates/mcp-protocol` and `crates/mcp-jsonrpc`: wire contracts
- `crates/mcp-builders` and `crates/mcp-app`: application composition
- `crates/mcp-stdio` and `crates/mcp-streamable-http`: transport adapters
- `crates/mcp-http-auth` and `crates/mcp-session-guard`: edge controls
- `crates/mcp-clients`, `crates/mcp-notifications`, and
  `crates/mcp-pagination`: protocol capabilities

## Delivery

CI runs Rust formatting, tests, and Clippy. The repository has no npm release
path and no TypeScript runtime fallback.
