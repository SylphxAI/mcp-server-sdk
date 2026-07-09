# SOTA Family Roadmap

Status: archived adoption plan
Owner: MCP Server SDK
Scope: repo-local future plan and its role in the SylphxAI MCP family
Decision record: `docs/adr/ADR-38-mcp-family-sota-roadmap.md`

## Family Role

MCP Server SDK is the archived TypeScript SDK/reference for earlier SylphxAI MCP
servers. It can preserve historical examples and typed API patterns, but it is
not the target foundation for the future product MCP family.

The repository is currently archived. Until reactivated, active product repos
must not depend on new unshipped SDK behavior.

## Family Fit

| Project | Relationship |
| --- | --- |
| Architecture Reader MCP | Uses Rust-native MCP serving with `modelcontextprotocol/rust-sdk` / `rmcp`. |
| CodeRAG MCP | Should migrate to Rust-native MCP serving while preserving its public tool contract. |
| Reader MCPs | Need consistent stdio/HTTP behavior, tool schemas, errors, and evidence envelopes through Rust servers. |
| Filesystem MCP | Needs strict protocol and error semantics for side-effecting tools through a Rust server. |
| Consultant MCP | Needs typed tool schemas, validation, and provider trace outputs through a Rust server. |

## SOTA End State

The SDK should remain archived unless a future ADR explicitly re-scopes it.
Active product repos should use the official Rust SDK directly:
`modelcontextprotocol/rust-sdk` / `rmcp`. It must not sit in the middle as an
unmaintained TypeScript adapter dependency.

## Roadmap

### Phase 0: Archive Decision

- Keep this repository archived as the historical TypeScript SDK/reference.
- Document the supported product path as Rust MCP servers using `rmcp`.
- Do not let product repos adopt new unshipped behavior from this package.

### Phase 1: Reference Preservation

- Preserve useful examples only as reference material.
- Mark public docs clearly as archived where they could be mistaken for the
  current family SDK.
- Link family runtime decisions to Architecture Reader portfolio planning.

### Phase 2: Rust SDK Migration Notes

- Document patterns that product repos should port to Rust: evidence envelopes,
  install diagnostics, `doctor` output, stable JSON examples, and conformance
  fixtures.
- Keep downstream product logic out of this archived package.

### Phase 3: Final Archive Hygiene

- Keep no active roadmap that competes with `rmcp`.
- Keep repository archive status unless a superseding ADR reactivates it with a
  new non-adapter purpose.

## Validation Gates

- Archived docs do not claim this is the active family SDK.
- Product repos consume `rmcp` or their own Rust server crates, not this archived
  package.
- Archived status is not lifted without a superseding ADR, CI, release, and
  consumer migration gates.
