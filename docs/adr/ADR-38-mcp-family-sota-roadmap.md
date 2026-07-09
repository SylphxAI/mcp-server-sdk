# ADR-38: Adopt MCP Server SDK Family SOTA Roadmap

Date: 2026-07-09
Status: Proposed in PR #38
Slug: mcp-family-sota-roadmap

## Context

MCP Server SDK is an active TypeScript package for earlier SylphxAI MCP
servers, examples, and protocol patterns. The family needs a clear forward
runtime decision: future MCP serving should use Rust and the official
`modelcontextprotocol/rust-sdk` / `rmcp` crate rather than treating this package
as the canonical adapter layer for new products.

## Decision

Adopt `docs/roadmap/sota-family-roadmap.md` as the repo-local roadmap.

This repository remains an active legacy TypeScript SDK/reference until a
separate archive or deprecation ADR changes its lifecycle. Active product repos
should build new MCP servers with Rust and `rmcp`; this package must not become
a new TypeScript adapter dependency for the Rust-native family.

## Consequences

- Product repos must not depend on new unshipped SDK behavior for future
  Rust-native MCP serving.
- The family default is Rust MCP serving, not a TypeScript adapter layer.
- Archive or deprecation requires a superseding ADR, consumer migration policy,
  protocol parity evidence, conformance evidence, and release proof.

## Verification

- Roadmap added at `docs/roadmap/sota-family-roadmap.md`.
- README and PROJECT link to the roadmap.
- Docs-only validation: `git diff --check`.
