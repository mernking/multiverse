# 16. Testing Strategy and Acceptance Criteria

**Status:** Draft 0.1

## 16.1 Testing layers

1. Unit tests: geometry, transforms, brush curves, document invariants.
2. Property tests: serialization, transforms, spatial queries, numerical edge cases.
3. Integration tests: command API, storage, history, import/export.
4. Golden tests: renderer output with approved tolerances.
5. Platform tests: input, file access, clipboard, GPU lifecycle.
6. End-to-end tests: create/draw/save/reopen/edit/export.
7. Security tests: malformed documents, archives, SVG, MCP schemas, plugins, secrets.
8. Performance tests: large documents, high sample-rate strokes, large images.
9. Provider conformance tests: OpenAI and Anthropic adapters using mocks and opt-in live tests.
10. Accessibility tests: keyboard navigation, labels, scaling, contrast.

## 16.2 Reference environments

Define and maintain at least:
- minimum supported Linux desktop;
- recommended Linux desktop with stylus/display scaling;
- Windows reference machine;
- macOS reference machine;
- Android tablet/phone with stylus if supported;
- iPad reference device if iOS is included in release.

Record CPU, GPU, RAM, OS, display scale, and input device for performance results.

## 16.3 Core acceptance scenarios

### Document integrity
- Create, save, close, reopen, and verify object/layer state.
- Kill process during save and verify last valid file/recovery.
- Load older schema fixture and migrate.
- Open corrupt asset and retain other document content.

### Canvas
- Draw at multiple zoom levels and rotations.
- Pan/zoom while preserving geometry.
- Select and transform nested groups.
- Test extreme coordinates and dense object sets.

### Input
- Mouse, touch, stylus, pressure, tilt, eraser.
- Missing pressure fallback.
- Palm rejection and interrupted gesture.
- Orientation/surface recreation on mobile.

### History
- Undo/redo strokes, transforms, layer operations, imports, and AI transactions.
- Verify no unrelated later user edit is removed by undoing a prior AI transaction.

### Import/export
- Test representative real-world fixtures and malformed files.
- Compare rendered output and geometry.
- Verify overwrite and batch error handling.

### CLI/MCP
- Run headless validation/export.
- Verify JSON schema stability.
- Deny unauthorized writes.
- Reject malformed and oversized requests.
- Verify local MCP server does not bind network by default.

### AI
- Test both providers with mock responses.
- Test streaming, cancellation, rate limits, malformed tool calls.
- Verify plan scope and permissions.
- Verify secret redaction.
- Verify document content is only sent with user-authorized provider requests.

## 16.4 Performance acceptance targets

Targets to be measured, not assumed:
- 60 FPS for ordinary navigation on reference hardware.
- Drawing feedback under 50 ms under normal load.
- No unbounded memory growth during repeated import/export.
- Autosave does not visibly block drawing.
- Large document stress suite records frame-time percentiles and memory.

## 16.5 Security acceptance

- Fuzz document/archive parsers.
- Reject path traversal and zip bombs.
- Sanitize SVG.
- Validate all MCP and plugin inputs.
- No secret values in logs.
- AI cannot execute shell commands or access arbitrary paths.
- Permission checks occur in application layer, not only UI.

## 16.6 Release gate

A release candidate must:
- pass all critical tests on each claimed platform;
- have no known data-loss defects;
- have documented limitations;
- pass security review for import, MCP, plugins, and AI;
- include migration and recovery documentation;
- have reproducible build/package instructions.
