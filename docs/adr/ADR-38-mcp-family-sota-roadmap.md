# ADR-38: Adopt MCP Server SDK Family SOTA Roadmap

Date: 2026-07-09
Status: Proposed in PR #38
Slug: mcp-family-sota-roadmap

## Context

MCP Server SDK is a foundation-layer package for SylphxAI MCP servers, but the
repository is archived. Active products need clarity on whether this SDK is a
canonical adapter layer or an archived reference.

## Decision

Adopt `docs/roadmap/sota-family-roadmap.md` as the repo-local roadmap.

The SDK should either be reactivated with CI, release, conformance, and npm
readback gates, or remain archived while active product repos use another
supported SDK path.

## Consequences

- Product repos must not depend on new unshipped SDK behavior while archived.
- Reactivation requires protocol parity, conformance evidence, package release
  proof, and clear consumer migration policy.
- The SDK may provide shared adapter conventions but must not contain product
  tool logic.

## Verification

- Roadmap added at `docs/roadmap/sota-family-roadmap.md`.
- README and PROJECT link to the roadmap.
- Docs-only validation: `git diff --check`.
