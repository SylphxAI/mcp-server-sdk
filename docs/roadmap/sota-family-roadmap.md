# SOTA Family Roadmap

Status: archived adoption plan
Owner: MCP Server SDK
Scope: repo-local future plan and its role in the SylphxAI MCP family
Decision record: `docs/adr/ADR-38-mcp-family-sota-roadmap.md`

## Family Role

MCP Server SDK is the foundation-layer TypeScript SDK for building SylphxAI MCP
servers. It can provide a consistent typed server API, transport behavior,
schema integration, examples, conformance support, and release discipline for
the product MCP family.

The repository is currently archived. Until reactivated, active product repos
must not depend on new unshipped SDK behavior.

## Family Fit

| Project | Relationship |
| --- | --- |
| Architecture Reader MCP | May use the SDK for TypeScript adapter ergonomics while the Rust core matures. |
| CodeRAG MCP | Can use SDK patterns for stable tool schemas and transport behavior. |
| Reader MCPs | Need consistent stdio/HTTP behavior, tool schemas, errors, and evidence envelopes. |
| Filesystem MCP | Needs strict protocol and error semantics for side-effecting tools. |
| Consultant MCP | Needs typed tool schemas, validation, and provider trace outputs. |

## SOTA End State

The SDK should either be reactivated as the canonical SylphxAI MCP TypeScript
adapter layer or remain archived while product repos use the official SDK
directly. It must not sit in the middle as an unmaintained dependency.

## Roadmap

### Phase 0: Reactivation Decision

- Decide whether this repository remains archived or becomes the active family
  SDK.
- If it remains archived, document product repos' supported SDK path elsewhere.
- If reactivated, restore CI, release, conformance, npm readback, and docs
  gates before product repos adopt new features.

### Phase 1: Protocol And Schema Parity

- Track MCP protocol versions explicitly.
- Generate schema examples and conformance fixtures.
- Standardize error envelopes, logging, transport selection, stdio behavior,
  streamable HTTP behavior, and auth hooks.

### Phase 2: Product Family Conventions

- Provide helpers for evidence envelopes, install diagnostics, `doctor` output,
  and stable JSON examples without hardcoding product behavior.
- Keep downstream tool logic out of the SDK.

### Phase 3: Rust Adapter Interop

- Define adapter boundaries for Rust-core projects that expose local binaries
  behind TypeScript MCP adapters.
- Support transition paths from TypeScript adapter to direct Rust MCP server
  without changing product tool contracts.

## Validation Gates

- Conformance suite passes.
- SDK examples compile and run.
- Product repos consume only published package exports.
- Release workflow proves npm package readback.
- Archived status is not lifted without CI and release gates.
