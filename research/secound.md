That changes the architecture quite significantly. We're no longer building just a Concepts alternative. We're building a cross-platform vector design and intelligent creative workspace, with a shared engine that can be operated through a graphical interface, command line, MCP, and an integrated AI agent.

I recommend we make this a first-class architectural requirement from the beginning, not something we add after the desktop application is finished.

We'll use Linux as our primary development and testing environment, but Linux must not become a platform dependency.

# 1. Updated project vision

# Concepts-inspired Creative Engine

Complete functional rebuild

An independent, cross-platform vector drawing, technical design, brainstorming and AI-assisted creative application.

Desktop

Linux, Windows, macOS

Mobile

Android, iOS

CLI

Automation and scripting

MCP Server

External AI integrations

Native Built-in AI Agent

OpenAI API · Anthropic API · Tool execution · Brainstorming

Shared Rust Core

One document model · One geometry engine · One command system · One plugin interface

The most important architectural principle:

The desktop application, mobile application, CLI, MCP server and AI agent must all operate on the same underlying creative engine.

We should never build five separate implementations of the same drawing functionality.

# 2. Cross-platform architecture

There is one important technical consideration before we commit to the UI architecture.

Iced officially documents Linux, Windows, macOS and Web support, but does not currently list Android and iOS as supported platforms in its standard platform matrix.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



`wgpu`, on the other hand, supports Vulkan on Android and Metal on iOS, with documented mobile integration options.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



My recommendation:

- Linux, Windows and macOS: Iced desktop application.
- Android: native mobile application shell with shared Rust core.
- iOS: native mobile application shell with shared Rust core.
- Web: optional future WebAssembly application.
- CLI and MCP: pure Rust services without GUI dependencies.

We can investigate whether Iced can serve as the mobile interface too, but we should not make the entire project dependent on unverified mobile support.

The drawing engine, document model, brushes, geometry, AI tools and commands must remain shared.

## 2.1 Updated workspace

```
creative-engine/
│
├── Cargo.toml
│
├── apps/
│   ├── desktop/
│   │   └── Iced desktop application
│   │
│   ├── cli/
│   │   └── Command-line interface
│   │
│   ├── mcp/
│   │   └── MCP server
│   │
│   ├── android/
│   │   └── Android application shell
│   │
│   ├── ios/
│   │   └── iOS application shell
│   │
│   └── web/
│       └── Future WebAssembly application
│
├── crates/
│   ├── core/
│   ├── application/
│   ├── commands/
│   ├── documents/
│   ├── canvas/
│   ├── geometry/
│   ├── renderer/
│   ├── brushes/
│   ├── input/
│   ├── selection/
│   ├── layers/
│   ├── precision/
│   ├── objects/
│   ├── history/
│   ├── importers/
│   ├── exporters/
│   ├── storage/
│   │
│   ├── agent/
│   ├── ai-providers/
│   ├── ai-tools/
│   ├── brainstorming/
│   ├── context/
│   │
│   ├── cli-engine/
│   ├── mcp-protocol/
│   ├── plugin-system/
│   └── platform/
│
├── resources/
│   ├── brushes/
│   ├── objects/
│   ├── icons/
│   ├── fonts/
│   └── textures/
│
└── tests/
```

The crucial dependency rule:

```
          Desktop UI
              |
Android UI ---+--- iOS UI
              |
           CLI / MCP
              |
       Application API
              |
        Shared Core
              |
       Document Engine
              |
       Geometry Engine
              |
       Rendering Engine
```

CLI and MCP should not implement drawing operations independently. They should invoke the same application commands that the graphical interface uses.

# 3. Native built-in AI agent

Major product capability

I want to give this its own major architectural module.

The AI agent shouldn't simply be a chatbot sitting beside the drawing application.

It should understand the actual document and be able to interact with the creative environment.

[Nodalist Ai: Your AI Thinking Partner | Orynth](https://images.openai.com/static-rsc-4/Ufs9Nm7oNAygyHJK-ryBc4ClgsMuDFkz99F-tWMGewl3wvGFJxqmFpeAoJHVUDTbyQEhYf3AKqOZqcform2s_1v_T9BYgpZ4sKRG5iT-VY0z0DV15y5JyyHL_m2NPEFkn8JUIbZF-Yy7q57t7fynB8om7R15FIA6HywIRFZ1n0k?purpose=inline)

[AI Concept Map Generator — Free Online | Cloudairy](https://images.openai.com/static-rsc-4/oyEywjHUC3Dq9ZS0a7eywDdXxUgYUk2c1kMRKMzLyq7aQveF5IFzvfBcf0On2Q5O6mkF_8z7slvfBwqmXu361ibWi3MgMVgH5q-ZgtA-GUhCLoTG2MqjLNgGmOXS8WIpxBSEUj-ZGxYB-viTuSEN2szFfPqca6H49hTvL2iYMN0?purpose=inline)

[Infinite Canvas app gallery](https://images.openai.com/static-rsc-4/S0IvC62-uf_gD2SOrH19o7HcnzW6-i1PlkO2mYH5e1hmre5J2Z3OkKt7CbiLJUQPmX0-Q1WjjVgE12pKJ22XLEWX0bjsnFmsiAeTI6XO6oqW3x9_5Cl81TO9klEKdhRhqf6Nnfw_wZy6cZUkr81ERj9E7bBS_soRXvOGheZ30j0?purpose=inline)

14

## 3.1 AI provider architecture

We should create a provider-independent interface.

Initially support:

- OpenAI API.
- Anthropic API.

The architecture should allow additional providers later.

```
              Native AI Agent
                     |
              Agent Controller
                     |
             Provider Interface
                 /       \
                /         \
          OpenAI API   Anthropic API
                \         /
                 \       /
                Response Adapter
                     |
                Tool Router
                     |
              Application Core
                     |
           Canvas / Documents / Tools
```

Important requirements:

- API key management.
- Provider configuration.
- Model selection.
- Conversation history.
- Streaming responses.
- Tool calling.
- Context management.
- Error handling.
- Usage tracking.
- Request cancellation.
- Configurable permissions.
- Local conversation storage.

API credentials should be stored securely using the operating system's credential facilities, not inside project files.

## 3.2 What the AI agent should actually do

### A. Brainstorming

A user should be able to say:

"Help me brainstorm a new electric vehicle design."

The agent should be capable of:

1. Understanding the subject.
2. Asking useful follow-up questions.
3. Generating different concepts.
4. Organizing ideas into groups.
5. Creating mind maps.
6. Connecting related ideas.
7. Identifying contradictions.
8. Suggesting improvements.
9. Summarizing the results.
10. Turning selected ideas into a visual design.

### B. Direct canvas manipulation

The agent should be able to:

- Create text objects.
- Create shapes.
- Draw lines.
- Create diagrams.
- Create mind maps.
- Add arrows.
- Group objects.
- Reorganize layouts.
- Change colours.
- Move objects.
- Create layers.
- Annotate designs.
- Generate structured visual plans.

For example:

"Create a mind map showing the architecture of an electric vehicle, including its battery, motor, controller, charging system and thermal management."

The AI should actually create the mind map on the canvas, not just describe how to draw one.

### C. Document understanding

The agent should understand:

- Current document.
- Active layer.
- Selected objects.
- Existing text.
- Canvas dimensions.
- Object relationships.
- Current viewport.
- Existing diagrams.
- Project metadata.

### D. Creative assistance

Other potential operations:

- Explain a drawing.
- Improve a design.
- Identify missing components.
- Generate alternative layouts.
- Convert brainstorming into diagrams.
- Turn sketches into structured specifications.
- Create presentation outlines.
- Generate technical annotations.

### E. Controlled execution

I recommend three interaction modes:

| Mode    | Behaviour                           |
| ------- | ----------------------------------- |
| Suggest | Agent proposes changes              |
| Review  | Agent prepares changes for approval |
| Execute | Agent performs authorized actions   |

Destructive operations should require explicit authorization.

## 3.3 Agent architecture modules

We should not place everything inside one AI crate.

| Module               | Responsibility                          |
| -------------------- | --------------------------------------- |
| Agent Core           | Agent lifecycle and orchestration       |
| Provider System      | OpenAI and Anthropic                    |
| Conversation Engine  | Chat history and messages               |
| Context Engine       | Document and workspace awareness        |
| Tool Registry        | Available operations                    |
| Tool Executor        | Execute application commands            |
| Brainstorming Engine | Ideas, mind maps and concept generation |
| Planning Engine      | Multi-step creative tasks               |
| Permission System    | User approval and restrictions          |
| Memory System        | Project-specific knowledge              |
| Streaming Engine     | Live responses                          |
| Audit System         | Track AI-generated changes              |

The AI should never need direct access to internal drawing data structures. It should interact through a stable application command interface.

This will make the AI safer and allow us to change providers without rewriting the drawing engine.

# 4. CLI architecture

The CLI should be a complete interface to the application engine, not simply a launcher.

Example command structure:

```
creative init my-project

creative project list

creative project open my-project

creative canvas info

creative document inspect drawing.cdoc

creative document validate drawing.cdoc

creative object list

creative object create shape

creative layer list

creative export drawing.cdoc --format svg

creative agent chat

creative agent brainstorm "Electric vehicle design"

creative agent execute plan.json

creative mcp start
```

These are proposed commands, not existing implementations.

CLI capabilities should include:

- Project creation.
- Document inspection.
- Document validation.
- Object management.
- Layer management.
- Import.
- Export.
- Batch processing.
- Rendering.
- AI brainstorming.
- AI task execution.
- MCP server management.
- Configuration management.

We should also support headless execution, so users can perform document operations without opening the graphical application.

# 5. MCP architecture

MCP should expose the creative engine to external AI clients.

For example, another AI assistant could connect to our MCP server and request:

- Create a new drawing.
- Inspect a document.
- Add shapes.
- Create a mind map.
- Organize layers.
- Export an illustration.
- Summarize a drawing.
- Generate a design concept.

We should expose distinct tool categories:

| Category      | Example MCP tools                      |
| ------------- | -------------------------------------- |
| Projects      | `project_create`, `project_list`       |
| Documents     | `document_open`, `document_inspect`    |
| Canvas        | `canvas_get_info`, `canvas_set_view`   |
| Drawing       | `stroke_create`, `shape_create`        |
| Objects       | `object_add`, `object_transform`       |
| Layers        | `layer_create`, `layer_move`           |
| Selection     | `selection_get`, `selection_transform` |
| Precision     | `grid_set`, `measurement_create`       |
| Brainstorming | `brainstorm`, `mindmap_create`         |
| AI            | `agent_run`, `agent_status`            |
| Export        | `document_export`, `selection_export`  |

MCP should use the same application API as the desktop UI and CLI.

That means an operation implemented once can become available to all three.

# 6. Updated development roadmap

The original 14 phases need to expand to accommodate the additional products and services.

Here is the revised sequence.

| Phase | Deliverable                                                       |
| ----- | ----------------------------------------------------------------- |
| 0     | Complete product research and specifications                      |
| 1     | Rust workspace, shared core, command API and platform abstraction |
| 2     | Iced desktop foundation and application shell                     |
| 3     | Canvas, coordinate system and camera                              |
| 4     | GPU rendering engine                                              |
| 5     | Input and stylus engine                                           |
| 6     | Vector drawing and brush engine                                   |
| 7     | Document model, saving and recovery                               |
| 8     | Layers, history and editing                                       |
| 9     | Precision drawing and measurements                                |
| 10    | Objects, libraries and assets                                     |
| 11    | Import/export and PDF                                             |
| 12    | Advanced desktop interface                                        |
| 13    | CLI implementation                                                |
| 14    | MCP server implementation                                         |
| 15    | AI provider integration                                           |
| 16    | Native AI agent and brainstorming                                 |
| 17    | Android application                                               |
| 18    | iOS application                                                   |
| 19    | Plugin and extension ecosystem                                    |
| 20    | Cross-platform testing and optimization                           |
| 21    | Packaging, distribution and release                               |

Important: We should establish the CLI, MCP and AI command interfaces during Phase 1, even though their full implementations come later. This prevents architectural rewrites.

We should also create an Android rendering and input proof-of-concept early, rather than waiting until Phase 17 to discover mobile integration problems.

## 7. Additional engineering requirements

These are requirements that become particularly important because we are targeting multiple platforms.

- Platform-independent document format.
- Platform-independent geometry.
- Platform-independent drawing commands.
- Touch and stylus abstraction.
- Hardware and software rendering fallback.
- Responsive desktop and mobile layouts.
- Cross-platform file access.
- Secure AI credential management.
- Offline drawing functionality.
- Local AI conversation storage.
- Provider-independent AI interface.
- Stable MCP tool definitions.
- Versioned CLI interface.
- Versioned native document format.
- Automated cross-platform tests.
- Plugin compatibility system.
- AI operation history and undo support.

## 8. How this relates to your earlier engineering architecture

I also found our earlier architecture work for the Engineer project. We had established a Desktop, CLI and MCP arrangement around a shared Rust core, with Iced and wgpu in the desktop stack. We can reuse that architectural principle here, while designing the creative and drawing subsystems independently.&#x20;

That is exactly the kind of separation we should maintain across these projects.

# 9. Updated Phase 0 deliverables

Before implementation, I want us to produce a complete technical specification with these additional documents:

1. `PRODUCT_REQUIREMENTS.md`
2. `FEATURE_INVENTORY.md`
3. `UI_UX_SPECIFICATION.md`
4. `RUST_WORKSPACE_ARCHITECTURE.md`
5. `CANVAS_ENGINE_SPECIFICATION.md`
6. `VECTOR_GEOMETRY_SPECIFICATION.md`
7. `RENDERING_ENGINE_SPECIFICATION.md`
8. `BRUSH_ENGINE_SPECIFICATION.md`
9. `DOCUMENT_FORMAT_SPECIFICATION.md`
10. `CROSS_PLATFORM_ARCHITECTURE.md`
11. `CLI_SPECIFICATION.md`
12. `MCP_SPECIFICATION.md`
13. `AI_AGENT_SPECIFICATION.md`
14. `BRAINSTORMING_ENGINE_SPECIFICATION.md`
15. `IMPORT_EXPORT_SPECIFICATION.md`
16. `TESTING_AND_ACCEPTANCE_CRITERIA.md`
17. `DEVELOPMENT_ROADMAP.md`

Every document should contain actual implementation requirements, not just descriptions.

My proposed rule for this project: no feature enters development without a documented specification, dependencies and acceptance criteria.

This will help us avoid building an attractive application that has an incomplete underlying engine.

The next logical task is to develop the complete Phase 0 specification, beginning with the document model, shared application API and platform architecture, since these three decisions affect practically everything else.

We can also track changes to the original Concepts feature set while the specification is being developed.