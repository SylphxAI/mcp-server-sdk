# MCP Server SDK

- Lifecycle: archived
- Layer: foundation
- Runtime authority: Rust workspace

## Boundary

This repository is retained as read-only history. It no longer owns an active
SDK or delivery surface. Consumers must use a maintained implementation.

## Public surfaces

- `crates/mcp-protocol` and `crates/mcp-jsonrpc`: wire contracts
- `crates/mcp-builders` and `crates/mcp-app`: application composition
- `crates/mcp-stdio` and `crates/mcp-streamable-http`: transport adapters
- `crates/mcp-http-auth` and `crates/mcp-session-guard`: edge controls
- `crates/mcp-clients`, `crates/mcp-notifications`, and
  `crates/mcp-pagination`: protocol capabilities

## Delivery

No builds or releases are produced after retirement.
