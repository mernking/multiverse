# Creative Engine — Implementation Plan

**Status:** Act Mode — Implementation Ready
**Date:** 10 November 2026
**Based on:** Phase 0 Specification Pack (17 documents) + Research Documents

---

## Executive Summary

This plan outlines the implementation of **Creative Engine**, a cross-platform vector drawing and creative workspace application inspired by Concepts. The application will feature:

- **Infinite canvas** with vector-first document model
- **GPU-accelerated rendering** (wgpu) with software fallback
- **Desktop UI** via Iced (Linux, Windows, macOS)
- **Mobile apps** via native shells + shared Rust core (Android, iOS)
- **CLI** for automation and batch operations
- **MCP server** for AI tool integration
- **Built-in AI agent** with OpenAI/Anthropic providers

**Key Architectural Principle:** All frontends (Desktop, CLI, MCP, Mobile, AI Agent) share the same Rust core — domain model, application commands, geometry, rendering, and document engine.

---

## Phase 0 Status: Specification Review Complete ✓

The 17 specification documents in `research/creative_engine_phase0_specifications_v0.1/creative_engine_phase0_specs/` are complete as Draft 0.1. They need owner review before implementation begins.

**Documents requiring review:**
1. README.md
2. PRODUCT_REQUIREMENTS.md
3. FEATURE_INVENTORY.md
4. UI_UX_SPECIFICATION.md
5. RUST_WORKSPACE_ARCHITECTURE.md
6. CANVAS_ENGINE_SPECIFICATION.md
7. VECTOR_GEOMETRY_SPECIFICATION.md
8. RENDERING_ENGINE_SPECIFICATION.md
9. BRUSH_ENGINE_SPECIFICATION.md
10. DOCUMENT_FORMAT_SPECIFICATION.md
11. CROSS_PLATFORM_ARCHITECTURE.md
12. CLI_SPECIFICATION.md
13. MCP_SPECIFICATION.md
14. AI_AGENT_SPECIFICATION.md
15. BRAINSTORMING_ENGINE_SPECIFICATION.md
16. IMPORT_EXPORT_SPECIFICATION.md
17. TESTING_AND_ACCEPTANCE_CRITERIA.md
18. DEVELOPMENT_ROADMAP.md

**Action Required:** Owner reviews each document and marks: Approved / Change Requested / Deferred / Rejected

---

## Implementation Phases Overview

Based on DEVELOPMENT_ROADMAP.md, the implementation proceeds in 21 phases. We'll start with the critical foundation phases.

### Phase 1: Workspace & Core Contracts (Weeks 1-3)
**Goal:** Establish Cargo workspace, CI, domain model skeleton, and application command API.

**Deliverables:**
- Cargo workspace with all crates defined
- Rust toolchain configuration
- GitHub Actions CI for Linux/Windows/macOS
- Domain model: `DocumentId`, `ObjectId`, `LayerId`, `BrushId`, `Color`
- Application command trait and base command types
- Error model with typed errors
- Platform abstraction traits (filesystem, clipboard, secure storage, input)
- Basic headless document create/save/load test
- CLI/MCP/AI tool interface definitions (stable signatures)

**Crates to create:**
```
crates/
  domain/           # Core entities and invariants
### Phase 3: Document & Geometry Foundation (Weeks 5-8)
**Goal:** Native document format, vector geometry, save/load, undo/redo.

**Deliverables:**
- Native `.ceng` document format (ZIP container with manifest + JSON)
- Path geometry: Bézier curves, shapes, transforms
- Stroke model with pressure/tilt/velocity data retention
- Spatial index (R-tree) for hit testing and culling
- Atomic save with temporary file + validation
- Autosave journal with crash recovery
- Undo/redo with transaction grouping
- Document migration framework (schema versioning)

**Crates to enhance:**
```
crates/
  geometry/         # Full path/transform implementation
  documents/        # Native file model and migrations
  history/          # Undo/redo and transaction journal
  storage/          # Save/load/autosave/recovery
  selection/        # Hit testing (basic)
```

**Acceptance Criteria:**
- Save/load round-trip preserves all document state
### Phase 4: Drawing & Editing (Weeks 8-12)
**Goal:** Brush engine, input pipeline, selection, transforms, layers.

**Deliverables:**
- Input normalization: pressure, tilt, azimuth, tool type
- Brush preset system (declarative, versioned)
- Stroke generation pipeline: capture → normalize → smooth → resample → evaluate curves → generate outline
- Initial brush families: technical pen, fixed-width, pressure pen, pencil, marker, airbrush, eraser
- Selection: lasso, rectangle, click, multi-select
- Transforms: move, rotate, scale, flip, skew with constraints
- Layers: create, rename, reorder, visibility, lock, opacity, groups
- Object inspector with numeric controls
- Brush presets UI, color picker, tool wheel

**Crates to create/enhance:**
```
crates/
  brushes/          # Brush models and stroke engine
  input/            # Full stylus pipeline
  selection/        # Complete selection tools
  layers/           # Layer tree and compositing
```

**Acceptance Criteria:**
- Pressure-sensitive drawing with < 50ms latency
- Stroke preview matches final stroke
- Selection hit-testing accurate at all zoom levels
- Transform handles work on nested groups
- Layer operations persist through save/load
- Brush preset import/export

### Phase 5: Precision & Visual Organization (Weeks 12-16)
**Goal:** Grids, snapping, guides, measurements, artboards, object library.

**Deliverables:**
- Grid types: dot, square, graph, lined, isometric, triangular
- Perspective guides: 1/2/3-point
- Snap system: grid, point, path, tangent, intersection, guide, bounds
- Rulers and guide lines
- Measurement annotations with calibrated scale
- Unit systems: metric/imperial with custom scale
- Shape guides: line, angle, arc, circle, rectangle, ellipse
- Shape recognition from hand-drawn input
- Artboards/pages on infinite canvas
- Object library: built-in symbols, user-created objects, collections, search
### Phase 7: CLI Implementation (Weeks 20-22)
**Goal:** Complete CLI with all command groups.

**Deliverables:**
- `creative` binary with all command groups
- Human-readable + JSON output modes
- Stable exit codes
- Document inspection, validation, conversion
- Object/layer listing and controlled editing
- Batch export and automation scripts
- Config management (platform-standard dirs)
- Secure credential handling (no CLI args for secrets)

**Crates:** `apps/cli/` + `crates/cli-api/`

**Acceptance Criteria:**
- CLI validates/exports without GUI
- JSON output conforms to versioned schema
- Dry-run makes no persistent changes
- Failed batch item doesn't corrupt others

### Phase 8: MCP Server (Weeks 22-24)
**Goal:** Local stdio MCP server exposing application capabilities.

**Deliverables:**
- MCP server binary (`creative mcp serve`)
- Read tools: document_list, document_inspect, canvas_get_info, selection_get, layer_list, object_list, object_inspect, asset_search, document_validate
- Write tools: document_create, object_create/update/transform/delete, layer_create/update/reorder, group_create, precision_set, mindmap_create, diagram_create, document_export
- Task tools: brainstorm, agent_plan, agent_execute, task_status, task_cancel
- Schema validation for all tools
- Permission gating (read / reversible write / destructive write / external)
- Local-only transport by default
- Audit logging

**Crates:** `apps/mcp/` + `crates/mcp-api/`

**Acceptance Criteria:**
- MCP client can inspect without write permission
- Write calls rejected when permission absent
- Invalid schemas/oversized payloads safely rejected
- Mutations undoable and audited
- No network bind unless explicitly configured

### Phase 9: AI Providers & Agent (Weeks 24-30)
**Goal:** OpenAI/Anthropic adapters, agent orchestration, tool execution.

**Deliverables:**
- Provider-neutral interface with OpenAI + Anthropic adapters
- Secure API key storage (OS keychain/credential manager)
- Streaming responses, cancellation, model config
- Context engine: document-aware, user-controlled
- Tool registry: inspect, create, update, delete, group, arrange, style, layers, export
- Agent modes: Suggest / Confirm Each / Approved Task / Read-Only
- Plan preview with diff before execution
- Undoable AI transactions with audit trail
- Conversation history (local, clearable)
- Memory system (opt-in, project-scoped)

**Crates to create:**
```
crates/
  agent/            # Agent orchestration
  ai-providers/     # OpenAI/Anthropic adapters
  ai-tools/         # Typed tool schemas and executors
  brainstorming/    # Ideation and visual structures
  context/          # Document context extraction
```

**Acceptance Criteria:**
- Both providers pass conformance tests
---

## Critical Technical Spikes (Start Immediately)

These must be prototyped **before** committing to full implementation:

| Spike | Description | Timeline | Success Criteria |
|-------|-------------|----------|------------------|
| 1. Iced + wgpu | Smooth canvas in Iced on Linux | Phase 1-2 | 60 FPS, no flicker, HiDPI works |
| 2. Android FFI | Rust core + Vulkan + stylus on Android | Phase 1-2 | Draw with pressure, save document |
| 3. iOS FFI | Rust core + Metal on iOS | Phase 1-2 | Draw with Pencil, save document |
| 4. Geometry lib | Evaluate lyon/kurbo/parley for paths/text | Phase 1 | Boolean ops, text shaping, perf |
| 5. File container | Benchmark ZIP vs custom for `.ceng` | Phase 1 | Save/load < 100ms for 10MB doc |
| 6. PDF/SVG import | Fidelity prototype with real files | Phase 2 | Import Concepts test files |
| 7. Brush latency | Stroke quality + latency measurement | Phase 2 | < 50ms end-to-end |
| 8. MCP impl | Review MCP SDK options (rust-mcp, etc.) | Phase 2 | Protocol compliance |
| 9. AI adapters | OpenAI/Anthropic streaming + tools | Phase 2 | Conformance tests pass |
| 10. Secure storage | OS keychain on Linux/Win/macOS/Android/iOS | Phase 2 | Keys not in memory/disk plaintext |

---

## Dependency & Licensing Inventory

Must maintain `DEPENDENCY_AND_LICENSING_REGISTER.md` from Phase 0:

| Category | Crates to Evaluate |
|----------|-------------------|
| Geometry | lyon, kurbo, parley, tiny-skia, piet |
| Rendering | wgpu, naga, bytemuck, raw-window-handle |
| UI | iced, iced_wgpu, iced_runtime |
| Serialization | serde, serde_json, bincode, rmp-serde |
| Spatial | rstar, kdtree |
| Async | tokio, async-std, futures |
| CLI | clap, indicatif, console |
| MCP | rmcp, mcp-sdk (evaluate) |
| AI | openai-api-rs, anthropic-api-rs (evaluate) |
| Crypto | ring, rustls, keyring |
| Image | image, webp, resvg, pdfium (evaluate) |
| Testing | proptest, criterion, insta |
| Mobile | uniffi, jni, objc2 |

**Action:** Evaluate each for license compatibility (MIT/Apache-2/BSD preferred), maintenance status, platform support, and API stability before Phase 1 lock-in.

---

## Development Environment Setup

### Immediate (Week 1)
1. **Git repository** - Already exists at `/storage/Documents/projects/software/concept/`
2. **Rust toolchain** - Create `rust-toolchain.toml` with stable version
3. **Cargo workspace** - Create root `Cargo.toml` with all crate definitions
4. **CI/CD** - GitHub Actions for Linux/Windows/macOS
5. **Formatting/Linting** - rustfmt, clippy, deny.toml for license checking
6. **Pre-commit hooks** - cargo fmt, cargo clippy, cargo test
7. **Documentation** - mdBook or rustdoc for architecture decision records (ADRs)

### Repository Structure to Create
```
creative-engine/
├── Cargo.toml                    # Workspace root
├── rust-toolchain.toml           # Pinned toolchain
├── deny.toml                     # License/CVE checking
├── .github/
│   └── workflows/
│       ├── ci.yml                # Linux/Win/macOS build + test
│       └── release.yml           # Release automation
├── apps/
│   ├── desktop/                  # Iced desktop app
│   ├── cli/                      # CLI binary
│   ├── mcp/                      # MCP server
---

## Testing Strategy

Following `TESTING_AND_ACCEPTANCE_CRITERIA.md`:

1. **Unit tests** - Geometry, transforms, brush curves, document invariants (in each crate)
2. **Property tests** - Serialization round-trips, transform inverses, spatial queries (proptest)
3. **Integration tests** - Command API, storage, history, import/export
4. **Golden tests** - Renderer output with approved pixel tolerances (insta)
5. **Platform tests** - Input, file access, clipboard, GPU lifecycle
6. **E2E tests** - Create/draw/save/reopen/edit/export workflows
7. **Security tests** - Malformed documents, archives, SVG, MCP schemas, secrets
8. **Performance tests** - Large docs, high sample-rate strokes, large images (criterion)
9. **Provider conformance** - OpenAI/Anthropic adapters with mocks + opt-in live tests
10. **Accessibility tests** - Keyboard nav, labels, scaling, contrast

**Test Fixtures:** Curated real-world documents in `tests/fixtures/` for regression testing.

---

## Risk Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Iced mobile not production-ready | High | High | Native mobile shells from start; don't depend on Iced mobile |
| wgpu mobile backend gaps | Medium | High | Early Android/iOS GPU spikes; software fallback |
| Geometry library limitations | Medium | High | Evaluate multiple; keep abstraction layer |
| AI provider API changes | Medium | Medium | Provider-neutral interface; versioned tool schemas |
| Large document performance | Medium | High | Spatial index, viewport culling, render caching from Phase 2 |
| Document format migration | Low | High | Versioned schema, migration framework from Phase 3 |
| Security vulnerabilities in imports | Medium | High | Fuzzing, sanitization, no execution from documents |
| Scope creep | High | Medium | Strict phase gates; defer non-baseline features |

---

## Next Steps (Act Mode - Starting Now)

### Immediate (This Session)
1. ✅ Create Cargo workspace structure
2. ✅ Set up CI/CD pipeline
3. ✅ Implement domain model skeleton
4. ✅ Begin Iced + wgpu integration spike
5. ✅ Begin Android/iOS FFI spikes in parallel

### Upon Completion of Foundation
1. Begin Phase 2: Desktop shell and canvas
2. Parallel: Mobile FFI prototypes
3. Parallel: Geometry library evaluation

---

## Questions for Owner Review

1. **Product name & extension:** Confirm final name (currently "Creative Engine"/`.ceng`)
2. **Mobile priority:** Android-first or iOS-first for Phase 11?
3. **Cloud sync:** In 1.0 scope or deferred? (Currently deferred per spec)
4. **Collaboration:** Real-time multi-user in scope? (Currently non-goal)
5. **Plugin marketplace:** Local-only initially or marketplace in 1.0?
6. **PSD/DXF import:** Investigate libraries now or gate behind conditional?
7. **Watercolor brushes:** Include in 1.0 or post-1.0? (Spec says post-1.0 if quality targets not met)
8. **Web/WASM target:** Keep as explicit non-goal for 1.0?
9. **Reference hardware:** Define minimum specs for performance targets
10. **Release timeline:** Target date for 1.0?

---

## Appendix: Specification Document Quick Reference

| Doc | Purpose | Key Decisions Needed |
|-----|---------|---------------------|
| PRODUCT_REQUIREMENTS.md | What we're building | Tiers, release gates, success measures |
| FEATURE_INVENTORY.md | Feature scope matrix | Priority classification, deferred items |
| UI_UX_SPECIFICATION.md | Interface design | Layout, tool wheel, accessibility |
| RUST_WORKSPACE_ARCHITECTURE.md | Crate structure | Dependency rules, technology choices |
| CANVAS_ENGINE_SPECIFICATION.md | Canvas & camera | Coordinate precision, gesture arbitration |
| VECTOR_GEOMETRY_SPECIFICATION.md | Geometry primitives | Path model, boolean ops, text |
| RENDERING_ENGINE_SPECIFICATION.md | GPU/software rendering | Backend selection, caching, color mgmt |
| BRUSH_ENGINE_SPECIFICATION.md | Brush system | Families, pressure curves, erasing |
| DOCUMENT_FORMAT_SPECIFICATION.md | `.ceng` format | Container, migration, security |
| CROSS_PLATFORM_ARCHITECTURE.md | Platform strategy | Mobile FFI, graphics, storage |
| CLI_SPECIFICATION.md | Command-line interface | Commands, output modes, safety |
| MCP_SPECIFICATION.md | MCP server | Tools, resources, security |
| AI_AGENT_SPECIFICATION.md | Built-in agent | Providers, modes, safety boundaries |
| BRAINSTORMING_ENGINE_SPECIFICATION.md | Ideation tools | Layout algorithms, persistence |
| IMPORT_EXPORT_SPECIFICATION.md | Format support | Import/export matrix, PDF/SVG behavior |
| TESTING_AND_ACCEPTANCE_CRITERIA.md | Quality gates | Test layers, reference envs, release gate |
| DEVELOPMENT_ROADMAP.md | Phase sequence | Critical spikes, parallel work |

---

## Appendix: Current Project State

**Existing files:**
- `/storage/Documents/projects/software/concept/research/` - Research documents (first.md, fist_conclusion.md, secound.md)
- `/storage/Documents/projects/software/concept/research/creative_engine_phase0_specifications_v0.1/creative_engine_phase0_specs/` - 17 specification documents
- `/storage/Documents/projects/software/concept/.git` - Git repository initialized
- `/storage/Documents/projects/software/concept/main/` - Empty directory with .git submodule

**Ready to create:**
- Root Cargo workspace
- All crate directories and Cargo.toml files
- CI/CD configuration
- Development tooling

---

**End of Implementation Plan**

*This plan is a living document. It will be updated as specifications are reviewed, spikes produce results, and implementation progresses.*
│   ├── android/                  # Android host (later)
│   └── ios/                      # iOS host (later)
├── crates/                       # All library crates (see architecture)
├── resources/
│   ├── icons/                    # Lucide SVG icons
│   ├── fonts/                    # UI fonts (Inter, JetBrains Mono)
│   ├── brush-presets/            # Default brush definitions
│   └── sample-documents/         # Test fixtures
├── tests/
│   ├── fixtures/                 # Test documents
│   ├── cross-platform/           # Platform-specific tests
│   └── performance/              # Benchmarks
└── docs/
    └── adr/                      # Architecture Decision Records
```
- Suggest mode never mutates documents
- Confirm mode prompts before every write
- Approved task cannot exceed plan scope
- Every mutation appears in history and is undoable
- Secrets absent from logs and transcripts

### Phase 10: Brainstorming Engine (Weeks 30-34)
**Goal:** Mind maps, concept maps, diagrams, layout engine.

**Deliverables:**
- Mind map generation from topic
- Flowchart/concept map layout algorithms
- Structured diagram creation (hierarchy, networks, sequences)
- Layout engine: force-directed, hierarchical, grid
- Partial acceptance: user can edit AI-generated structures
- Agent-to-canvas workflow integration
- Brainstorming session persistence

**Crates:** `crates/brainstorming/`

**Acceptance Criteria:**
- Agent creates editable mind map from prompt
- Layout algorithms produce readable diagrams
- User can modify AI-generated structures
- Brainstorming sessions save/restore

### Phase 11: Mobile Applications (Weeks 34-44)
**Goal:** Android and iOS apps with shared Rust core.

**Deliverables:**
- Android: native host (Kotlin), JNI/UniFFI bridge, Vulkan surface, stylus/touch input, file picker, document provider API
- iOS: native host (Swift), FFI bridge, Metal surface, Pencil input, document picker
- Adaptive UI: floating tool rail, contextual sheets, two-finger nav
- Cross-platform document parity
- Mobile storage (scoped access, content URIs)
- Background/foreground lifecycle handling

**Crates:** `apps/android/`, `apps/ios/` + `crates/platform/` mobile impl

**Critical Early Spikes (must start in Phase 1-2):**
1. Android Rust core + GPU surface + stylus input
2. iOS Rust core + Metal surface
3. FFI call/callback and cancellation behaviour
4. File picker and atomic save on mobile

**Acceptance Criteria:**
- Same document opens/edits on desktop and mobile
- Stylus pressure works on supported devices
- Touch navigation fluid
- Documents sync via filesystem/cloud (user-managed)

### Phase 12: Hardening & Release (Weeks 44-52)
**Goal:** Production readiness.

**Deliverables:**
- Performance optimization (profiling, memory, GPU)
- Accessibility: keyboard nav, screen reader, high contrast, scaling
- Security review: fuzzing, path traversal, SVG sanitization, MCP/plugin validation
- Crash recovery testing
- Installers: Linux (AppImage/Flatpak/deb), Windows (MSIX/EXE), macOS (DMG/notarized), Android (AAB), iOS (TestFlight/App Store)
- Signing and reproducible builds
- User documentation
- Support workflow
- Versioned release with migration docs

**Release Gates:**
- Alpha: core document, canvas, drawing, save/load, undo
- Beta: precision, layers, objects, import/export, CLI, MCP, AI
- RC: mobile, performance, accessibility, migration, security, packaging
- 1.0: all approved requirements pass acceptance
- Text objects with font, size, alignment

**Crates to create/enhance:**
```
crates/
  precision/        # Grids, snapping, guides, measurements
  objects/          # Object library and asset metadata
  geometry/         # Boolean ops, shape recognition
```

**Acceptance Criteria:**
- All grid types render and snap correctly
- Measurements match calibrated scale
- Shape recognition converts hand drawing to clean geometry
- Artboards export correctly
- Object library supports drag-drop placement

### Phase 6: Import/Export (Weeks 16-20)
**Goal:** PNG, JPEG, WebP, SVG, PDF, DXF import/export.

**Deliverables:**
- Raster import: PNG, JPEG, WebP → image objects
- SVG import: sanitized, paths→vectors, text fallback, gradients
- PDF import: multi-page, page selection, vector/raster strategy
- Native `.ceng` full round-trip
- Export: PNG, JPEG, WebP, SVG, PDF with options (DPI, region, layers, background)
- DXF export: supported geometry subset with documented limitations
- Clipboard: internal payload + SVG/PNG fallbacks
- Batch export from CLI

**Crates to create:**
```
crates/
  importers/        # Format-specific import logic
  exporters/        # Format-specific export logic
```

**Acceptance Criteria:**
- SVG exports open in Inkscape/browser for supported features
- PDF page count and geometry match settings
- Round-trip native preserves editable content
- Import reports unsupported features, doesn't silently discard
- Batch CLI export works headlessly
- Kill process during save → recover valid document
- Load v0.1 fixture → migrate to current schema
- Undo/redo works for: strokes, transforms, layer ops, imports
- 100,000 objects navigable with viewport culling
  application/      # Use cases and command/query API
  commands/         # Validated command definitions
  platform/         # Platform abstraction traits
  storage/          # Atomic save, autosave, recovery
  cli-api/          # CLI-facing stable operations
  mcp-api/          # MCP schemas and protocol mapping
  plugin-api/       # Versioned extension interfaces
```

**Acceptance Criteria:**
- `cargo test` passes on Linux CI
- Headless test: create document → add object → save → load → verify
- All crate boundaries respect dependency direction
- No Iced/wgpu/AI dependencies in `domain` or `application`

### Phase 2: Desktop Shell & Canvas Proof (Weeks 3-5)
**Goal:** Iced desktop application with wgpu integration and basic canvas navigation.

**Deliverables:**
- Iced application shell with window management
- wgpu renderer initialization integrated with Iced
- Camera system: world↔screen transforms, pan, zoom, rotate
- Pointer input normalization (mouse, touch, stylus)
- Render simple primitives: lines, rectangles, circles
- Viewport culling with spatial index (rstar)
- Linux packaging development path (AppImage/Flatpak)
- 60 FPS navigation on reference hardware

**Crates to create:**
```
crates/
  canvas/           # Camera, viewport, navigation
  renderer/         # Render graph and backend abstraction
  renderer-wgpu/    # GPU implementation
  input/            # Normalized pointer/stylus events
  geometry/         # Paths, curves, transforms (basic)
```

**Critical Technical Spike:** Iced + wgpu integration on Linux
- Must prove smooth canvas rendering inside Iced before committing
- Test with high-DPI displays and fractional scaling

**Acceptance Criteria:**
- Desktop app launches, shows canvas
- Pan/zoom/rotate with mouse and touch
- 60 FPS with 10,000 simple objects
- No memory leaks during extended navigation