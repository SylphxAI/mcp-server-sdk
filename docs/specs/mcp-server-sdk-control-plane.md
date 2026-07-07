# MCP Server SDK Control Plane Spec

## Purpose

`@sylphx/mcp-server-sdk` is a public foundation package for building Model Context Protocol servers. Its project control plane must be discoverable by humans, agents, CI, release automation, and external package consumers without requiring private Sylphx Doctrine knowledge.

## Source of truth

- `PROJECT.md` is the human-readable project boundary, lifecycle, public surfaces, delivery, and adoption entry point.
- `project.manifest.json` is the vendor-neutral machine-readable manifest selected by GroundAtlas.
- `.doctrine/project.json` is the Sylphx Doctrine adapter and organization-local governance catalog. It must remain an adapter, not the vendor-neutral default.
- `package.json` owns npm package exports, package metadata, validation scripts, and published package identity.
- `README.md`, examples, and tests own the public SDK usage contract until generated API reference documentation is introduced.

## Required GroundAtlas gate

The CI workflow must run the published GroundAtlas package/action on pull requests, merge-group candidates, and `main` pushes. The gate must assert:

1. `project.manifest.json` is selected as the vendor-neutral manifest.
2. `.doctrine/project.json` is detected only as an adapter.
3. Fleet strict mode reports `1 adopted, 0 warning, 0 blocked, 1 total`.
4. The Markdown fleet scorecard exists and includes the report title and summary.
5. Generated GroundAtlas JSON and Markdown output remains navigation-only evidence/read models and is not committed as SSOT.

The workflow must upload `groundatlas-package-dogfood` with:

- `groundatlas-manifest.json`
- `groundatlas-fleet.json`
- `groundatlas-fleet.md`

## Release-proof boundary

This spec does not close the existing package-release adoption gaps. The SDK still needs durable release proof that links npm registry readback, MCP conformance evidence, and consumer smoke evidence after publication. That work remains tracked in `.doctrine/project.json` until a release-proof ADR/spec closes it.
