# 12. Model Context Protocol (MCP) Specification

**Status:** Draft 0.1

## 12.1 Purpose

Expose the application as a controlled creative tool to MCP-compatible AI clients. MCP is an interface to the application command API, not a second document engine.

## 12.2 Transport and security

- Local stdio transport is the default.
- HTTP/remote transport is a later opt-in capability.
- Remote access requires authentication, explicit binding configuration, and transport security.
- Default to least privilege.
- Validate every argument against a schema.
- Apply document permissions and confirmation policy.
- Do not expose arbitrary filesystem access.

## 12.3 Tool categories

Read tools:
- `document_list`
- `document_inspect`
- `canvas_get_info`
- `selection_get`
- `layer_list`
- `object_list`
- `object_inspect`
- `asset_search`
- `document_validate`

Write tools:
- `document_create`
- `object_create`
- `object_update`
- `object_transform`
- `object_delete`
- `layer_create`
- `layer_update`
- `layer_reorder`
- `group_create`
- `precision_set`
- `mindmap_create`
- `diagram_create`
- `document_export`

Task tools:
- `brainstorm`
- `agent_plan`
- `agent_execute`
- `task_status`
- `task_cancel`

Final names are subject to MCP naming and compatibility review.

## 12.4 Resources

Potential resources:
- active document summary;
- selected objects;
- layer tree;
- document metadata;
- asset catalog;
- supported format list;
- application capability manifest.

Resources should be scoped and size-limited. Large content is returned through paginated or targeted queries.

## 12.5 Tool schemas

Every tool defines:
- stable name and version;
- description and intended use;
- typed input schema;
- typed result schema;
- permission level (read, reversible write, destructive write, external action);
- confirmation requirement;
- timeout and cancellation;
- maximum payload size;
- audit metadata.

## 12.6 Transactions

A multi-step write tool should execute as one named transaction when possible. If a sequence partially fails, return completed steps and rollback status. Never claim rollback if it did not occur.

## 12.7 Prompt injection and untrusted content

Document text, imported SVG/PDF metadata, comments, and external MCP responses are untrusted data, not instructions. The agent must not treat embedded text as permission to reveal secrets or execute unrelated actions.

## 12.8 Versioning

Expose protocol/server version and tool schema version. Removing or changing a tool requires deprecation policy and compatibility tests.

## 12.9 Acceptance

- MCP client can inspect a document without write permission.
- Write calls are rejected when permission is absent.
- Invalid schemas and oversized payloads are safely rejected.
- Mutations are undoable and audited.
- Local server does not listen on a network interface unless explicitly configured.
