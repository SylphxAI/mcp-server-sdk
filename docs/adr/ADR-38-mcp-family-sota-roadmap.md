# ADR-38: Adopt MCP Server SDK Family SOTA Roadmap

Date: 2026-07-09
Status: Proposed in PR #38
Slug: mcp-family-sota-roadmap

## Context

MCP Server SDK is an archived TypeScript package for earlier SylphxAI MCP
servers. Active products need clarity that future MCP serving should use Rust
and the official `modelcontextprotocol/rust-sdk` / `rmcp` crate rather than this
package as a canonical adapter layer.

## Decision

Adopt `docs/roadmap/sota-family-roadmap.md` as the repo-local roadmap.

This repository should remain archived as a historical reference unless a future
ADR gives it a new non-adapter purpose. Active product repos should build Rust
MCP servers with `rmcp`.

## Consequences

- Product repos must not depend on new unshipped SDK behavior while archived.
- The family default is Rust MCP serving, not a TypeScript adapter layer.
- Reactivation requires a superseding ADR, protocol parity, conformance
  evidence, release proof, and clear consumer migration policy.

## Verification

- Roadmap added at `docs/roadmap/sota-family-roadmap.md`.
- README and PROJECT link to the roadmap.
- Docs-only validation: `git diff --check`.
