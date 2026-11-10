# 1. Product Requirements Specification

**Status:** Draft 0.1

## 1.1 Product vision

Build a professional, cross-platform, infinite-canvas creative workspace for freehand drawing, technical sketching, visual planning, diagrams, and spatial brainstorming. The product combines a high-quality drawing engine with precise vector editing, reusable objects, a scriptable CLI, MCP integration, and an AI agent that can reason about and operate on the user's canvas under explicit permissions.

The product must be useful without an AI provider, account, or network connection. AI is an integrated capability, not a dependency of the drawing application.

## 1.2 Target users and jobs

| User group | Primary jobs |
|---|---|
| Sketchers and illustrators | Draw naturally with pen, touch, mouse, and stylus |
| Designers and product thinkers | Explore concepts, annotate references, organize visual ideas |
| Engineers and technical users | Create precise geometry, diagrams, measurements, and perspective sketches |
| Students and educators | Explain concepts visually, annotate documents, build mind maps |
| Automation-oriented users | Inspect, transform, batch-export, and generate documents through CLI |
| AI/workflow builders | Connect the application through MCP and controlled agent tools |

## 1.3 Product goals

- Provide a responsive infinite canvas with fluid navigation at high zoom levels.
- Support pressure-aware stylus input and a configurable brush system.
- Provide precise vector geometry, selection, transforms, layers, snapping, grids, and measurements.
- Preserve editable document structure and provide reliable save/recovery.
- Import and export common visual and technical formats.
- Offer equivalent core document operations from GUI, CLI, MCP, and AI tools.
- Support Linux, Windows, macOS, Android, and iOS through a shared Rust core.
- Make AI brainstorming and canvas execution useful, transparent, reversible, and permission-controlled.

## 1.4 Non-goals for the initial production baseline

Unless explicitly approved later:
- Real-time multi-user collaboration.
- Hosted cloud storage or mandatory user accounts.
- A full CAD constraint solver or parametric 3D CAD system.
- Full Photoshop/Illustrator feature parity.
- A general-purpose code IDE.
- Training or hosting foundation models.
- Unrestricted autonomous file-system or network access by the agent.
- Guaranteed import fidelity for every proprietary format/version.

## 1.5 Functional requirements

### Canvas and editing
- Infinite or practically unbounded 2D world coordinates.
- Pan, zoom, rotate, fit, selection zoom, and viewport reset.
- Vector paths, shapes, text, images, groups, and supported raster layers.
- Selection, multi-selection, move, resize, rotate, duplicate, delete, align, distribute, group, lock, hide, and reorder.
- Layers with visibility, lock, opacity, naming, and ordering.
- Undo/redo with transaction grouping.
- Autosave, crash recovery, and explicit save/export.

### Drawing
- Mouse, touch, stylus, pressure, tilt, and eraser support where hardware/OS provides data.
- Brush presets, size, opacity, smoothing, color, spacing, and pressure curves.
- Stroke editing and selection.
- Independent raster and vector drawing modes where appropriate.

### Precision and technical work
- Grid types, snapping, guides, shape recognition, measurement annotations, scale calibration, and perspective guides.
- Unit systems: metric and imperial.
- Geometry must remain stable under zoom, rotation, and document transforms.

### Documents and assets
- Native editable document format with versioning.
- Page/artboard support.
- Image, PDF, SVG, and other import paths as detailed in the import/export specification.
- Export to common image, vector, and document formats.
- Asset/object library with reusable symbols and user-created collections.

### CLI and MCP
- Headless document inspection, validation, conversion, export, and batch operations.
- Structured command results and stable exit/error codes.
- MCP tools for safe document and canvas operations.
- Explicit consent and permissions for actions that alter files or documents.

### AI agent
- OpenAI and Anthropic API adapters behind a provider-neutral interface.
- Streaming responses, cancellation, model configuration, and usage visibility.
- Document-aware context retrieval.
- Brainstorming, critique, summarization, mind-map generation, diagram creation, and canvas editing.
- Preview plans and diffs before execution; undoable execution.
- No secret exposure in prompts, logs, or tool outputs.

## 1.6 Non-functional requirements

| Area | Requirement |
|---|---|
| Performance | Interactive navigation and drawing should remain responsive on supported reference hardware |
| Reliability | No acknowledged save may be lost under normal process termination |
| Recovery | Recover last autosaved state after crash, with clear recovery UI |
| Portability | Document files must not depend on platform-specific absolute paths |
| Security | Validate imports, MCP inputs, plugin code, and AI tool arguments |
| Accessibility | Keyboard navigation, scalable UI, contrast-aware themes, accessible labels |
| Privacy | No telemetry or cloud transmission without clear disclosure and controls |
| Maintainability | Modular Rust crates, explicit interfaces, documented APIs |
| Testability | Core logic testable without GUI or GPU |
| Compatibility | Versioned file and protocol schemas with migration strategy |

## 1.7 Product tiers and release definition

The first release is a full local creative application, not a demo. “Full” means all features explicitly approved for the release baseline are implemented and tested; it does not mean reproducing every feature of every competing product.

Suggested release gates:
- **Alpha:** core document model, canvas, drawing, save/load, undo.
- **Beta:** precision, layers, object library, import/export, CLI, MCP, AI.
- **Release candidate:** mobile integration, performance, accessibility, migration, security, packaging.
- **1.0:** all approved release requirements pass acceptance.

## 1.8 Success measures

- A user can create, save, close, reopen, edit, and export a complex document without data loss.
- Typical drawing and navigation remain responsive on the minimum supported hardware.
- A document opened on one supported platform remains editable on another.
- GUI, CLI, MCP, and agent operations produce consistent document results.
- AI-generated edits are reviewable and undoable.
- Import/export limitations are documented and surfaced rather than silently corrupting content.

## 1.9 Decisions to confirm

1. Final product name and file extension.
2. Whether cloud sync is desired in 1.0 or later.
3. Whether collaboration is in scope.
4. Whether first release includes iOS or Android-first mobile.
5. Whether a plugin marketplace is desired or only local plugin loading initially.
