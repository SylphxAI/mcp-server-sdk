# Repository Instructions

Engineering doctrine: https://github.com/SylphxAI/doctrine. Read doctrine
`AGENTS.md`, `PRINCIPLES.md`, and `ADR.md`; load doctrine `standards/*.md` when
the task triggers them.

Read [PROJECT.md](./PROJECT.md) and
[.doctrine/project.json](./.doctrine/project.json) before changing behavior,
CI, delivery, documentation, public surfaces, persistence, security posture, or
cross-repository integrations.

This repository owns the `@sylphx/mcp-server-sdk` package only. Keep consumer
server business logic and product-specific tools/resources/prompts out of SDK
core. Consumers must rely on package exports and documented examples, not
internal source paths.

Generated `.groundatlas*` files and GroundAtlas JSON/Markdown reports are
evidence/read models only. Do not treat them as project-control SSOT; update
`project.manifest.json` or `.doctrine/project.json` at the owning boundary.
