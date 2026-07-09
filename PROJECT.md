# MCP Server SDK

MCP Server SDK is a TypeScript package for building Model Context Protocol
servers on Bun and Node.js. It owns the pure functional server API, builders,
schema integration, JSON-RPC/MCP protocol handling, stdio and streamable HTTP
transports, examples, tests, conformance support, and package release path.

## Lifecycle

- State: `active`
- Layer: `foundation`
- Vendor-neutral manifest: [`project.manifest.json`](./project.manifest.json)
- Doctrine adapter manifest: [`.doctrine/project.json`](./.doctrine/project.json)

## Goals

- Provide a type-safe, composable SDK for MCP tools, resources, prompts,
  notifications, sampling, elicitation, pagination, and transports.
- Keep package exports, examples, tests, conformance behavior, and release
  automation aligned with the supported MCP protocol surface.
- Use central ADR-29 admission and release workflows without embedding
  enterprise process policy in this repository.

## Non-Goals

- This repository does not own the MCP specification, third-party client
  behavior, or consumer server product logic.
- This repository does not own downstream package-specific tools, prompts,
  resources, or customer integrations.
- This repository does not own enterprise doctrine, manifest schema, org
  rulesets, or shared workflow implementation.

## Boundary

The repository owns the `@sylphx/mcp-server-sdk` package source, tests,
examples, README, changelog, CI, and release workflow hooks. Consumers must use
the published package exports and documented examples rather than internal file
paths.

## Public Surfaces

- Package exports and metadata: `package.json`
- SDK source exports: `src/index.ts`
- README and examples: `README.md`, `examples/`
- SOTA family roadmap: `docs/roadmap/sota-family-roadmap.md`
- Test and conformance surface: `src/**/*.test.ts`, `examples/conformance-server.ts`
- CI and release workflows: `.github/workflows/ci.yml`, `.github/workflows/release.yml`
- Human project orientation: `PROJECT.md`
- Vendor-neutral project manifest: `project.manifest.json`
- Doctrine adapter manifest: `.doctrine/project.json`

## Delivery

- CI model: `adr29-admission`
- Required contexts: `risk-classification/pass`, `ci`, `trunk-admission/pass`
- Deploy/release path: pull requests and merge groups run ADR-29 admission plus
  lint, typecheck, test, and build; `main` uses the central release workflow.
- Production proof: postsubmit ADR-29 proof exists; GroundAtlas package dogfood
  proves the project-control boundary with JSON and Markdown evidence/read
  models, but npm/package readback and consumer smoke evidence are not yet
  documented as a complete release gate.
- Recovery class: `forward-fix-only`

Adoption is migrating. The current gaps are tracked in
`.doctrine/project.json`; the vendor-neutral GroundAtlas gate verifies
`project.manifest.json` as the public control-plane manifest while keeping
`.doctrine/project.json` as the Sylphx adapter. Generated `.groundatlas*` files
and GroundAtlas JSON/Markdown reports are evidence/read models only, not
authoritative project-control inputs.
