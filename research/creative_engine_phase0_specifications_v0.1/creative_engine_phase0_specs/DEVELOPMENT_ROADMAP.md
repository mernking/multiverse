# 17. Development Roadmap

**Status:** Draft 0.1

## 17.1 Delivery philosophy

Build a complete product through verified vertical slices. Do not postpone all mobile, CLI, MCP, or AI architecture until the end. Establish shared interfaces early, then implement capabilities incrementally.

## 17.2 Phase 0 — specification and risk reduction

Deliverables:
- this 17-document specification pack;
- feature and platform matrix;
- architecture decision record (ADR) log;
- acceptance criteria;
- dependency/license review;
- prototype risk report.

Exit criteria:
- owner has reviewed and approved requirements;
- high-risk technical spikes have results;
- no unresolved architecture question blocks Phase 1.

## 17.3 Phase 1 — workspace and core contracts

- Rust workspace and CI.
- Domain model skeleton.
- Application command/query API.
- Document IDs, object IDs, errors, transactions.
- Platform abstraction traits.
- CLI/MCP/AI tool interfaces defined.
- Basic headless document test.

## 17.4 Phase 2 — desktop shell and canvas proof

- Iced desktop application.
- wgpu integration proof.
- Camera and coordinate transforms.
- Basic pointer navigation.
- Render a small set of primitives.
- Linux packaging development path.

## 17.5 Phase 3 — document and geometry foundation

- Native document schema.
- Paths, shapes, transforms.
- Save/load, atomic writes, autosave, recovery.
- Undo/redo.
- Spatial index and hit testing.

## 17.6 Phase 4 — drawing and editing

- Input normalization.
- Stylus pressure pipeline.
- Brush presets and stroke generation.
- Selection and transforms.
- Layers, groups, object inspector.

## 17.7 Phase 5 — precision and visual organization

- Grids, snapping, guides, measurements.
- Artboards/pages.
- Object library and reusable assets.
- Text and connectors.

## 17.8 Phase 6 — import/export

- Raster, SVG, PDF import/export.
- Native format validation/migration.
- Clipboard.
- Conditional PSD/DXF feasibility gate.

## 17.9 Phase 7 — CLI

- Document inspection/validation.
- Batch export and conversion.
- Object/layer operations.
- JSON output contract.

## 17.10 Phase 8 — MCP

- Local stdio server.
- Read tools/resources.
- Permission-gated write tools.
- Protocol conformance and security tests.

## 17.11 Phase 9 — AI providers and agent

- OpenAI adapter.
- Anthropic adapter.
- Provider settings and secure key storage.
- Context engine.
- Tool registry, plan preview, approval, execution.
- Audit and undo integration.

## 17.12 Phase 10 — brainstorming

- Mind maps, concept maps, diagrams.
- Layout engine.
- Partial acceptance and editability.
- Agent-to-canvas workflow.

## 17.13 Phase 11 — mobile

- Android host, FFI, renderer surface, stylus/touch, file picker.
- iOS host and Metal integration.
- Adaptive UI and mobile storage.
- Cross-platform document parity.

## 17.14 Phase 12 — hardening and release

- Performance optimization.
- Accessibility.
- Security review.
- Crash recovery testing.
- Installers, signing, packaging, release notes.
- User documentation and support workflow.

## 17.15 Critical early technical spikes

Must happen before committing to full implementation:
1. Iced + wgpu integration on Linux.
2. Android Rust core + GPU surface + stylus input.
3. iOS Rust core + Metal surface feasibility.
4. Vector geometry library comparison.
5. Native file container benchmark.
6. PDF/SVG import/export fidelity prototype.
7. Brush latency and stroke-quality prototype.
8. MCP SDK/protocol implementation review.
9. OpenAI and Anthropic adapter conformance.
10. Secure credential storage on each OS.

## 17.16 Dependencies and parallel work

Geometry, document model, and command API are foundational. Renderer and input can proceed against stable snapshots/events. CLI and MCP can begin once application commands exist. AI provider work can proceed in parallel with canvas implementation, but AI canvas execution must wait for stable command semantics. Mobile prototypes should begin early to expose integration risks, even though full mobile UI follows desktop foundation.

## 17.17 Definition of done

A feature is done only when:
- requirements and edge cases are documented;
- implementation is in the correct crate;
- errors and cancellation are handled;
- undo/redo is defined where applicable;
- tests pass;
- cross-platform impact is checked;
- accessibility and security impact is reviewed;
- documentation is updated.
