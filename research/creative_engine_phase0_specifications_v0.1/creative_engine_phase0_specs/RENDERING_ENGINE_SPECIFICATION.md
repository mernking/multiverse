# 7. Rendering Engine Specification

**Status:** Draft 0.1

## 7.1 Responsibilities

Renderer converts immutable document/render snapshots into pixels. It does not own canonical document state, command history, or serialization.

## 7.2 Backend architecture

- Renderer facade defines capabilities and frame submission.
- Primary GPU backend proposed with wgpu.
- Software renderer supports export, fallback, tests, and environments without a suitable GPU.
- Backend selection is capability-based and reported to the user when degraded.
- Avoid platform-specific assumptions in the render graph.

## 7.3 Render pipeline

1. Determine visible world region.
2. Query spatial index.
3. Build ordered render items by layer and z-order.
4. Resolve style, clipping, opacity, and blend state.
5. Tessellate/cache vector geometry.
6. Render images, paths, text, effects, grids, guides, selection overlays.
7. Composite layers and output to viewport.
8. Reuse cached resources until source revision changes.

## 7.4 Render layers

Separate document content from editor overlays:
- document objects;
- page/artboard backgrounds;
- grids and guides;
- selection outlines and handles;
- cursor and stylus feedback;
- transient previews;
- AI proposed changes.

Editor overlays should not be accidentally included in document exports unless explicitly requested.

## 7.5 Caching

Cache keys include object ID, geometry revision, style revision, zoom bucket where relevant, and renderer/backend generation. Invalidate precisely on edits. GPU resources must be recreated after device loss.

## 7.6 Color management

- Define document color space and working assumptions.
- Support sRGB baseline.
- Plan explicit profile handling for imported/exported images.
- Alpha compositing and blend modes must be consistent across GPU and software renderers.
- Do not claim wide-gamut/HDR support until tested end-to-end.

## 7.7 Text and images

- Text shaping/layout must be stable across platforms as far as practical.
- Image decode is asynchronous and bounded.
- Large images use downsampled previews/tiles.
- Missing or corrupt image assets show clear placeholders and diagnostics.

## 7.8 Export rendering

Export renderer must be deterministic for the same document, export settings, and software version within documented tolerances. Support transparent background, selected layers, artboard, selection, or custom region.

## 7.9 Device loss and fallback

On GPU failure:
- preserve document state;
- notify user;
- attempt backend reinitialization;
- fall back to software rendering if feasible;
- log diagnostic details without private document contents.

## 7.10 Acceptance

- GPU and software output match within defined pixel tolerance for supported primitives.
- Offscreen objects are culled.
- Device loss does not corrupt document state.
- Selection overlays never leak into normal export by default.
- Large images do not block input during decode.
