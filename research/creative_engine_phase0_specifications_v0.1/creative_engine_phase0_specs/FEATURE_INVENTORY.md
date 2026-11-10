# 2. Feature Inventory and Scope Matrix

**Status:** Draft 0.1

This inventory defines candidate capabilities. “Release baseline” means the feature is proposed for 1.0, subject to owner approval. Platform availability may vary when hardware or operating-system APIs differ.

## 2.1 Canvas and navigation

| Feature | Description | Priority |
|---|---|---|
| Infinite canvas | Large world-coordinate plane | Baseline |
| Pan | Mouse, touch, stylus gesture, keyboard | Baseline |
| Zoom | Pointer-centred zoom, pinch, shortcuts | Baseline |
| Rotate view | Rotate viewport without altering document geometry | Baseline |
| Fit content | Fit all objects or selection to viewport | Baseline |
| Canvas backgrounds | Solid, transparent, grid-backed | Baseline |
| Artboards/pages | Named bounded work areas on an infinite canvas | Baseline |
| Navigator/minimap | Overview and viewport indicator | Later if performance constrained |

## 2.2 Drawing and brushes

- Freehand vector strokes.
- Pressure-sensitive variable width.
- Fixed-width pen, pencil, marker, technical pen, highlighter.
- Eraser by stroke, segment, or pixel where supported.
- Brush size, opacity, color, smoothing, spacing, angle, pressure curve.
- Tilt and velocity response when available.
- Brush presets, custom presets, import/export of brush definitions.
- Stroke outline conversion and editable control points.
- Raster brush mode for textured/paint-like effects.
- Fill and color sampling.
- Stroke stabilization and prediction, with raw input retained for editing.

## 2.3 Shapes and geometry

- Point, line, polyline, bezier path, arc, ellipse, rectangle, rounded rectangle, polygon.
- Arrow, callout, dimension line, connector.
- Shape recognition from hand-drawn input.
- Boolean operations: union, subtract, intersect, exclude — release only after robust geometry tests.
- Offset, simplify, smooth, trim, split, join.
- Fill rules, stroke caps/joins, dashes, gradients.
- Text objects with font family, size, alignment, line spacing, and text-on-path as later capability.

## 2.4 Selection and transforms

- Single and multiple selection.
- Lasso, rectangle, object click, layer selection.
- Move, rotate, scale, flip, skew.
- Constrain proportions and snap pivots.
- Duplicate, copy/paste, cut, delete.
- Align/distribute, group/ungroup, lock/unlock, hide/show.
- Transform handles and numeric transform entry.
- Non-destructive clipping/masking; advanced warp/perspective transforms may be deferred.

## 2.5 Layers and document organization

- Create, rename, duplicate, delete, reorder.
- Visibility, lock, opacity, blend mode.
- Groups and nested layers.
- Background, drawing, image, annotation, measurement, and reference layer roles.
- Layer-level export and isolation.
- Layer history and safe merge operations.

## 2.6 Precision tools

- Dot, square, graph, lined, isometric, triangular grids.
- Custom grid spacing, origin, rotation, color, opacity.
- One-, two-, and three-point perspective guides.
- Snap to grid, point, path, tangent, intersection, guide, and object bounds.
- Rulers and guides.
- Measurement annotations and calibrated scale.
- Metric and imperial units.
- Shape guides for line, angle, arc, circle, rectangle, ellipse.
- Constraint helpers: horizontal, vertical, equal, perpendicular, parallel.

## 2.7 Objects and asset library

- Built-in symbols and reusable objects.
- User-created objects from selected canvas elements.
- Collections, tags, favorites, search, recent items.
- Placement, scaling, rotation, recoloring, and replacement.
- Asset licensing and attribution metadata.
- External stock integration is not a 1.0 dependency.

## 2.8 Import and export

See `IMPORT_EXPORT_SPECIFICATION.md`. Candidate formats:
- Import: PNG, JPEG, WebP, SVG, PDF, native format; PSD support subject to test and documented flattening/feature limitations.
- Export: PNG, JPEG, WebP, SVG, PDF, native; DXF as a technical vector export subject to geometry mapping.
- Clipboard: native object payload plus PNG/SVG fallbacks.
- Batch and region export from CLI.

## 2.9 UI and productivity

- Tool bar and configurable tool wheel.
- Layers, precision, object library, color, brush, and inspector panels.
- Keyboard shortcuts and command search.
- Context menus, status bar, zoom/rotation readout.
- Themes, panel layout persistence, workspace presets.
- Presentation mode and clean canvas mode.
- Onboarding and searchable help.

## 2.10 CLI and MCP

- Project/document listing, inspection, validation, conversion.
- Object/layer listing and controlled editing.
- Batch exports and automation scripts.
- MCP resource exposure for document metadata and selected context.
- MCP tools for read and write operations with schema validation.
- Local-only default transport; remote transport requires authentication and explicit configuration.

## 2.11 AI capabilities

- Ask questions about the current document.
- Brainstorm from a topic or selected content.
- Generate mind maps, flowcharts, concept maps, diagrams, and structured layouts.
- Summarize, label, critique, and reorganize content.
- Create and edit canvas objects from natural language.
- Generate alternative design directions on duplicate pages/layers.
- Explain a selected region or object.
- AI plan preview, approval, execution, undo, and audit history.

## 2.12 Platform support matrix

| Capability | Linux | Windows | macOS | Android | iOS |
|---|---:|---:|---:|---:|---:|
| Core document model | Yes | Yes | Yes | Yes | Yes |
| Canvas drawing | Yes | Yes | Yes | Yes | Yes |
| Mouse/keyboard | Yes | Yes | Yes | Optional | Optional |
| Stylus pressure | Device/driver dependent | Device dependent | Device dependent | API/device dependent | API/device dependent |
| CLI | Yes | Yes | Yes | Not initially | Not initially |
| MCP host | Yes | Yes | Yes | Optional later | Optional later |
| Built-in AI | Yes | Yes | Yes | Yes | Yes |
| GPU rendering | Backend dependent | Backend dependent | Metal/Vulkan path | Vulkan/GLES path | Metal path |

## 2.13 Explicitly deferred candidates

Cloud sync, collaboration, marketplace, 3D workspace, parametric CAD, animation timeline, video editing, and hosted model inference require separate approval and specifications.
