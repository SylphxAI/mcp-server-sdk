# ADR-DRAFT: Dogfood GroundAtlas package gate

## Status

Draft. Rename to the GitHub PR number before merge.

## Context

The MCP Server SDK has a Doctrine adapter manifest and ADR-29 validation, but public package consumers and neutral agents should not need private Sylphx Doctrine context to understand the project boundary. The repo also had no SECURITY.md and no durable ADR/spec records for its project control-plane boundary, which caused GroundAtlas strict fleet scans to warn.

## Decision

Adopt `project.manifest.json` as the vendor-neutral project manifest and run the published GroundAtlas action/package in CI using `SylphxAI/groundatlas@v0.1.2` and `groundatlas@0.1.2`.

The CI gate must assert that:

- GroundAtlas selects `project.manifest.json`.
- `.doctrine/project.json` is reported only as an adapter.
- Fleet strict mode has zero blocked projects and zero warnings.
- Generated GroundAtlas files are navigation-only and are not committed as SSOT.

Add a public `SECURITY.md` and a control-plane spec so vulnerability reporting, SSOT boundaries, and future release-proof work are durable.

## Consequences

- External users and neutral agents can discover the SDK boundary from `PROJECT.md` and `project.manifest.json`.
- Internal Sylphx governance can keep using `.doctrine/project.json` without imposing Doctrine as the public default.
- This does not close package-release proof gaps; npm readback, conformance proof, and consumer smoke evidence remain separate release-adoption work.
