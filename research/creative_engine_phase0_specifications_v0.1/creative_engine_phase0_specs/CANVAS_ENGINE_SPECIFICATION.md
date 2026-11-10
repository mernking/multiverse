# 5. Canvas Engine Specification

**Status:** Draft 0.1

## 5.1 Coordinate systems

Three distinct spaces:
1. **Document/world space:** persistent content coordinates, independent of window size and zoom.
2. **Viewport space:** logical coordinates within the application canvas region.
3. **Device space:** physical pixels after display scale factor.

The camera maps world to viewport and viewport to device. Document geometry is never rewritten merely because the user pans, zooms, rotates, or resizes the window.

## 5.2 Camera model

Camera state:
- world-space centre;
- zoom factor;
- rotation angle;
- viewport dimensions;
- device scale factor.

Required operations:
- world-to-screen and screen-to-world transforms;
- pointer-centred zoom;
- pan by screen delta;
- rotate around pointer or viewport centre;
- fit document bounds;
- fit selection;
- zoom to numeric level;
- clamp only to safe numerical limits, not arbitrary canvas boundaries.

Use f64 for world/camera calculations where precision requires it; render coordinates may be converted to f32 relative to a local origin.

## 5.3 Infinite-world strategy

- No fixed document width/height for the free canvas.
- Spatial index partitions content into queryable regions.
- Rendering queries visible objects plus a margin for strokes/effects.
- Large-coordinate precision handled with camera-relative rendering origin.
- Artboards are bounded regions within the world, not the world itself.

## 5.4 Input pipeline

Normalize OS events into:
- pointer down/move/up;
- pressure;
- tilt X/Y;
- azimuth/rotation if available;
- contact size;
- tool type (mouse, finger, stylus, eraser);
- timestamp;
- modifier keys;
- cancellation/lost capture.

Keep raw samples for high-quality stroke reconstruction. Coalesce samples only when safe and preserve timestamps.

## 5.5 Gesture arbitration

Priority rules must be explicit:
- active stylus drawing takes precedence over incidental touch when configured;
- two-finger gesture navigates by default;
- one-finger touch behaviour is user-configurable;
- palm rejection uses platform signal where available;
- interrupted gestures cancel safely and do not leave partial invalid objects.

## 5.6 Hit testing and spatial indexing

- Use a spatial index (R-tree, quadtree, or measured alternative).
- Hit tests account for screen-space tolerance, zoom, stroke width, and object transforms.
- Separate geometric hit testing from visual selection policy.
- Deterministic z-order tie-breaking.
- Support point, rectangle, lasso, and region queries.

## 5.7 Canvas command model

Every persistent change is represented as a command:
- create/delete object;
- update geometry/style;
- transform object/group;
- reorder or modify layer;
- add/remove artboard;
- update precision settings;
- add measurement/annotation.

Commands validate IDs, geometry, layer constraints, and document version. Each command returns a result and inverse/undo data or a reversible transaction record.

## 5.8 History

- Group continuous pointer movement into one logical stroke transaction.
- Group drag transforms into one transaction.
- Support undo/redo across save/load.
- AI operations execute as named transactions.
- Undo of AI work should revert only the AI transaction, not later user edits.
- History storage must have bounded memory and configurable checkpointing.

## 5.9 Performance targets

Initial targets to validate on reference hardware:
- 60 FPS navigation and drawing on ordinary documents.
- 120 FPS where display and hardware support it.
- Input-to-visible-feedback target below 50 ms under normal load.
- Viewport culling prevents work proportional to all document objects.
- A stress document with 100,000 simple vector objects remains navigable, with degradation measured and documented.

These are engineering targets, not claims of achieved performance.

## 5.10 Acceptance tests

- World coordinates remain unchanged after arbitrary pan/zoom/rotation.
- Pointer-centred zoom keeps the world point under the pointer stable.
- Very large and very small coordinate values do not cause visible jitter within supported limits.
- Hit testing remains consistent across zoom levels.
- Interrupted input does not corrupt the document.
- Undo/redo reproduces exact prior document state.
