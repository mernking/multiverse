# 3. UI/UX Specification

**Status:** Draft 0.1

## 3.1 Design principles

- Canvas-first: the drawing area receives the largest usable region.
- Tools are discoverable without permanently consuming canvas space.
- Common operations require few gestures or clicks.
- Precision controls remain accessible but do not overwhelm freehand workflows.
- Desktop and mobile share concepts and terminology while adapting interaction patterns.
- UI state must never be the sole source of document truth.

## 3.2 Desktop layout

Recommended default layout:
- **Top/command area:** document name, undo/redo, tool mode, search/command palette, export/share.
- **Left tool rail:** select, pen, pencil, marker, eraser, shape, text, image, hand/pan.
- **Centre:** infinite canvas viewport.
- **Right inspector:** contextual properties, layers, precision, object library, AI panel.
- **Bottom status area:** zoom, rotation, active tool, input state, measurement scale.
- Panels can collapse, dock, resize, and restore to defaults.

The layout is a proposal, not a pixel-perfect clone of another product.

## 3.3 Tool interaction

Each tool defines:
- stable tool identifier;
- cursor and touch affordance;
- input state machine;
- configurable properties;
- keyboard shortcut;
- accessibility label;
- undo transaction policy;
- supported platforms;
- test scenarios.

Tool switching must not accidentally commit or discard an in-progress gesture. Escape cancels transient operations where appropriate.

## 3.4 Mobile layout

- Canvas remains primary.
- Compact floating tool rail or radial tool selector.
- Contextual property sheet rather than permanent desktop inspector.
- Two-finger navigation; one-finger drawing when a pen is active or user chooses draw mode.
- Palm rejection delegated to platform input where available, supplemented by configurable touch policy.
- Large touch targets and safe-area handling.
- Undo/redo always readily accessible.
- Layers and precision panels open as sheets or drawers.

## 3.5 Tool wheel

- Configurable slots, initially up to eight.
- Slots may contain tools or frequently used actions.
- Wheel opens around a stable invocation point and supports mouse, touch, and stylus.
- Keyboard alternative and accessible linear menu.
- Tool wheel configuration is user preference, not document content.

## 3.6 Panels

Required panels:
- Layers
- Brush/tool properties
- Color and palette
- Precision/grid/guides
- Object library
- Document/page/artboard
- AI assistant and activity history
- Import/export

Panel state is persisted per workspace. Mobile uses adaptive presentation.

## 3.7 Selection and inspector

Selecting objects shows:
- bounding box and transform handles;
- object type and layer;
- dimensions/position/rotation when meaningful;
- fill/stroke/opacity;
- grouping and lock state;
- relevant geometry controls.

Numeric controls must support units and preserve precision.

## 3.8 AI panel UX

- Conversation view with streaming output.
- Context indicator showing which document, page, layer, selection, or region is shared.
- Provider/model indicator.
- Permission mode: Suggest, Confirm Each Action, or Approved Task.
- Plan preview listing intended changes.
- Diff/preview before execution when feasible.
- Cancel button and progress state.
- Clear record of completed tool actions and undo option.
- Never show API keys or sensitive credential material.

## 3.9 Accessibility

- Keyboard access to all major tools and panels.
- Screen-reader labels for controls.
- High-contrast theme option.
- UI scaling and text-size settings.
- Do not encode state by color alone.
- Respect OS reduced-motion settings.
- Provide alternatives to gesture-only actions.

## 3.10 UI state persistence

Persist workspace preferences separately from document:
- panel layout;
- selected tool;
- recent colors/brushes;
- shortcuts;
- theme;
- tool wheel;
- zoom preferences.

Document state includes only content and document-level presentation settings.

## 3.11 UX acceptance

- New user can create a document, draw, select, edit, save, reopen, and export without external instructions.
- Every destructive operation has clear feedback and an undo path where technically possible.
- Mobile controls remain usable in portrait and landscape.
- Core actions have keyboard and touch alternatives.
