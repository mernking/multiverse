# 15. Import and Export Specification

**Status:** Draft 0.1

## 15.1 General rules

- Importers never execute embedded content.
- Imports occur into a new document or explicit target layer/page.
- Preserve source file and report unsupported features.
- Export settings are explicit and reproducible.
- Batch operations use the same conversion pipeline as GUI operations.

## 15.2 Import matrix

| Format | Proposed support | Notes |
|---|---|---|
| PNG | 1.0 | Raster image with alpha |
| JPEG | 1.0 | Raster image |
| WebP | 1.0 | Verify decoder/encoder support |
| SVG | 1.0 | Sanitize; support paths, shapes, text fallback, gradients as feasible |
| PDF | 1.0 | Multi-page import; page selection and raster/vector strategy |
| Native `.ceng` | 1.0 | Full editable round trip |
| PSD | Conditional | Layered import only if reliable library and compatibility tests justify it; otherwise flattened import with warning |
| DXF | Later/conditional | Technical geometry interchange; define supported entity subset |
| AI/EPS | Later/conditional | Do not promise native fidelity without a tested parser |

## 15.3 Export matrix

| Format | Proposed support | Notes |
|---|---|---|
| PNG | 1.0 | DPI/scale, transparency, region selection |
| JPEG | 1.0 | Background colour and quality |
| WebP | 1.0 | Quality/lossless options if library supports |
| SVG | 1.0 | Vector-preserving export; raster fallback for unsupported effects |
| PDF | 1.0 | Page/artboard export, multipage, vector/raster content |
| Native `.ceng` | 1.0 | Editable document |
| DXF | Conditional | Map supported geometry and units; document limitations |
| PSD | Later/conditional | Layer mapping and rasterization rules required |

## 15.4 PDF behaviour

- Multi-page import.
- Select pages or import all.
- Preserve vector paths where possible; rasterize unsupported content with stated resolution.
- Pages may be placed as reference objects or imported as artboards.
- Export multipage PDF with explicit page size, margins, DPI, and background settings.

## 15.5 SVG security and fidelity

- Sanitize external references, scripts, event handlers, and unsafe constructs.
- Block network fetches by default.
- Convert unsupported filters/effects to raster with a warning.
- Preserve supported paths and fills as editable vector objects.
- Define font fallback and text-to-path option.

## 15.6 Clipboard

- Copy internal structured payload for application-to-application fidelity.
- Provide SVG and PNG fallback where supported.
- Clipboard data is untrusted on paste and must be validated.

## 15.7 Export options

- document, page/artboard, selection, layer, or custom region;
- output dimensions and DPI;
- background and transparency;
- visible layers only;
- color profile where supported;
- file naming and overwrite behaviour.

## 15.8 Acceptance

- Round-trip native export/import preserves editable content.
- PNG export has correct alpha and dimensions.
- SVG export opens in at least two independent SVG consumers for supported features.
- PDF page count and geometry match settings.
- Unsupported import features are reported, not silently discarded.
