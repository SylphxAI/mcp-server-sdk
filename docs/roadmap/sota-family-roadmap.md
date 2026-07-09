# SOTA Family Roadmap

Status: legacy-maintenance migration plan
Owner: MCP Server SDK
Scope: repo-local future plan and its role in the SylphxAI MCP family
Decision record: `docs/adr/ADR-38-mcp-family-sota-roadmap.md`

## Family Role

MCP Server SDK is the active legacy TypeScript SDK/reference for earlier
SylphxAI MCP servers. It preserves useful examples and typed API patterns, but
it is not the target foundation for the future product MCP family.

Until a separate deprecation or archive ADR changes its lifecycle, this
repository remains maintained for existing package truth. New product MCP
servers should not depend on new unshipped SDK behavior from this package.

## Family Fit

| Project | Relationship |
| --- | --- |
| Architecture Reader MCP | Uses Rust-native MCP serving with `modelcontextprotocol/rust-sdk` / `rmcp`. |
| CodeRAG MCP | Should migrate to Rust-native MCP serving while preserving its public tool contract. |
| Reader MCPs | Need consistent stdio/HTTP behavior, tool schemas, errors, and evidence envelopes through Rust servers. |
| Filesystem MCP | Needs strict protocol and error semantics for side-effecting tools through a Rust server. |
| Consultant MCP | Needs typed tool schemas, validation, and provider trace outputs through a Rust server. |

## SOTA End State

Active product repos should use the official Rust SDK directly:
`modelcontextprotocol/rust-sdk` / `rmcp`. This package must not sit in the
middle as a TypeScript adapter dependency for the Rust-native family.

## Roadmap

### Phase 0: Runtime Direction

- Keep this repository's current lifecycle truthful until a separate archive or
  deprecation ADR changes it.
- Document the supported product path as Rust MCP servers using `rmcp`.
- Do not let product repos adopt new unshipped behavior from this package.

### Phase 1: Reference Preservation

- Preserve useful examples as reference material.
- Mark public docs clearly where they could be mistaken for the current family
  runtime direction.
- Link family runtime decisions to Architecture Reader portfolio planning.

### Phase 2: Rust SDK Migration Notes

- Document patterns that product repos should port to Rust: evidence envelopes,
  install diagnostics, `doctor` output, stable JSON examples, and conformance
  fixtures.
- Keep downstream product logic out of this package.

### Phase 3: Decommission Option

- Keep no active roadmap that competes with `rmcp`.
- If consumer migration completes, decide through a superseding ADR whether to
  archive, deprecate, or re-scope this package.

## Validation Gates

- Docs do not claim this is the active future family runtime.
- Product repos consume `rmcp` or their own Rust server crates, not this legacy
  package as a new adapter dependency.
- Archive/deprecation status is not changed without a superseding ADR, CI,
  release, and consumer migration gates.
