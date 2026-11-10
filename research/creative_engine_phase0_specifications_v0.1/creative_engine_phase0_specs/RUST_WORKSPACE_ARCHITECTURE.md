# 4. Rust Workspace and System Architecture

**Status:** Draft 0.1

## 4.1 Architectural rule

All front ends call the same application command API. UI code, CLI commands, MCP tools, and AI tools must not independently mutate document internals.

## 4.2 Proposed repository

```text
creative-engine/
  Cargo.toml
  rust-toolchain.toml
  apps/
    desktop/          # Iced desktop application
    cli/              # command-line interface
    mcp/              # MCP server process
    android/          # Android host/shell and JNI/UniFFI bridge
    ios/              # iOS host/shell and FFI bridge
  crates/
    domain/           # document entities and invariants
    application/      # use cases and command/query API
    commands/         # validated command definitions
    canvas/           # camera, viewport, navigation
    geometry/         # paths, curves, booleans, transforms
    input/            # normalized pointer/stylus events
    brushes/          # brush models and stroke generation
    renderer/         # render graph and backend abstraction
    renderer-wgpu/    # GPU implementation
    renderer-software/# fallback/export renderer
    selection/        # hit testing and selection tools
    layers/           # layer tree and compositing semantics
    precision/        # grids, guides, snapping, measurements
    assets/           # object library and asset metadata
    documents/        # native file model and migrations
    history/          # undo/redo and transaction journal
    importers/
    exporters/
    storage/          # atomic save, autosave, recovery
    platform/         # filesystem, clipboard, secure storage, input
    agent/            # agent orchestration
    ai-providers/     # OpenAI/Anthropic adapters
    ai-tools/         # typed tool schemas and executors
    brainstorming/    # ideation and visual structures
    context/          # document context extraction
    cli-api/          # CLI-facing stable operations
    mcp-api/          # MCP schemas and protocol mapping
    plugin-api/       # versioned extension interfaces
    telemetry/        # opt-in diagnostics only
  resources/
    icons/
    fonts/
    brush-presets/
    sample-documents/
  tests/
    fixtures/
    cross-platform/
    performance/
  docs/
```

## 4.3 Dependency direction

```text
Desktop UI ─┐
CLI ────────┤
MCP ────────┼──> Application API ──> Domain / Commands / Services
Android ────┤                         │
iOS ────────┘                         ├──> Geometry / Canvas / History
                                      ├──> Documents / Storage
                                      ├──> Importers / Exporters
                                      └──> Agent tools (through permission gateway)

Renderer consumes immutable render snapshots; it does not own document truth.
```

Rules:
- `domain` must not depend on Iced, wgpu, AI SDKs, or platform UI.
- `application` orchestrates domain services and transactions.
- `renderer` reads render snapshots and reports device/input capabilities.
- AI and MCP invoke application commands through typed interfaces.
- Avoid cyclic crate dependencies.
- Platform-specific code is isolated behind traits and target-specific modules.

## 4.4 Core public interfaces

Illustrative contracts:
```rust
pub trait DocumentRepository {
    fn load(&self, id: DocumentId) -> Result<Document, StorageError>;
    fn save(&self, document: &Document) -> Result<SaveReceipt, StorageError>;
}

pub trait CommandHandler<C> {
    type Output;
    fn execute(&mut self, command: C) -> Result<Self::Output, AppError>;
}

pub trait AiProvider {
    fn capabilities(&self) -> ProviderCapabilities;
    async fn stream(&self, request: ModelRequest)
        -> Result<ModelStream, ProviderError>;
}
```
These are architectural examples, not frozen API signatures.

## 4.5 State ownership

- Domain model owns document content and invariants.
- Application layer owns active session and command orchestration.
- UI owns transient presentation state.
- Renderer owns GPU resources/cache, never canonical geometry.
- Storage owns durable serialization and recovery.
- Agent owns conversation/task state, never bypasses command validation.

## 4.6 Technology selection

Proposed:
- Rust stable toolchain.
- Iced for desktop UI.
- wgpu for GPU rendering where it fits target integration.
- Native Android/iOS shells with shared Rust libraries.
- serde for serialization.
- SQLite only for indexes, local metadata, history databases, or caches where beneficial; native document remains self-contained.
- tracing for structured diagnostics.
- lyon/kurbo or equivalent geometry libraries after license, maintenance, and feature evaluation.
- image, resvg, and tiny-skia or equivalent for raster/vector import/export paths.

Before locking dependencies, evaluate licenses, maintenance, platform support, API stability, and performance. Pin versions and maintain a dependency inventory.

## 4.7 Error model

Use typed errors at crate boundaries. Distinguish:
- invalid user command;
- missing object/document;
- unsupported operation;
- corrupt document;
- import/export failure;
- storage failure;
- renderer/device failure;
- provider/network failure;
- permission denied;
- version incompatibility.

Errors shown to users must be actionable and must not expose secrets or internal stack traces.

## 4.8 Concurrency

- UI thread remains responsive.
- Expensive import, export, indexing, and AI work run asynchronously.
- Document mutations are serialized through the application transaction boundary.
- Rendering consumes immutable snapshots.
- Long tasks support cancellation and progress.
- Background tasks must not hold locks across network waits.

## 4.9 API stability

Document schema, CLI output, MCP tool names, plugin API, and agent tool schemas are versioned independently. Breaking changes require migration notes and compatibility tests.
