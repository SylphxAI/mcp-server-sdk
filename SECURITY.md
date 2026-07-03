# Security Policy

## Supported surface

This repository publishes the `@sylphx/mcp-server-sdk` npm package, examples, tests, CI workflows, release hooks, and project-control manifests. It does not operate a hosted runtime, customer data plane, or downstream product-specific MCP server.

## Reporting a vulnerability

Please report security issues privately through GitHub Security Advisories for this repository when available. If advisories are unavailable, contact the repository maintainers through the organization-owned security contact listed on the SylphxAI GitHub organization profile.

Do not open public issues containing secrets, exploit details, customer data, or unreleased vulnerability information.

## Security expectations

- Do not commit secrets, private keys, credentials, tokens, customer data, or private runtime configuration.
- Do not encode downstream product-specific tools, resources, prompts, or customer logic into the SDK core.
- Keep consumers on declared package exports instead of internal source paths.
- Keep project-control facts in `PROJECT.md`, `project.manifest.json`, `.doctrine/project.json`, specs, ADRs, package metadata, tests, and CI instead of chat or generated navigation files.

## Validation

Security-relevant changes should pass the repo CI plus the GroundAtlas package gate:

```bash
bun run lint
bun run typecheck
bun test
bun run build
npm exec --yes --package groundatlas@0.1.2 -- ga fleet . --out .groundatlas-pilot --require-atlas --strict --json
```
