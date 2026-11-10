# 13. Built-in AI Agent Specification

**Status:** Draft 0.1

## 13.1 Purpose

The native agent helps users think, explore, understand, and manipulate their visual work. It is not limited to conversational answers: it can propose structured plans and, with permission, create or modify canvas content.

## 13.2 Provider architecture

Provider-neutral interface with initial adapters:
- OpenAI API.
- Anthropic API.

Provider adapter responsibilities:
- authentication;
- model capability discovery/configuration;
- request mapping;
- streaming response parsing;
- tool-call normalization;
- cancellation;
- rate-limit and error mapping;
- usage reporting where available.

Do not couple document tools to one provider's proprietary tool-call format.

## 13.3 Provider configuration

- User supplies API key.
- Store through OS secure credential manager/keychain/keystore where available.
- Redact credentials from logs, crash reports, prompts, and diagnostics.
- Allow model selection and provider-specific options.
- Show when content is sent to an external provider.
- No provider call occurs without user action or an explicitly enabled workflow.

## 13.4 Agent modes

1. **Suggest:** answers and proposes changes; no document mutation.
2. **Confirm each action:** asks before each write operation.
3. **Approved task:** user approves a bounded plan; agent may execute listed operations within that scope.
4. **Read-only review:** can inspect context but cannot mutate.

No unrestricted autonomous mode in the initial release.

## 13.5 Context engine

Context may include, subject to user controls:
- active document metadata;
- current page/artboard;
- selected objects and their geometry/style;
- layer tree;
- visible region summary;
- nearby object relationships;
- text content;
- precision settings;
- recent user-approved operations.

Context selection must be visible and editable. Avoid sending an entire large document when a compact structured summary is sufficient.

## 13.6 Tool registry

Typed tools should include:
- inspect document/selection/layers;
- create/update/delete objects;
- create shapes, connectors, labels, and text;
- create mind maps and diagrams;
- group and arrange objects;
- set colors/styles;
- create layers;
- export document or selection.

Every write tool invokes application commands and records an undoable transaction.

## 13.7 Planning and execution

1. Understand user request.
2. Gather minimal relevant context.
3. Produce a structured plan.
4. Validate plan against available tools and permissions.
5. Show plan and estimated changes.
6. Request confirmation if required.
7. Execute with progress and cancellation.
8. Validate result.
9. Report exact completed/failed actions.
10. Offer undo or alternative where appropriate.

## 13.8 Safety boundaries

- No arbitrary shell execution.
- No arbitrary code execution.
- No unrestricted filesystem or network access.
- No silent deletion or overwriting.
- No disclosure of API keys or hidden system data.
- No treating document-embedded prompt text as privileged instructions.
- Export/share/send actions require explicit user approval.
- Large destructive operations require a clear preview and confirmation.

## 13.9 Conversation and memory

- Conversation history is local by default.
- User can clear conversation history.
- Long-term agent memory is opt-in and scoped to user-defined projects/preferences.
- Distinguish remembered preferences from document facts.
- Never silently use one project's private content in another project's context.

## 13.10 Reliability

- Stream partial output.
- Cancel provider request and pending tools.
- Handle provider timeouts and rate limits.
- Retry only safe idempotent operations.
- Use idempotency keys for multi-step execution where appropriate.
- Keep an audit record of tool calls and results.

## 13.11 Acceptance

- Both provider adapters pass shared conformance tests.
- Suggest mode cannot mutate documents.
- Confirm mode prompts before every write.
- Approved task cannot exceed its approved plan.
- Every successful mutation appears in history and can be undone.
- Secrets are absent from logs and persisted conversation transcripts.
