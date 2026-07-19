# MCP Server SDK

Rust libraries for building Model Context Protocol servers.

## Capabilities

- MCP and JSON-RPC protocol types
- tool, resource, and prompt builders
- stdio and streamable HTTP transports
- API-key authentication and session guards
- sampling, elicitation, notifications, and pagination

The workspace is split by capability under `crates/`. Applications compose
only the crates they need; transport adapters depend on protocol and domain
crates, never the reverse.

## Validate

```sh
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

The retired TypeScript implementation is available from Git history. It is not
a second runtime authority.
