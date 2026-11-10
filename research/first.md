# Concepts App — Complete Clean-Room Rebuild Research & Technical Blueprint

Research started

Rust + Iced

Desktop-first architecture

[Working with Your Infinite Canvas • Concepts App • Infinite, Flexible Sketching](https://images.openai.com/static-rsc-4/Se0MkBU0G2LJdpxI7grML1asyi01KfAkkH7O-UzIgHQkk3NY9PuIQwaqXIn8lT8gr71Qas87qCOEPkt5vobiGeGolyQBHbiCtLbug422nVqcOD1Obvt0oPQIbMqAMM-hPuoBcviJprigVuFSGBHDPntvML8x-c920II-VhOqU44?purpose=inline)

[How to Design in Concepts • Concepts App • Infinite, Flexible Sketching](https://images.openai.com/static-rsc-4/chxNUaAuMTQvWcxni-iEcte96F7Y0m8eQ1nc3QamXCwRXVX-Qwrcngxw6VNdvj5FvA6G-3AunIaxatGu_gn8goVThgtXdJxnQG-XuUntAUEYk-nUHykaqaUjWTm94b4did6nBcv1Vug8h9FwQw-XyCgp08HCCGBiukFIHy0pF5c?purpose=inline)

[How to Design in Concepts • Concepts App • Infinite, Flexible Sketching](https://images.openai.com/static-rsc-4/wURs3t4SGkj-g7fg3OMyFVfxe2FrbiiX5r7HzL58UbSrYVDwN2A4BN9gXP0JiE4kDmde_ofzbffMBMz5ztA7vlluYEsKSltPnKVikxis5H3zn1U4LADLgYHfVRNCTA2IjP5E5fdIb-FBI0sLNsu1xPbAbjbukbzOfx_7QVNIGCQ?purpose=inline)

16

The application we're studying is Concepts, developed by TopHatch.

Official website: [Concepts App](https://concepts.app/en/?utm_source=chatgpt.com)

Official feature list: [Complete Feature List](https://concepts.app/en/features?utm_source=chatgpt.com)

Official manual: [Concepts User Manual](https://concepts.app/en/manual?utm_source=chatgpt.com)

The objective is to understand the product deeply enough to create an independent application that reproduces its important capabilities and workflows, using:

- Programming language: Rust.
- UI framework: Iced.
- Application type: Native desktop application.
- Rendering: Hardware-accelerated 2D drawing engine.
- Architecture: Modular, extensible, document-based.
- Development approach: Clean-room implementation, divided into independently testable phases.

One important distinction: we are rebuilding the product's functionality and interaction concepts, not copying its source code, proprietary assets, branding, brush artwork, or protected implementation. We should independently implement the underlying behaviours and create our own visual identity.

## 1. Understanding what Concepts actually is

Concepts is not simply a drawing application.

It combines several types of software into one environment:

1. Infinite-canvas drawing application.
2. Vector illustration application.
3. Digital sketchbook.
4. Visual brainstorming and mind-mapping workspace.
5. Technical drawing and lightweight CAD environment.
6. PDF annotation and markup application.
7. Design presentation tool.
8. Visual asset library.
9. Layer-based composition editor.
10. Precision measurement and drafting environment.

Its most important distinguishing feature is that drawings are vector-oriented and editable.

Instead of treating every drawing as a flat image, the application stores individual drawing elements that can be moved, adjusted, recolored, resized, transformed, and reorganized after creation.

For example, a user can:

- Draw a line.
- Select that line later.
- Change its colour.
- Change its thickness.
- Move it.
- Adjust its curvature.
- Stretch it.
- Rotate it.
- Put it on another layer.
- Duplicate it.
- Erase only part of it.

This is fundamentally different from a traditional raster painting application.

The other major characteristic is its infinite canvas. There is no requirement to start with an A4 page, a square image, or a fixed drawing boundary. Users can continue drawing in any direction and zoom in or out as necessary.

The official feature documentation confirms vector-based strokes, 120 Hz rendering, stylus dynamics, infinite canvas, advanced selection, precision tools, layers, objects, and extensive import/export capabilities.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://concepts.app\&sz=32)

Infinite, Flexible Sketching+2



## 2. Application structure and major modules

This is the initial functional decomposition we should use when investigating and eventually implementing the application.

## Application architecture — functional map

Desktop Application

Rust + Iced

Canvas Engine

Infinite workspace, camera and rendering

Drawing Engine

Vectors, strokes and brushes

Editing Engine

Selection, transformations and masks

Precision Engine

Grids, guides and measurement

Layer Engine

Layer organization and compositing

Asset Engine

Objects, images and libraries

Document Engine

Projects, saving and recovery

Import / Export

PDF, images and vector formats

Input Engine

Mouse, keyboard and stylus

Workspace UI

Panels, menus and settings

Persistence & Platform Services

Native files · Autosave · Export · Device input · File system

This is our proposed independent architecture, not a claim about Concepts' internal source-code structure.

# 3. Complete feature inventory

I'll organize the research by functional area. Each feature will eventually become an individual requirement in our rebuild specification.

## A. Infinite canvas system

[Working with Your Infinite Canvas • Concepts App • Infinite, Flexible Sketching](https://images.openai.com/static-rsc-4/Se0MkBU0G2LJdpxI7grML1asyi01KfAkkH7O-UzIgHQkk3NY9PuIQwaqXIn8lT8gr71Qas87qCOEPkt5vobiGeGolyQBHbiCtLbug422nVqcOD1Obvt0oPQIbMqAMM-hPuoBcviJprigVuFSGBHDPntvML8x-c920II-VhOqU44?purpose=inline)

[Concepts for iPad: An Adaptable Infinite Canvas to Suit Anyone’s Needs - MacStories](https://images.openai.com/static-rsc-4/0HkXAQ_qZM34DE1bBSAisnAdD6D-5-qTIgSo4bRQOpriIRYmoPJMVTBxzfHO3njhrkJY0Mghw6uPPb2z7DUS6VAQ3rVEHYSjLAAkdPQp8XervIjY_QZYK5poMkGtFDj4bYMGef7_QxG7mvP8syXAUHyU8ZpX4lU-UPtvBrETQ9o?purpose=inline)

[Designing the Astrea Water Bottle • Concepts App • Infinite, Flexible Sketching](https://images.openai.com/static-rsc-4/0BabD5nry7faoYb3UwarDZMEpyKTV-sS3qaln0PgfK2vB-E9flnKamHqIpeo1unL_RB1UYhT1ue0A2ZEu5-aceH5Xxhhq4LKb7g4cL7wfyldswDo4_2tI2BLtQrvoF-SdPsTD_xOkFUQkvfkoBFX6ALZsUE9CDzQME2OiVTK0_w?purpose=inline)

16

Core functionality

The infinite canvas is the foundation of the application.

### A1. Infinite workspace

- No fixed canvas boundaries.
- Drawing in any direction.
- Unlimited conceptual workspace.
- Objects can exist anywhere in world coordinates.
- Ability to work with drawings spread over a large area.
- Canvas background independent of drawing content.
- Artboards can be positioned inside the infinite canvas.

### A2. Canvas navigation

- Zoom in.
- Zoom out.
- Pan horizontally.
- Pan vertically.
- Rotate the canvas.
- Zoom around the pointer or gesture focus.
- Navigate between different drawing areas.
- Maintain the current camera position.
- Return to a preferred zoom level.

### A3. Canvas gestures

- Two-finger drag for panning.
- Pinch gestures for zooming.
- Two-finger rotation.
- Two-finger tap for undo.
- Three-finger tap for redo.
- Configurable interaction gestures.

### A4. Canvas display options

- Different background colours.
- Textured paper backgrounds.
- Transparent background.
- Blueprint background.
- Dark-print background.
- Grid visibility.
- Artboard visibility.
- Measurement visibility.

The official documentation describes the infinite canvas, zooming, panning, rotation, background options, and movable artboards.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://concepts.app\&sz=32)

Concepts Manual+2



Our implementation requirements:

The canvas must use a world-coordinate system independent of screen pixels.

A camera should transform world coordinates into screen coordinates. This will let us support infinite movement without making the document's geometry depend on the monitor resolution.

## B. Drawing engine and brushes

[Brushes and Tools       - Concepts for iOS Manual - Concepts App • Infinite, Flexible Sketching](https://images.openai.com/static-rsc-4/b9XUWLFicGjIfMruqn870EmXxTPT33i6bZdeu4pn1Iifhow2mE8VZegRmbf_sF5cRF_rkKC_qNKAsojh2M3Vruw7KzHJ2Jzk5_fyek_-9aVoelAUf7bvxDalk9KO-CRZODQnF4YzWP_gmV1x40-tCDwJIV558Eon114-R-pAfmc?purpose=inline)

[How to Make a Custom Greeting Card • Concepts App • Infinite, Flexible Sketching](https://images.openai.com/static-rsc-4/ElxPYICYJQZ6GpG6ji-q5ONOeVW_kLPEBY2RSnx7vFUV3IkT0YVd9YVVuB1-BCwxEuxBZlIfnRWWwMgOBKa3MJ2QsOCa4q4U4RODLkoDPAocULNzXp-dsorS6pyz0lA7-Byj-d140MR8MLW4XGrCkIe9fCm99P0RZ_pZibGuSHI?purpose=inline)

[Introducing Vector Watercolor • Concepts App • Infinite, Flexible Sketching](https://images.openai.com/static-rsc-4/I_heZ0EPg4QSoU7Pxb3qFcrtuX82kwRpqyn1ztd45GvdHFsIc432sMYAJ20wyU0oiMGjWUGQDCJzm1nlmLudsoKZbAymvEzirYvxy6h7vFpe22Y1aVhdovIYL9RTo0thS4lvLXcfrWZxb9_Tlyxohpz2swEarHE6cEWRkgWO_nk?purpose=inline)

15

Core functionality

This is one of the largest components of the application.

Concepts uses a vector-oriented drawing system with brushes that can simulate traditional drawing materials.

### B1. Basic drawing tools

The documented tools include:

| Tool            | Function                      |
| --------------- | ----------------------------- |
| Pen             | General-purpose drawing       |
| Fountain Pen    | Velocity-responsive strokes   |
| Dynamic Pen     | Pressure-responsive width     |
| Fixed Width Pen | Constant-width lines          |
| Wire            | Consistent technical lines    |
| Pencil          | Traditional pencil appearance |
| Soft Pencil     | Softer pencil texture         |
| Hard Pencil     | Sharper pencil appearance     |
| Marker          | Marker-like strokes           |
| Airbrush        | Soft, sprayed appearance      |
| Watercolor      | Blended watercolor appearance |
| Dotted          | Dotted vector strokes         |
| Fill            | Filled drawing regions        |
| Text            | Text annotations              |

### B2. Brush properties

Every brush should have configurable properties, including:

- Brush size.
- Brush opacity.
- Stroke smoothing.
- Stroke colour.
- Stroke dynamics.
- Pressure response.
- Tilt response.
- Velocity response.
- Brush preview.
- Size presets.
- Opacity presets.
- Smoothing presets.

### B3. Pressure-sensitive drawing

Stylus input can contain:

- Pressure.
- Tilt.
- Velocity.
- Azimuth.
- Hover information.

These values can influence the resulting stroke.

For example:

A pressure-sensitive brush should produce a thin line when the user presses lightly and a thicker line when the user applies more pressure.

A velocity-sensitive brush should vary its width according to how quickly the user draws.

### B4. Live stroke smoothing

The application needs to process incoming pointer positions to remove unwanted irregularities.

The documented smoothing options include:

- 0%: preserve the original input.
- 50%: reduce unwanted irregularities.
- 100%: produce a straight line between the start and end points.

### B5. Advanced brush capabilities

The official manual also documents:

- Custom brush libraries.
- Brush packs.
- Brush previews.
- Brush editing.
- Stamp-based brushes.
- Reveal-based brushes.
- Brush spacing.
- Brush scatter.
- Brush rotation.
- Rotation along the stroke path.
- Pressure and tilt variation.
- Brush-specific layer assignments.

Custom brush creation is particularly important because it allows the application to become extensible rather than limiting users to a fixed collection of brushes.

[image](https://www.google.com/s2/favicons?domain=https://concepts.app\&sz=32)

Concepts Manual+2

## C. Drawing editing and manipulation

[Selection - Concepts Manual](https://images.openai.com/static-rsc-4/K8E_1tuWyzq7wTPHFNX2oiFNOsjuVeS1hbHi-0zpwq-IisC6ToVIkRY3RjaMsAVtRffizdNF6pP6glWgtCqWIKkdct4ekQvM7XoJYR134p2Pua28LCXzreWif5cqrF3rQRw-srQkrGXWcGc1-FgUdnuo30B6CL-fgJnWOmGj0f0?purpose=inline)

[Select and Edit Your Notes, Drawings and Designs • Concepts App • Infinite, Flexible Sketching](https://images.openai.com/static-rsc-4/alhEplJOGqmcCA_BgAEc1s28Um-8Wi-4nLpzklwyNldyDsjKkqojLOWh9DMHy26QlxmsU1Ztx4HZw9fV48RCzdLFF7F5G4Br7YUnyujbWiYxDXGXe7ZgPKjSnOWn9g0Hwv_jLI3vSoLsW08Bx0WS0uAKk2PdP0UdECn7Hcy9knU?purpose=inline)

[The Nudge Tool • Concepts App • Infinite, Flexible Sketching](https://images.openai.com/static-rsc-4/a3PO8MVInIvejwte1gLDkl27RqY7z7I_69bWhhBhN6fOHksH9501qfCoALlWBOyrdsZJO9Qd1U8lw2q6VvFIWmNZnZgIK0tspnWNZpqLJzE96kUufbvcnC4dwJsEBmDL85SWdS0PcwCvFMLSABDBgL6cjFeSnmjO14fetTaL5tY?purpose=inline)

10

Core functionality

One of the most important parts of Concepts is that drawing elements remain editable after they have been created.

### C1. Selection tools

- Lasso selection.
- Individual item selection.
- Multi-item selection.
- Colour and vector-property selection.
- Tap-and-hold selection.
- Selection of strokes on the active layer.
- Selection across multiple layers.
- Add items to a selection.
- Remove items from a selection.
- Select complete strokes.
- Select partial strokes.
- Filter locked items.
- Filter mask items.

### C2. Selection actions

- Move.
- Copy.
- Paste.
- Duplicate.
- Delete.
- Group.
- Ungroup.
- Lock.
- Unlock.
- Flip horizontally.
- Flip vertically.
- Change layer.
- Change colour.
- Change opacity.
- Change brush properties.

### C3. Transformation tools

- Scale.
- Stretch.
- Rotate.
- Distort.
- Skew.
- Perspective transformation.
- Non-linear transformation.
- Aspect-ratio control.
- Corner control points.
- Exact rotation values.
- Exact scale values.

### C4. Nudge tool

This is a particularly interesting feature.

Instead of selecting an entire object and moving it, the user can interact directly with a vector stroke and push or pull its geometry.

Examples:

- Correct an imperfect curve.
- Adjust a corner.
- Improve a line's shape.
- Reshape an illustration without redrawing it.

### C5. Slice tool

The Slice tool acts as a vector eraser.

It removes the portion of a stroke that intersects with the eraser.

Unlike a normal raster eraser, it modifies the underlying vector geometry.

It should support:

- Variable eraser width.
- Partial stroke removal.
- Splitting strokes.
- Layer filtering.
- Locked-element filtering.

### C6. Mask tools

Two types are documented:

- Hard Mask.
- Soft Mask.

These are non-destructive editing tools.

They hide portions of drawings or imported images without permanently deleting the underlying content.

This distinction is important: slicing permanently modifies a stroke, whereas masking preserves the original content.

### C7. Grouping

Multiple drawing elements can be grouped so that they can be manipulated as one logical object.

Grouping should support:

- Selecting an entire group.
- Moving the group.
- Rotating the group.
- Scaling the group.
- Duplicating the group.
- Ungrouping the elements.

### C8. Highlight selection

When selecting an object, the application can visually distinguish the selected elements from the rest of the drawing.

This improves editing clarity, particularly in complicated drawings.

[image](https://www.google.com/s2/favicons?domain=https://concepts.app\&sz=32)

Concepts Manual+2

## D. Colour management system

[Colors       - Concepts for iOS Manual - Concepts App • Infinite, Flexible Sketching](https://images.openai.com/static-rsc-4/ptXATB_DCNXKD7bWAEmLUK9VEvdCCJnQxOSUF88inyOBKJxdSONzRx_7Jsv052c-g08rz9-CoCqikdkXEeUDyaMcqoa0HXV2PEZOkIfL7mh58sB-ePxRCHOFvgmvr-vzHbOHkXx_JK48HY8JbQD0IlJeG3UVYJXja3GwlKXFuWY?purpose=inline)

[Colors       - Concepts for iOS Manual - Concepts App • Infinite, Flexible Sketching](https://images.openai.com/static-rsc-4/3heboU7udXlyGYVDHRHKrfao5Wj3D1lth9vrlnCkDVB_mcqlgU4xUOvvgie5HA6IeyWyG7McCw--VuNIMgyvw9cc9EdtNJzLk4DBQiyJkHFRCP5O6Yai1XT_hxwxKYgYMheXeZw5o8QCOZj9UzJYYKijeNHAPsM3SkcLXH9g3B4?purpose=inline)

[Designing Concepts: 10 Years of Creative Development • Concepts App • Infinite, Flexible Sketching](https://images.openai.com/static-rsc-4/8TQHGZE8xwpElAx1B_O55We2DjPqsS1MwdpbvVzJ_za98Sp-FZgsw9z5S8z_LTG8nZId_N7MpK3t21gzQ63xYM8ohjxVPGwyfFVWX6JOgawf8k80rXMaYOWQ59iaKycI6GFnVKW1kh1_7Heivi7G7zhT2goAMVphRDS2GD2oCYQ?purpose=inline)

9

### D1. Colour selection

The documented colour systems include:

- COPIC colour wheel.
- HSL colour selection.
- RGB colour selection.
- Canvas colour picker.
- Tap-and-hold colour sampling.
- Custom colour palettes.
- Dynamic colour palettes.

### D2. Colour picker

The colour picker can sample colours from:

- Existing drawings.
- Imported photographs.
- Other canvas elements.

The vector-oriented selection colour picker also allows users to retrieve brush and stroke properties, not just a colour.

### D3. Colour palettes

- Create palettes.
- Save palettes.
- Switch between palettes.
- Reuse saved colours.
- Share palettes where supported.
- Mix palette colours using a gradient-style interaction.

### D4. Colour editing

The application should support changing the colour of previously created strokes without redrawing them.

It should also support opacity adjustments and brush-property changes.

Rebuild requirement: Implement colour handling as a central subsystem. Avoid implementing a separate colour picker for every drawing tool.

[image](https://www.google.com/s2/favicons?domain=https://concepts.app\&sz=32)

Concepts Manual+2

## E. Precision drawing and technical design tools

[How to Design in Concepts • Concepts App • Infinite, Flexible Sketching](https://images.openai.com/static-rsc-4/QBwHt3nplpKJEs0-GUOFB0Z7uOBmtGyeuk4nfYnzg7JXe-OHEg2AobZJc-ROMqriS9-mDkBRC0ZATW5LeuH2vinIk3XTeWNf5IlZjHTOKm0SELL2_vedXTcYRfsMU-naClvDfM-uzl4-KLggK_NXmp2qqXOfOnIlAAj8Wd-n7ig?purpose=inline)

[How to Sketch with a Perspective Grid • Concepts App • Infinite, Flexible Sketching](https://images.openai.com/static-rsc-4/e9qMffmVG58iIwpjN6avtEc1bEm1J6GokB4riGWngm7oJ36TV7TQv2QQ0rY0fHPNiBkxpuoxM4pz1DHiBlw5U6L4UaQcsKS0CMsLUUmKGjZNq-lQ7yjRmMUJc_3Ok2zhTxRCtvCFkS_CGb7GSW0HC6ch0PpC34HWIXzyhMhAobk?purpose=inline)

[How to Edit Your Grid • Concepts App • Infinite, Flexible Sketching](https://images.openai.com/static-rsc-4/PIw8i4oPeWJCwtYz_DMRkrpx-uXW1WvrgARFpMo-W81RAdtB4iFgm7ecGj6yteNbFU5X53MZ7fgykFpV5gocss2zNauADcNoA-hI8yRSDv4JqNk1Di1q1h-_2Qb-_vPSqs7_uu_fxHyuClinB3FdkU1VLLvCD25L97DFeU7MxyQ?purpose=inline)

12

Important differentiator

This is where Concepts becomes more than a sketching application.

### E1. Grid system

The documented grid types include:

1. Dot grid.
2. Graph paper.
3. Lined paper.
4. Isometric grid.
5. Triangle grid.
6. One-point perspective grid.
7. Two-point perspective grid.
8. Three-point perspective grid.
9. Custom grids.

Grid configuration should include:

- Grid spacing.
- Grid positioning.
- Grid visibility.
- Grid orientation.
- Grid appearance.
- Custom grid editing.
- Grid snapping.

### E2. Snap system

Snapping allows strokes and objects to align with important positions.

Potential snap targets include:

- Grid intersections.
- Grid lines.
- Existing drawing points.
- Geometric features.
- Guide boundaries.
- Measurement references.

We should treat snapping as a dedicated geometric service, rather than adding snapping logic separately to each brush.

### E3. Shape guides

The documented shape guides include:

- Line.
- Arc.
- Angle.
- Rectangle.
- Ellipse.

A guide should work like a digital ruler.

The user can position the guide and draw against it to produce precise geometry.

### E4. Shape recognition

The user can draw a rough shape and hold at the end of the drawing gesture to trigger recognition.

Documented recognized shapes include:

- Straight lines.
- Curved lines.
- Arrows.
- Triangles.
- Squares.
- Rectangles.
- Circles.
- Ellipses.

The detected shape can then be adjusted before finalizing it.

### E5. Real-world measurement

Concepts supports drawing at a defined scale.

The documented units include:

| Measurement category | Units                                        |
| -------------------- | -------------------------------------------- |
| Digital              | Pixels, points                               |
| Metric               | Millimetres, centimetres, metres, kilometres |
| Imperial             | Inches, feet, yards, miles                   |

Additional functionality:

- Automatic unit scaling.
- Custom drawing scale.
- Measurement annotations.
- Stroke length measurements.
- Measurement precision.
- Measurement formatting.
- Selection scaling information.

For example, a user can draw a line representing 5 metres rather than treating it as an arbitrary number of screen pixels.

### E6. Measurement layer

Measurements can be organized independently from normal drawing content.

This allows technical annotations to be shown or hidden without affecting the rest of the design.

[image](https://www.google.com/s2/favicons?domain=https://concepts.app\&sz=32)

Concepts Manual+3

## F. Layer management

[Layers - Concepts for iOS Manual - Concepts App • Infinite, Flexible Sketching](https://images.openai.com/static-rsc-4/niHlgYSzgX6fXJEbR2t9V09mTL6wVpPm9z7eXQSYNu7HD5ubxfpntB_ugHlce9xlsll6BlQKItGLq7P_XRTtYLrffPayaHOq1mtp0DpRNkp4RUd-r4pYxPWXTdzSmbPQBhS4I4QA3J64hzFja978bsBzXTM-5COcNf-IIsUBvpg?purpose=inline)

[Layers       - Concepts for Android & Chrome OS Manual - Concepts App • Infinite, Flexible Sketching](https://images.openai.com/static-rsc-4/kBW1fJ4wEJC9jTCjcmdo3uobSBulb7ak6M6KXq8agBohm8qx9jV9Z2PE92_Zzoo05X8HaSGDljxfYtmoQhyFLPZ9OWsX0HmUXCqOLLVlER9YK0cFjMxXTQTowTegs0DDgp7csVpyhZgpF9CmO83dQ4-2Lw7bW9PkIRfr5o8xTwo?purpose=inline)

[Working with Your Infinite Canvas • Concepts App • Infinite, Flexible Sketching](https://images.openai.com/static-rsc-4/Se0MkBU0G2LJdpxI7grML1asyi01KfAkkH7O-UzIgHQkk3NY9PuIQwaqXIn8lT8gr71Qas87qCOEPkt5vobiGeGolyQBHbiCtLbug422nVqcOD1Obvt0oPQIbMqAMM-hPuoBcviJprigVuFSGBHDPntvML8x-c920II-VhOqU44?purpose=inline)

9

Core functionality

Layers are essential for maintaining complicated projects.

### F1. Layer types

The documented system includes:

- Normal drawing layers.
- Background layer.
- Grid layer.
- Measurement layer.

### F2. Layer creation

- Create layers.
- Delete layers.
- Duplicate layers.
- Rename layers.
- Merge layers.
- Reorder layers.
- Move selected strokes to another layer.

### F3. Layer visibility

- Show layer.
- Hide layer.
- Toggle visibility.
- Focus mode.
- Isolate a particular layer.

### F4. Layer editing

- Lock layer.
- Unlock layer.
- Change opacity.
- Select everything on a layer.
- Merge down.
- Duplicate.
- Delete.

### F5. Automatic layering

This is a particularly useful feature.

Concepts can automatically organize drawing elements according to the drawing tool being used.

For example:

- Pen strokes go into a pen layer.
- Pencil strokes go into a pencil layer.
- Marker strokes go into a marker layer.
- Other supported tools have their own assigned layer categories.

Changing the drawing tool can automatically switch to its corresponding layer.

### F6. Manual layering

Users can disable automatic sorting and work with a manually organized layer stack.

They can then draw with different brushes on the same active layer.

### F7. Layer ordering

Users can drag layers to rearrange them.

The ordering affects how drawing content is composited.

### F8. Layer restrictions

Concepts currently documents a five-layer limit in its free offering, with unlimited layers in paid offerings.

For our implementation, I recommend an unrestricted internal layer model, with any future limits handled through licensing rather than the drawing engine.

[image](https://www.google.com/s2/favicons?domain=https://concepts.app\&sz=32)

Concepts Manual+2

## G. Object library and reusable assets

[Objects - Concepts for Windows Manual - Concepts App • Infinite, Flexible Sketching](https://images.openai.com/static-rsc-4/_EB-2H9hDmsoz58YfCbNEEfwBVHEit3oIengrVA3fcP8RsT1vpnuxiTW1XsoLfrNVEsA2Gq9c61GLmn_o0B2O9iTEUQgY0TuBeFZvc2tq5kL9Nn7-bcS881bOwih0keqY79wgALb33X1g5osmDS39HXBa3bo0rpsF_lyQd_HMhQ?purpose=inline)

[Objects - Concepts Manual](https://images.openai.com/static-rsc-4/VpojrRHfR_NCmHAj9d4EIbJzR53fOVdsssFw9quTm8yFJmkJRd-N3DKHbnvR3yO5eZdKXV0cFonRC72n4UI-JjdPkC9FgwnZ422BD2XepGMm9Cmg27JDp5QhTWVEwEX77_vUGoHtR2cZPsYw_n2-rowz1Q3DmdV1hRrK43dgOBE?purpose=inline)

[Using the Object Library and Make Your Own Objects • Concepts App • Infinite, Flexible Sketching](https://images.openai.com/static-rsc-4/9jXhJV1sdycSJtQ4suzDSqcF3H_BXE9Ibr9EPwrZINNYPrii10jEG9wSsNV_390katCBBeBH4t0Wigr_JHZH4TKiddkbaNMnJdlrBtJLGWom_DtyX_66twzpv6wpzzXaWhrA-nWnk5nvKOZJTSd6MB9m-I9DP1bn0MvRY4ZbqDM?purpose=inline)

11

This module turns the application into a reusable design environment.

### G1. Built-in objects

The application provides reusable vector objects through asset libraries.

The documented categories include basic shapes and professional object packs.

### G2. Object management

- Browse object packs.
- Search objects.
- Search by category.
- Preview objects.
- Favourite objects.
- Favourite packs.
- Open individual packs.
- Add objects to drawings.
- Drag and drop objects.
- Resize objects.
- Rotate objects.
- Move objects.
- Duplicate objects.
- Mirror objects.
- Change object colours.

### G3. Custom object creation

Concepts supports a feature called Make Your Own Objects.

Users can:

1. Create a drawing.
2. Select the desired strokes.
3. Create an object pack.
4. Give the pack a name.
5. Add selected drawings as reusable objects.
6. Organize the object collection.
7. Reuse objects in other drawings.

### G4. Object properties

Custom objects can inherit selected properties from the currently active drawing tool.

The documented options include:

- Brush type.
- Size.
- Colour.

### G5. Asset search

Users can search:

- Object names.
- Object packs.
- Object descriptions.
- Stock images where available.

### G6. Stock image integration

Concepts integrates Pexels image search into its object library on supported platforms.

Users can search for stock photographs and bring them directly into their drawings.

### G7. Shared asset libraries

Concepts supports sharing certain custom object packs between users and devices.

Our rebuild recommendation: Build an independent asset library system with a clear separation between:

- System assets.
- User-created assets.
- Imported assets.
- Asset packs.
- Favourites.
- Search indexes.

The library should not be hardwired into the canvas engine.

[image](https://www.google.com/s2/favicons?domain=https://concepts.app\&sz=32)

Concepts Manual+2

## H. Image and document importing

[Import - Concepts Manual](https://images.openai.com/static-rsc-4/IOj1ArevBwp_N3wIr44Td6V_-cXE4kIXmmIA4q23gTSi9t17Zzx2pUX8zeOsGRzi3L68rUsMyzU_-pkEA3BQpDZKMljU6S-225QVIrSndsGU8ym3HwUZdItsr0Qvr3hsuKP7qVTNKpX6N6-QqRU2pn2NmObTq44Uc-T9OzE94cI?purpose=inline)

[How to Edit a PDF • Concepts App • Infinite, Flexible Sketching](https://images.openai.com/static-rsc-4/DuUtiMSZi4TITDEzdiaoINHNeU2EDFLS2HVZ9-dgEErTWt4Xi-dORyO8sNweFbrfGzrZOVlLNpP2ex1BHtar7oyJxmn2SPsVznjJsH2YjGHq7DAzVydemjiJITrRv5hjUugzNEqthms8ZLPku_pc-37sDaEENaawX-uwvbd6tKc?purpose=inline)

[Draw Your Own Hand • Concepts App • Infinite, Flexible Sketching](https://images.openai.com/static-rsc-4/xhyGsDpLxWpHRs4itpfNI4WBg3nDjQwuGtkAK0-OM_ZJ6Prk8NTD9kIPAYG7pnqJAij_2jA6lgq6YKYw_t0rK15jMyKkxIzOQsJR3vy4ZjUWpHufDm5oV3uYPS9H9wTDipHGBzfM5__RUMtbJTxsgRw7Ypp9Dfu-togh8tv7Zsc?purpose=inline)

13

### H1. Supported import formats

The official documentation lists:

| Format                    | Import                                                                     |
| ------------------------- | -------------------------------------------------------------------------- |
| JPG / JPEG                | Yes                                                                        |
| PNG                       | Yes                                                                        |
| PSD                       | Yes                                                                        |
| PDF                       | Yes                                                                        |
| Native Concepts documents | Yes                                                                        |
| SVG                       | Not established as a supported import format in the published feature list |
| DXF                       | Export-only in the published feature list                                  |

An important limitation: PSD and AI files may be flattened into PNG images when imported, rather than retaining editable Photoshop or Illustrator layers.

### H2. Import methods

- Import menu.
- Drag and drop.
- Clipboard.
- Photo library.
- File system.
- Device camera where supported.

### H3. Image placement

Imported images should be:

- Positioned.
- Resized.
- Rotated.
- Duplicated.
- Grouped.
- Locked.
- Moved between appropriate layers.
- Masked.

### H4. Scale-aware importing

Imported documents and images can be positioned according to their physical or drawing scale.

This is particularly useful for architectural drawings and technical plans.

### H5. Clipboard support

The application can accept clipboard content from other applications and transfer selected content between supported applications.

[image](https://www.google.com/s2/favicons?domain=https://concepts.app\&sz=32)

Concepts Manual+2

## I. PDF editor and annotation system

[コンセプトアプリ • 無限に自由自在なスケッチング](https://images.openai.com/static-rsc-4/bBdoFqF24BRZGdbwnFqKkkiSZ09bW_oZuM4bH-gbFcNAyFpGsCdqmQJFqltI2wu5jgPdOchzImJUARuONmqY60DfwuHLnLwPUrXaZOyuywo6QhT5WkGQ0mlM_SA_XfFu3MNWXgqOE2dsUkQsFCuCxD62c7ToO2JMr6rNzUIds_U?purpose=inline)

[Concepts for Business](https://images.openai.com/static-rsc-4/b1W8UcnRM6Zr3Hkf3aeLlecOIVmfCB7zySh8Np21WUTR7Q7tKlGlLDCtaoQPI6wk33_gE-ZDf21EKVsWJxa2DEjdvaw9p-iTX_wazMFEb1vDqvfT6Cl0fKIVBPYX7V0Ca2-5xFw9FiXdIxurq2_FOhrq0h2UNIY1pqPFYTrAXPQ?purpose=inline)

[PDF       - Concepts for iOS Manual - Concepts App • Infinite, Flexible Sketching](https://images.openai.com/static-rsc-4/UjB6CXDiK0mMwceeZR3RJuAKivUTL15k9HTBGtGzWr_O0gTebzYokNPYZPt0Cm3xgYEuIX-TFfVqdmoj3ufTAkFGTImwM1u8lmwf9vC9hfqUQxwts4dMiot5VF7qd0--qDboX5M7snKWgPJcx1Gxtj0X4tXZFJRSKHg7JOkNQCw?purpose=inline)

13

This is a substantial feature, not merely a file import operation.

### I1. PDF import

- Import single-page PDFs.
- Import multi-page PDFs.
- Display page thumbnails.
- Select individual pages.
- Drag pages onto the canvas.
- Preserve the original document's page dimensions.
- Import pages at the correct physical scale.

### I2. PDF editing

- Annotate PDF pages.
- Draw over documents.
- Add text.
- Add signatures.
- Position annotations.
- Move pages.
- Duplicate pages.
- Delete pages.
- Lock pages.
- Group content.
- Mirror pages.

### I3. Multi-page management

Users can rearrange pages by positioning them in the desired order.

### I4. PDF export

- Export PDF documents.
- Export selected content.
- Export PDF bounds.
- Include original pages.
- Include annotations.
- Include background or paper.
- Include grid where appropriate.
- Export flattened documents.

### I5. PDF transparency

The imported PDF page can have its alpha channel enabled or disabled.

### I6. PDF page replacement workflow

A useful documented workflow is importing an existing multi-page PDF, editing selected pages, and exporting the updated document while retaining original pages.

One limitation to note: Concepts' documentation describes PDF page workflows and editing, but does not establish a general-purpose full PDF authoring system equivalent to a dedicated PDF publishing application.

[image](https://www.google.com/s2/favicons?domain=https://concepts.app\&sz=32)

Concepts Manual+2

## J. Export engine

[PDF       - Concepts for iOS Manual - Concepts App • Infinite, Flexible Sketching](https://images.openai.com/static-rsc-4/UjB6CXDiK0mMwceeZR3RJuAKivUTL15k9HTBGtGzWr_O0gTebzYokNPYZPt0Cm3xgYEuIX-TFfVqdmoj3ufTAkFGTImwM1u8lmwf9vC9hfqUQxwts4dMiot5VF7qd0--qDboX5M7snKWgPJcx1Gxtj0X4tXZFJRSKHg7JOkNQCw?purpose=inline)

[How to Create Custom Gift Wrap • Concepts App • Infinite, Flexible Sketching](https://images.openai.com/static-rsc-4/uqJb3INRcEORPo8uE3FTiWlIiKLkfTtemaXpFVHJplVZrFOi1qUVZkvD8AGlhb26lzwVa7mjyR0DrLmcZOdnhbw_Mtta880qEH-Ixg11il0Afo2Ho_z4fb_6tki_5NcwmH7FUAmkmWujCgJvf2PEonJAx6JNavdT8jhnUKfdcZc?purpose=inline)

[How to Just Sketch in Concepts • Concepts App • Infinite, Flexible Sketching](https://images.openai.com/static-rsc-4/E6GNwNBK9q6Gc-PAnzOJgtm9gfnlKgSmLpPwJyjGN9PhQJ_asJ6vbKPRPk_iTj-U_N9rL39_CUBS8cebbxnmFbFCmR_8IJvaohIvNs3a4utcPsLlg3P_6p2F2sbDqeDyALKRI7FTF9m76D52hagAevcotkL_qmSHbmg8rIE2oAg?purpose=inline)

8

### J1. Supported export formats

| Format        | Export capability        |
| ------------- | ------------------------ |
| JPG           | Yes                      |
| PNG           | Yes                      |
| PDF           | Yes                      |
| PSD           | Yes                      |
| SVG           | Yes                      |
| DXF           | Yes                      |
| Native format | Yes, for saving projects |

### J2. Export regions

Users can choose what to export:

- Entire drawing.
- Artboard.
- Screenshot.
- Selection.
- PDF bounds.

### J3. Export quality

The documented output settings include:

- 72 PPI.
- 150 PPI.
- 300 PPI.
- 600 PPI.

For pixel-based screenshots and artboards, other output scaling options are also available.

### J4. Export options

- Transparent background.
- Include paper background.
- Include grid.
- Use filters.
- Wireframe DXF export.
- Visible layers only for PSD.
- Original PDF pages.

### J5. Selection export

Selected drawing content can be exported independently of the rest of the canvas.

### J6. External application workflows

The application supports clipboard transfer and drag-and-drop workflows on supported platforms.

For our implementation, we should treat exporting as its own subsystem, with separate format encoders and a shared document-rendering interface.

[image](https://www.google.com/s2/favicons?domain=https://concepts.app\&sz=32)

Concepts Manual+2

# 4. Additional application functionality

The features above cover the main drawing and editing experience. There are additional capabilities that should also be included in our product research.

## K. Project gallery and file management

[Concepts 6 for iOS Release • Concepts App • Infinite, Flexible Sketching](https://images.openai.com/static-rsc-4/oFh2B-mIT3mV38LUYpGi1HKwKwhP_qm_MzbcHPHwNEjZLstxgjdEcTD1o2973WnRyx3hUNDkKRUHjRY2W-23ZzL2aNgPISqnl4fIlFaxRs-O9fAexgLycSFB0q0pB5Uq9RApxDuRVSWlSaXm-fRpJkvqnNpIdfl_pUV4sZdMGwU?purpose=inline)

[concepts.app](https://concepts.app/releases/concepts-6-for-ios?utm_source=chatgpt.com)

Concepts starts in a Gallery where users manage their drawings.

Documented capabilities include:

- Create new drawings.
- Open existing drawings.
- Rename drawings.
- Organize projects.
- Create folders.
- Group drawings.
- Sort drawings.
- Manage files.
- Access native documents through the file system.
- Print drawings.
- Open documents in place.
- Cloud backup and platform-specific synchronization.

The feature list also documents unlimited drawings and projects.

For our rebuild, this means we need a dedicated document-management module separate from the actual canvas.

[image](https://www.google.com/s2/favicons?domain=https://concepts.app\&sz=32)

Concepts Manual+2

## L. Artboards and composition

[How to Design in Concepts • Concepts App • Infinite, Flexible Sketching](https://images.openai.com/static-rsc-4/22WHh2u8-E3hxyKfA9SWLa76L8Jq3aVvXBU_02aPcmKhFxU-HC5PdAWcrmfCqe93yuWRNsCERuwsACdlhfcw2ZUcYo_QMZS0_q09vRxGl_k6tWW62p4dyzWsWLyKs5xqc9naV9mwbQMDJL0pvQgaUdO9qjUse3_bkPw-pp2mtFo?purpose=inline)

[concepts.app](https://concepts.app/en/tutorials/how-design-concepts/?utm_source=chatgpt.com)

Although the canvas is infinite, users can define specific artboards.

Features include:

- Predefined artboard dimensions.
- Custom dimensions.
- Artboard positioning.
- Artboard movement.
- Artboard resizing.
- Artboard-based export.
- Artboard visibility and interaction settings.

This is important because it lets users work freely while still producing deliverables with defined dimensions.

[image](https://www.google.com/s2/favicons?domain=https://concepts.app\&sz=32)

Concepts Manual+1

## M. Keyboard, mouse and touch interactions

Concepts is designed around stylus and touch input, but it also supports keyboard shortcuts and other configurable interactions.

Documented capabilities include:

- Keyboard shortcuts.
- Configurable gestures.
- Stylus input.
- Finger input.
- Mouse navigation.
- Tool-wheel interaction.
- Tool-bar interaction.
- Tap-and-hold actions.
- Temporary navigation modes.

Some documented shortcuts include:

| Shortcut         | Action      |
| ---------------- | ----------- |
| Ctrl + Z         | Undo        |
| Ctrl + Shift + Z | Redo        |
| Space            | Pan mode    |
| R                | Rotate mode |
| S                | Zoom mode   |

The exact interaction behaviour depends on platform and input configuration.

[image](https://www.google.com/s2/favicons?domain=https://concepts.app\&sz=32)

Concepts Manual+2

## N. Workspace interface

[您的工作空间 - 概念画板Windows版本用户手册 - 概念画板App • 无限、灵活地进行绘图](https://images.openai.com/static-rsc-4/Ci-XvZBZ5tjj6sh4SHh3TWcOK28X5W2NLxarVEWvuZ3tw1NrtgImo_orLjCstUQtPzHYQyxXlMZKoXeaQNonjfpZG5bzIwcH_tacjmoxt1ucEHfMy0-kE5UxT15ZN2SXqlaTocXo9L6_SRqmc9fOuZonZbPDnNwdqjMf_bhNpYs?purpose=inline)

[Layers - Concepts for iOS Manual - Concepts App • Infinite, Flexible Sketching](https://images.openai.com/static-rsc-4/niHlgYSzgX6fXJEbR2t9V09mTL6wVpPm9z7eXQSYNu7HD5ubxfpntB_ugHlce9xlsll6BlQKItGLq7P_XRTtYLrffPayaHOq1mtp0DpRNkp4RUd-r4pYxPWXTdzSmbPQBhS4I4QA3J64hzFja978bsBzXTM-5COcNf-IIsUBvpg?purpose=inline)

[Precision Tools - Concepts for Windows Manual - Concepts App • Infinite, Flexible Sketching](https://images.openai.com/static-rsc-4/-7sNxrlyV8uIyn-kJzf9rNMrgLS9fViC5tI5PNz6T7jQkzJIqECz9Sc7M7Tr5ibh_mFoGAl9K9aUE4A-2E5Z8i_u-X6_vf5wGu5nLQ5343O3ZFYS8FBEEkqYaofh1ZmkmU8TMTcmjM0UjOUnlnrdSkeLnF9O69rbZFs79BcarfM?purpose=inline)

13

The interface contains:

- Status bar.
- Drawing title.
- Tool Wheel.
- Tool Bar.
- Colour controls.
- Precision panel.
- Layers panel.
- Object Library panel.
- Zoom and rotation controls.
- Import controls.
- Export controls.
- Settings.
- Help.
- Store.

### Movable interface elements

A distinctive feature is that several interface panels can be moved around the canvas.

These include:

- Tool Wheel.
- Precision panel.
- Layers panel.
- Object Library.

The user can customize their workspace by moving these elements.

### Tool Wheel

The documented Tool Wheel supports up to eight configurable tool slots.

It includes:

- Tool selection.
- Brush settings.
- Size adjustment.
- Opacity adjustment.
- Smoothing adjustment.
- Undo.
- Redo.
- Colour selection.

Users can also choose a more traditional Tool Bar layout.

[image](https://www.google.com/s2/favicons?domain=https://concepts.app\&sz=32)

Concepts Manual+2

## O. Presentation and collaboration

Concepts advertises automatic Presentation Mode on supported platforms.

The purpose is to allow users to sketch live while presenting ideas to clients or teammates, including through virtual meeting applications.

We should distinguish this from a fully collaborative multiplayer canvas: the published feature description establishes live presentation, but does not establish a complete simultaneous multi-user editing system.

Potential separate features for our product:

- Presentation mode.
- Hide editing controls.
- Display canvas cleanly.
- Follow current drawing activity.
- Share screen.
- Export presentation snapshots.

[image](https://www.google.com/s2/favicons?domain=https://www.concepts.app\&sz=32)

Infinite, Flexible Sketching+1

## P. Search and discovery

The documented application features include:

- Searching the object library.
- Searching object packs.
- Searching stock images.
- Platform-specific drawing discovery and search capabilities.
- Deep handwritten-note search in beta on a supported platform.

We should investigate the exact search implementation separately rather than assume every platform has identical functionality.

## Q. Help and learning

Concepts includes:

- Getting-started guidance.
- Tutorials.
- Drawing lessons.
- User manual.
- Help and support.
- Product updates.
- Community content.

These are supporting product features rather than essential drawing-engine requirements.

# 5. Feature completeness and platform differences

An important finding: Concepts is not currently identical on every platform.

Its published feature matrix distinguishes between iOS/iPadOS and Windows/Android/ChromeOS. Some features are platform-specific or still being brought into parity.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://concepts.app\&sz=32)

Infinite, Flexible Sketching+1



For our project, we should avoid reproducing those limitations unnecessarily.

We can establish a consistent desktop feature set from the beginning.

| Functional area         | Concepts feature documented       | Rebuild target         |
| ----------------------- | --------------------------------- | ---------------------- |
| Infinite canvas         | Yes                               | Full                   |
| Vector drawing          | Yes                               | Full                   |
| 120 Hz rendering        | Yes                               | Target                 |
| Pressure and tilt       | Yes                               | Full                   |
| Brush smoothing         | Yes                               | Full                   |
| Custom brushes          | Yes                               | Full                   |
| Selection               | Yes                               | Full                   |
| Advanced transforms     | Yes                               | Full                   |
| Colour management       | Yes                               | Full                   |
| Grids                   | Yes                               | Full                   |
| Perspective grids       | Yes                               | Full                   |
| Shape guides            | Yes                               | Full                   |
| Shape recognition       | Yes                               | Full                   |
| Measurements            | Yes                               | Full                   |
| Infinite layers         | Paid feature                      | Full                   |
| Automatic layering      | Yes                               | Full                   |
| Object library          | Yes                               | Full                   |
| Custom objects          | Yes                               | Full                   |
| PDF editing             | Yes                               | Full                   |
| PNG/JPG export          | Yes                               | Full                   |
| PSD export              | Yes                               | Full                   |
| SVG export              | Yes                               | Full                   |
| DXF export              | Yes                               | Full                   |
| Artboards               | Yes                               | Full                   |
| Project gallery         | Yes                               | Full                   |
| Presentation            | Platform-dependent                | Planned                |
| Cloud synchronization   | Platform-dependent                | Later                  |
| Real-time collaboration | Not established as a core feature | Optional future module |

# 6. Recommended Rust + Iced technical architecture

Now that we have the functional inventory, we need to translate it into an actual engineering architecture.

I strongly recommend that we do not build the entire application inside Iced widgets.

Iced should manage the application interface, while a separate Rust drawing engine handles geometry, input, rendering, and document operations.

## Proposed engineering architecture

Application Interface

Iced · Windows · Linux · macOS

Application Controller

State · Commands · Events · Tool routing

Canvas Core

Camera and world coordinates

Input Core

Mouse, keyboard, stylus

Geometry Core

Paths, curves and transforms

Brush Core

Stroke generation and dynamics

Editing Core

Selection, slicing and masks

Precision Core

Grids and measurement

Document Core

Objects, layers and history

Rendering Core

GPU drawing and compositing

Infrastructure

Native file format · Autosave · Import · Export · Persistence

## 6.1 Rust workspace structure

I recommend a Cargo workspace with independently testable crates.

```
our-app/
│
├── Cargo.toml
│
├── crates/
│   ├── app/
│   │   └── Application entry point
│   │
│   ├── ui/
│   │   └── Iced interface and panels
│   │
│   ├── canvas/
│   │   └── Infinite canvas and camera
│   │
│   ├── renderer/
│   │   └── GPU rendering engine
│   │
│   ├── geometry/
│   │   └── Vector geometry and paths
│   │
│   ├── input/
│   │   └── Mouse, keyboard and stylus
│   │
│   ├── brushes/
│   │   └── Brush definitions and stroke engine
│   │
│   ├── selection/
│   │   └── Selection and transformations
│   │
│   ├── layers/
│   │   └── Layer management and compositing
│   │
│   ├── precision/
│   │   └── Grid, snapping and measurement
│   │
│   ├── objects/
│   │   └── Reusable object libraries
│   │
│   ├── documents/
│   │   └── Document and project model
│   │
│   ├── history/
│   │   └── Undo and redo
│   │
│   ├── importers/
│   │   └── File importing
│   │
│   ├── exporters/
│   │   └── File exporting
│   │
│   └── storage/
│       └── Saving and recovery
│
├── assets/
│   ├── icons/
│   ├── fonts/
│   ├── brushes/
│   └── textures/
│
└── tests/
    ├── geometry/
    ├── rendering/
    ├── documents/
    └── integration/
```

This structure keeps the drawing engine independent of the interface.

That means we could eventually replace Iced without rewriting the entire application.

## 6.2 Recommended Rust technologies

These are proposed implementation technologies, not technologies confirmed to be used internally by Concepts.

| Technology | Responsibility                      |
| ---------- | ----------------------------------- |
| Rust       | Main programming language           |
| Iced       | Desktop interface                   |
| wgpu       | GPU rendering                       |
| lyon       | 2D path tessellation                |
| kurbo      | 2D curves and geometry              |
| glam       | Vector and matrix mathematics       |
| rstar      | Spatial indexing                    |
| image      | Image processing                    |
| resvg      | SVG rendering                       |
| tiny-skia  | 2D rasterization                    |
| serde      | Document serialization              |
| SQLite     | Application metadata                |
| zip        | Native document packaging           |
| tracing    | Diagnostics and performance tracing |

We should evaluate exact crate versions and compatibility when creating the project.

### Important architectural decisions

1\. Independent rendering engine

The canvas renderer should not depend on Iced's widget tree to render every individual drawing element.

We should create a dedicated canvas component that can manage its own rendering pipeline.

2\. Vector-first document storage

Drawing strokes should be stored as editable geometry and brush parameters.

We should not flatten the entire document into a bitmap after each operation.

3\. GPU acceleration

The rendering engine should use GPU acceleration for:

- Stroke rendering.
- Canvas compositing.
- Layer rendering.
- Image rendering.
- Zooming and panning.
- Selection highlighting.
- Grid rendering.
- Mask compositing.

4\. Spatial indexing

An infinite canvas can eventually contain thousands or millions of objects.

We should use a spatial index to identify visible or relevant elements instead of processing the entire document every time the camera moves.

5\. Rendering separation

The document model should remain independent from the rendering representation.

For example, a vector stroke should remain a vector stroke in the document, even if the renderer converts it into GPU geometry for display.

## 6.3 Document data model

I recommend starting with a structure similar to this:

```
struct Document {
    id: DocumentId,
    name: String,
    camera: Camera,
    settings: WorkspaceSettings,
    layers: Vec<Layer>,
    objects: Vec<CanvasObject>,
    artboards: Vec<Artboard>,
    assets: Vec<AssetReference>,
}

struct Layer {
    id: LayerId,
    name: String,
    visible: bool,
    locked: bool,
    opacity: f32,
    objects: Vec<ObjectId>,
}

enum CanvasObject {
    Stroke(Stroke),
    Text(TextObject),
    Image(ImageObject),
    Group(GroupObject),
    PdfPage(PdfPageObject),
    Measurement(MeasurementObject),
}

struct Stroke {
    points: Vec<StrokePoint>,
    brush: BrushSettings,
    color: Color,
    opacity: f32,
}

struct StrokePoint {
    x: f64,
    y: f64,
    pressure: f32,
    tilt_x: f32,
    tilt_y: f32,
    timestamp: u64,
}
```

This is an initial illustrative model, not the final schema.

We will need stable IDs, serialization versions, object transforms, masks, grouping relationships, and brush-specific data before implementation.

# 7. Proposed development phases

I would not attempt to implement all these features at once.

The drawing engine is the most important part of the entire application. If we get its architecture wrong, every subsequent feature becomes harder.

Here is the development sequence I recommend.

## Development roadmap

1. Phase 0 — Product specification and research

   Current phase
   - Complete feature inventory.
   - Document every interaction.
   - Establish application requirements.
   - Establish file-format requirements.
   - Establish architecture.
   - Define acceptance tests.
2. Phase 1 — Rust project foundation
   - Cargo workspace.
   - Iced application shell.
   - Application state management.
   - Window management.
   - Event routing.
   - Logging and diagnostics.
   - Platform configuration.
3. Phase 2 — Infinite canvas engine
   - World coordinates.
   - Camera.
   - Zoom.
   - Pan.
   - Rotation.
   - Canvas background.
   - Grid rendering.
   - Coordinate conversion.
4. Phase 3 — Rendering engine
   - wgpu integration.
   - GPU pipeline.
   - Vector path rendering.
   - Stroke rendering.
   - Anti-aliasing.
   - Render caching.
   - Viewport culling.
   - Performance profiling.
5. Phase 4 — Drawing and brush engine
   - Pen.
   - Pencil.
   - Marker.
   - Fixed-width brush.
   - Dynamic brush.
   - Pressure support.
   - Stroke smoothing.
   - Brush settings.
   - Stroke editing.
6. Phase 5 — Editing engine
   - Selection.
   - Lasso.
   - Item picker.
   - Move.
   - Copy and paste.
   - Duplicate.
   - Delete.
   - Rotate.
   - Scale.
   - Grouping.
7. Phase 6 — Layers and document model
   - Layer creation.
   - Layer deletion.
   - Visibility.
   - Locking.
   - Reordering.
   - Opacity.
   - Automatic layering.
   - Manual layering.
   - Undo and redo.
8. Phase 7 — Advanced drawing tools
   - Nudge.
   - Slice.
   - Hard mask.
   - Soft mask.
   - Fill.
   - Text.
   - Advanced brushes.
   - Custom brushes.
9. Phase 8 — Precision drawing
   - Grid types.
   - Snapping.
   - Shape guides.
   - Shape recognition.
   - Drawing scale.
   - Measurements.
   - Perspective grids.
10. Phase 9 — Objects and asset libraries
    - Object management.
    - Object packs.
    - Custom objects.
    - Asset search.
    - Favourites.
    - Reusable assets.
11. Phase 10 — Import and export
    - PNG.
    - JPG.
    - SVG.
    - PDF.
    - PSD.
    - DXF.
    - Clipboard.
    - Drag and drop.
12. Phase 11 — Gallery and project management
    - Drawing gallery.
    - Project folders.
    - Sorting.
    - Search.
    - Native document format.
    - Autosave.
    - Recovery.
13. Phase 12 — Advanced workspace
    - Artboards.
    - Customizable panels.
    - Tool Wheel.
    - Tool Bar.
    - Keyboard shortcuts.
    - Gesture customization.
    - Workspace settings.
14. Phase 13 — Presentation and collaboration
    - Presentation mode.
    - Sharing.
    - Export presets.
    - Optional real-time collaboration.
15. Phase 14 — Optimization and release
    - Performance testing.
    - Memory management.
    - Large-document testing.
    - Crash recovery.
    - Cross-platform verification.
    - Installer and packaging.

The order is deliberate. For example, we need a reliable document and geometry model before implementing complex selection, transformations, masks, and undo/redo.

# 8. Features we should improve beyond the reference

We should not blindly reproduce every limitation.

I recommend that our independent application eventually offer these additional capabilities:

| Improvement                       | Benefit                        |
| --------------------------------- | ------------------------------ |
| Unlimited layers                  | No artificial drawing limits   |
| Better large-document performance | More complex drawings          |
| Native editable SVG import        | Better vector interoperability |
| Full keyboard customization       | Desktop productivity           |
| Plugin architecture               | Third-party extensions         |
| Scriptable tools                  | Advanced workflows             |
| Better crash recovery             | Protect user projects          |
| Local-first operation             | No mandatory account           |
| Optional cloud synchronization    | Cross-device access            |
| Advanced object organization      | Better asset management        |
| Modular export engine             | More formats over time         |
| Multiple workspaces               | Different working environments |

These are product opportunities, not claims that Concepts lacks every one of them.

# 9. Important product and licensing considerations

Concepts has a free tier, one-time purchases, and subscription-based access.

Its documented commercial model includes:

- Basic free drawing functionality.
- Essentials one-time purchase.
- Everything subscription.
- Separately purchasable brush and object packs.
- Cross-platform subscription access.

The exact entitlements and prices vary by platform and region.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://tophatch.helpshift.com\&sz=32)

Concepts Help Center+2



For our application, I recommend keeping licensing completely separate from the drawing engine.

We can eventually introduce:

- Free edition.
- Professional edition.
- Optional subscription.
- Asset marketplace.
- Custom brush marketplace.
- Optional cloud services.

However, the core document format should not depend on whether someone has a subscription.

## Clean-room implementation rules

To keep this a genuinely independent rebuild:

1. Research publicly documented behaviours and workflows.
2. Write our own functional specifications.
3. Design our own application interface.
4. Write all Rust code independently.
5. Create original brush algorithms and assets.
6. Develop our own native document format.
7. Do not extract or reuse Concepts' proprietary code.
8. Do not redistribute its commercial brush packs, object packs, logos, or artwork.
9. Implement interoperability using independently developed format readers and writers.
10. Maintain a record of requirements, design decisions, implementation and testing.

We can use publicly documented feature descriptions to understand what the application does without copying its protected implementation.

# 10. What we still need to investigate before coding

The official website and manual provide substantial information, but they do not reveal every implementation detail.

I would divide the remaining investigation into these areas.

| Research area | Information still required                                |
| ------------- | --------------------------------------------------------- |
| Canvas        | Exact zoom behaviour, coordinate precision, camera limits |
| Rendering     | Stroke tessellation, caching, anti-aliasing strategy      |
| Brushes       | Individual brush behaviour and rendering characteristics  |
| Stylus        | Input event handling, pressure curves, latency            |
| Selection     | Exact hit-testing and transformation behaviour            |
| Nudge         | Vector control-point manipulation                         |
| Slice         | Intersection and vector subdivision algorithms            |
| Masks         | Mask geometry and compositing behaviour                   |
| Layers        | Exact automatic layer assignment rules                    |
| Precision     | Snapping priority and geometric tolerances                |
| Measurement   | Scale conversion and rounding rules                       |
| Shapes        | Recognition and curve-fitting behaviour                   |
| PDF           | Page representation and annotation persistence            |
| Native format | Document serialization and compatibility                  |
| Export        | Vector fidelity, rasterization and font handling          |
| Autosave      | Save frequency and recovery behaviour                     |
| Performance   | Large-document memory and rendering behaviour             |
| Shortcuts     | Complete keyboard shortcut reference                      |
| UI            | Panel states, menus and interaction details               |

Some of these details may require hands-on inspection of the application. They cannot reliably be established from the marketing website alone.

## 11. Research sources

These are the primary sources I recommend keeping as our reference library.

[image](https://www.google.com/s2/favicons?domain=https://concepts.app\&sz=32)

Official Concepts documentation

- [Full feature list](https://concepts.app/en/features?utm_source=chatgpt.com)
- [Pro feature comparison](https://concepts.app/en/features-pro?utm_source=chatgpt.com)
- [Complete user manual](https://concepts.app/en/manual?utm_source=chatgpt.com)
- [Brushes and drawing tools](https://concepts.app/en/manual/brushes-and-tools?utm_source=chatgpt.com)
- [Selection and editing](https://concepts.app/en/manual/selection?utm_source=chatgpt.com)
- [Layer management](https://concepts.app/en/manual/layers?utm_source=chatgpt.com)
- [Precision tools](https://concepts.app/en/manual/precision-tools?utm_source=chatgpt.com)
- [Object library](https://concepts.app/en/manual/library?utm_source=chatgpt.com)
- [Importing](https://concepts.app/en/manual/import?utm_source=chatgpt.com)
- [Exporting](https://concepts.app/en/manual/export?utm_source=chatgpt.com)
- [PDF editing](https://concepts.app/en/manual/pdf?utm_source=chatgpt.com)
- [Settings](https://concepts.app/en/manual/settings?utm_source=chatgpt.com)

  [image](https://www.google.com/s2/favicons?domain=https://apps.apple.com\&sz=32)

  Additional product reference
- [Concepts App Store listing](https://apps.apple.com/us/app/concepts/id560586497?utm_source=chatgpt.com)
- [Pricing explanation](https://concepts.app/en/pricing-explained/?utm_source=chatgpt.com)
- [Product media and screenshots](https://concepts.app/en/media?utm_source=chatgpt.com)

# 12. My recommendations for the actual rebuild

There are three decisions I would establish before we start writing code.

First: desktop-first, but not desktop-only architecture.

We should optimize the first version for Windows and Linux desktop usage, while keeping input and rendering systems adaptable to tablets and styluses.

Second: vector-first, not raster-first.

The document must preserve editable strokes, individual objects, groups, masks, layers, and transformations.

Third: independent engine, not an Iced-dependent drawing implementation.

Iced handles the UI. Rust handles the drawing engine. wgpu handles GPU rendering.

This separation will make the application much easier to maintain and extend.

## 13. Initial project completion checklist

Here is the status of our preparation.

Research preparation

11 / 16 completed

Show pending work only

Identify the original application's purpose

Research its main product capabilities

Identify the major functional modules

Document drawing and brush functionality

Document editing and transformation functionality

Document layers and precision tools

Document object libraries

Document import and export formats

Identify platform differences

Propose Rust + Iced architecture

Define development phases

Inspect actual desktop application interactions

Establish detailed technical specifications

Define native document format

Define rendering and geometry algorithms

Define phase acceptance tests

The 68.75% is completion of our initial research-preparation checklist, not completion of the actual product research or development.

# 14. What I want us to do next

I recommend that we do not jump into Phase 1 yet.

We should complete Phase 0: Product Specification first.

That phase should produce five substantial engineering documents:

| Document                          | Contents                                     |
| --------------------------------- | -------------------------------------------- |
| 01 — Product Requirements         | Every feature, behaviour and expected result |
| 02 — UI/UX Specification          | Every screen, panel, menu and interaction    |
| 03 — Technical Architecture       | Rust crates, dependencies and communication  |
| 04 — Drawing Engine Specification | Geometry, rendering, brushes and input       |
| 05 — Testing Specification        | Functional, performance and acceptance tests |

Each feature should have:

- Unique feature ID.
- Description.
- User interaction.
- Expected behaviour.
- Technical implementation.
- Dependencies.
- Edge cases.
- Acceptance criteria.
- Development phase.

This will allow us to build phase by phase without losing requirements or making architectural decisions that need to be reversed later.

## One important question before we proceed

Which version should we use as our primary reference?

1\. Primary reference platform

Windows desktop version

iPad / iOS version

Android version

All versions, combining the best features

2\. Rebuild scope

Complete functional rebuild, including advanced tools

Core drawing application first, advanced tools later

Complete rebuild with additional improvements beyond Concepts

Confirm project direction

My recommendation is all versions as references, complete functionality, desktop-first implementation. That gives us the broadest feature coverage without inheriting unnecessary platform restrictions.

We can then turn this research into a proper engineering specification and begin implementation one phase at a time.

I can also keep monitoring the official documentation for newly documented features and changes as we work through the rebuild.