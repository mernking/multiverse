# 9. Native Document Format Specification

**Status:** Draft 0.1

## 9.1 Goals

- Editable and self-contained.
- Versioned and migratable.
- Portable across desktop and mobile.
- Efficient for incremental save and large assets.
- Resilient to partial corruption.
- Safe to inspect and validate without rendering.

## 9.2 Proposed container

Temporary extension: `.ceng`.

Proposed ZIP-compatible container:
```text
document.ceng
  manifest.json
  document.json
  assets/
    images/<content-hash>
    fonts/<permitted-embedded-fonts>
    thumbnails/<id>.webp
  previews/
    thumbnail.webp
  history/             # optional bounded journal/checkpoint
```
Container compression and layout are provisional; benchmark before finalizing. Avoid storing GPU-specific binaries as canonical content.

## 9.3 Manifest

Fields:
- format identifier;
- schema major/minor/patch;
- application version;
- document UUID;
- created/modified timestamps;
- asset inventory and hashes;
- feature flags;
- optional compatibility metadata.

## 9.4 Document model

- document metadata;
- world settings;
- artboards/pages;
- layer tree;
- objects with stable IDs;
- geometry and styles;
- precision settings;
- measurement calibration;
- object references to content-addressed assets;
- optional application-specific extension data.

## 9.5 Object identity and ordering

Use stable UUIDs. Z-order and layer order are explicit. References must be validated. Deleted IDs must not accidentally resolve to unrelated objects.

## 9.6 Saving and recovery

- Write to temporary file.
- Flush and validate.
- Atomically replace destination where platform permits.
- Maintain autosave journal/checkpoint.
- Keep recovery copy until successful reopen/validation.
- Never overwrite the last known good file after a failed save.
- Provide recovery choices on next launch.

## 9.7 Schema evolution

- Major version: incompatible structural change.
- Minor version: additive compatible fields.
- Patch: correction with no semantic change.
- Unknown optional fields should be retained when possible.
- Migrations must be deterministic and tested with fixtures.
- Never silently discard unsupported objects; preserve as opaque extension data or clearly report incompatibility.

## 9.8 Security

- Limit archive entry count, expanded size, nesting, and compression ratio.
- Reject path traversal and duplicate critical entries.
- Verify content hashes.
- Treat embedded SVG, fonts, and metadata as untrusted.
- Do not execute content from a document.

## 9.9 Acceptance

- Save/load round trip preserves semantic document state.
- Interrupted save leaves a recoverable valid document.
- Older fixture documents migrate correctly.
- Unknown optional extension data survives load/save.
- Corrupt assets are isolated and reported without crashing the whole application.
