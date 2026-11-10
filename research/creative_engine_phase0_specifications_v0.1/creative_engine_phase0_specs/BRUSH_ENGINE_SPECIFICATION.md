# 8. Brush Engine Specification

**Status:** Draft 0.1

## 8.1 Brush architecture

A brush preset is a versioned declarative definition interpreted by the stroke engine. Presets must not require executable code.

Properties:
- identifier, name, version, category;
- rendering mode (vector outline or raster texture);
- base size and opacity;
- spacing;
- smoothing/stabilization;
- pressure response curves;
- tilt/azimuth response;
- velocity response;
- rotation and scatter;
- color dynamics;
- texture/stamp references;
- blend mode where supported.

## 8.2 Input sample

Each sample may contain position, timestamp, pressure, tilt, azimuth, contact dimensions, and input device type. Missing values have explicit defaults; do not assume every device reports pressure.

## 8.3 Stroke generation

Pipeline:
1. Capture raw input.
2. Normalize device values.
3. Apply configurable smoothing/stabilization.
4. Resample based on distance/time.
5. Evaluate pressure/tilt/velocity curves.
6. Generate vector outline or raster dab sequence.
7. Store editable source samples and stroke parameters.
8. Render preview with low latency; refine after stroke completion.

## 8.4 Brush families

Initial families:
- technical pen;
- fixed-width pen;
- pressure pen;
- pencil;
- marker/highlighter;
- soft airbrush;
- textured/raster brush;
- eraser;
- dotted/stipple brush.

Watercolor and complex media simulation require a separate quality/performance evaluation and may be post-1.0 if they cannot meet quality targets.

## 8.5 Pressure curves

Use normalized input range [0,1] and a monotonic curve editor. Define behaviour for devices without pressure (default pressure value). Users can invert or adjust pressure response.

## 8.6 Erasing

Support:
- whole-object deletion;
- stroke-segment erase for vector strokes;
- raster erase for raster layers.
The selected erase mode must be visible. Segment erase must preserve path integrity and undo data.

## 8.7 Preset portability

Brush packs use a documented archive format with manifest, version, previews, and licensed assets. Reject path traversal, oversized payloads, and executable content.

## 8.8 Acceptance

- Same input sample sequence and brush preset yields deterministic stroke geometry within tolerance.
- Pressure curve behaves as configured.
- Missing pressure does not create invisible strokes.
- Stroke preview and final stroke align.
- Brush pack import validates all assets and metadata.
