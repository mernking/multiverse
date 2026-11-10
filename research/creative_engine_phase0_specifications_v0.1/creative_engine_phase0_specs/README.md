# Creative Engine — Phase 0 Specification Pack

**Status:** Draft 0.1 — for owner review  
**Date:** 10 November 2026  
**Purpose:** Establish the product, architecture, engineering constraints, and acceptance criteria before production implementation.

This is a complete first-pass specification pack for a cross-platform, vector-first creative canvas application inspired by the capabilities of Concepts, but implemented independently. It is intended to be reviewed document by document and revised into an approved baseline before production coding.

## Product scope

- Full-featured infinite canvas for drawing, sketching, diagrams, visual thinking, technical illustration, and design.
- Linux-first development and desktop release, with Windows and macOS support.
- Android and iOS clients using the same Rust application and document core.
- Rust shared core and application command API.
- Iced desktop interface; native mobile shells with shared Rust core and renderer integration.
- GPU-accelerated rendering with a defined software/fallback path.
- CLI for project/document operations, automation, batch processing, and agent use.
- MCP server exposing controlled application capabilities to external MCP clients.
- Built-in AI agent with OpenAI and Anthropic provider adapters, brainstorming, document understanding, and approved canvas operations.
- Offline-first drawing and local document storage. Network features are optional and explicit.

## Pack contents

1. `PRODUCT_REQUIREMENTS.md`
2. `FEATURE_INVENTORY.md`
3. `UI_UX_SPECIFICATION.md`
4. `RUST_WORKSPACE_ARCHITECTURE.md`
5. `CANVAS_ENGINE_SPECIFICATION.md`
6. `VECTOR_GEOMETRY_SPECIFICATION.md`
7. `RENDERING_ENGINE_SPECIFICATION.md`
8. `BRUSH_ENGINE_SPECIFICATION.md`
9. `DOCUMENT_FORMAT_SPECIFICATION.md`
10. `CROSS_PLATFORM_ARCHITECTURE.md`
11. `CLI_SPECIFICATION.md`
12. `MCP_SPECIFICATION.md`
13. `AI_AGENT_SPECIFICATION.md`
14. `BRAINSTORMING_ENGINE_SPECIFICATION.md`
15. `IMPORT_EXPORT_SPECIFICATION.md`
16. `TESTING_AND_ACCEPTANCE_CRITERIA.md`
17. `DEVELOPMENT_ROADMAP.md`

## How to review

Review one document at a time. Mark each requirement as:
- **Approved** — accepted as written.
- **Change requested** — specify the required change.
- **Deferred** — intentionally moved to a later release.
- **Rejected** — not part of the product.

No production feature should be considered complete until its specification, dependencies, tests, and acceptance criteria are approved.

## Baseline assumptions requiring owner confirmation

These are explicit assumptions, not hidden decisions:
1. Working product name is **Creative Engine** only as a temporary project label.
2. The native editable format will be a versioned `.ceng` document/container.
3. Local-first storage is the default; cloud sync is not required for the first production release.
4. The application is primarily 2D and vector-first, with raster images and raster brush layers supported.
5. AI API keys are supplied by the user and stored using the operating system's secure credential facility where available.
6. The first public release targets Linux, Windows, macOS, and Android; iOS is a planned target whose release timing depends on native integration and distribution constraints.
7. Iced is the desktop UI framework, not a requirement for the mobile UI shell.
8. The initial release is single-user/local. Collaboration and hosted accounts are not assumed.

## Global engineering principles

- Reimplement independently; do not copy proprietary source code, artwork, brush assets, branding, or protected UI assets.
- Keep document semantics independent of UI widgets and GPU resources.
- Treat all external inputs (files, plugins, MCP calls, AI output) as untrusted.
- Every meaningful edit is a command with validation, undo semantics, and test coverage.
- Use deterministic document operations where practical.
- Never make AI output silently destructive.
- Preserve forward compatibility: unknown optional document fields should be retained when possible.
- Version every public API surface: document schema, CLI interface, MCP tools, plugin API.
- Maintain a written decision log for architectural choices and deviations.
