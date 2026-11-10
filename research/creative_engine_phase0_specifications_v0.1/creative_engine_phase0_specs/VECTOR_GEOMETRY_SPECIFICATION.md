# 6. Vector Geometry Specification

**Status:** Draft 0.1

## 6.1 Geometry primitives

Canonical geometry types:
- Point2, Vector2, Transform2D, Rect, AABB.
- Line segment and polyline.
- Cubic Bézier path segments.
- Ellipse and arc.
- Polygon and compound path.
- Text layout box.
- Image rectangle with source transform.
- Connector/arrow geometry.
- Dimension/measurement geometry.

Use a single documented coordinate convention (right-handed or screen-oriented) throughout. Define angle units and transform multiplication order centrally.

## 6.2 Path model

A path contains:
- stable path ID;
- subpaths;
- move/line/cubic segments;
- closed/open state;
- fill rule;
- stroke style;
- optional pressure profile;
- optional semantic metadata.

Avoid storing tessellated GPU triangles as the canonical document geometry.

## 6.3 Stroke model

Store raw or resampled input samples and the generated outline/centreline representation needed for faithful rendering and editing. A stroke should retain enough source data to support pressure changes, smoothing adjustments, and future regeneration where feasible.

## 6.4 Transform model

- Affine transforms are first-class.
- Group transforms compose predictably.
- Transform origin/pivot is explicit.
- Transform operations preserve object-local geometry when possible.
- Non-uniform scaling effects on stroke width are defined and tested.
- Singular or near-singular transforms are rejected or normalized safely.

## 6.5 Boolean geometry

Boolean operations must define:
- fill rule;
- tolerance;
- self-intersection handling;
- open path behaviour;
- curve flattening tolerance;
- result topology;
- failure reporting.

Do not silently approximate unsupported cases. Preserve originals until the user confirms the operation.

## 6.6 Shape recognition

Recognition pipeline:
1. Capture input samples.
2. Filter noise while retaining corners.
3. Generate candidate primitives.
4. Score fit, closure, smoothness, and user intent.
5. Present preview or auto-convert according to user preference.
6. Preserve original stroke for undo.

Initial recognized shapes: line, polyline, rectangle, square, circle, ellipse, triangle, arc, arrow.

## 6.7 Text and typography

Text object stores content, font reference, size, line height, alignment, paragraph layout, fill, and transform. Missing fonts must trigger fallback without altering stored font preference. Font licensing and embedding rules must be respected.

## 6.8 Units and precision

Canonical geometry uses a stable logical unit. Display and measurement layers map to mm, cm, m, inches, points, or user-defined scale. Unit conversion must not mutate stored coordinates unless user explicitly changes document scale.

## 6.9 Geometry correctness

Property-based tests should cover:
- transform/inverse-transform round trips;
- bounding box containment;
- path closure;
- hit-test stability;
- boolean topology;
- serialization round trips;
- numerical edge cases;
- extreme zoom and coordinate ranges.
