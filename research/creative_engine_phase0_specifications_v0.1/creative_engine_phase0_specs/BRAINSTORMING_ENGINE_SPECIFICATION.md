# 14. Brainstorming and Visual Thinking Specification

**Status:** Draft 0.1

## 14.1 Purpose

Convert a topic, question, selected canvas content, or rough notes into structured ideas and editable visual artifacts.

## 14.2 Supported outputs

- Mind map.
- Concept map.
- Flowchart.
- Decision tree.
- Timeline.
- Affinity clusters.
- Pros/cons matrix.
- SWOT-style matrix.
- User journey or process map.
- Idea cards and sticky notes.
- Freeform brainstorm clusters.

## 14.3 Workflow

1. User supplies topic or selects source content.
2. Agent asks a clarifying question only when needed.
3. Generate candidate ideas and relationships.
4. Optionally classify by theme, priority, effort, risk, or dependency.
5. Present textual outline and visual preview.
6. User edits the outline or layout.
7. Commit as a new page/layer/group, preserving existing work.

## 14.4 Data model

Idea node:
- stable ID;
- title;
- description;
- category/tags;
- source/provenance;
- priority/confidence if requested;
- position and dimensions;
- style;
- links to related nodes.

Relationship:
- source node;
- target node;
- relationship type;
- label;
- directionality;
- style.

Do not present model-generated confidence or factual claims as verified evidence.

## 14.5 Layout engine

Initial algorithms:
- radial mind map;
- tree layout;
- layered directed graph;
- force-directed graph for exploratory clusters;
- grid/card layout;
- swimlane/timeline layout.

Layout must be deterministic for a given graph and settings where practical. User can move objects after generation. Avoid automatic reflow that unexpectedly overwrites manual positioning.

## 14.6 Editing and provenance

- Generated objects are normal editable document objects.
- Optional provenance metadata records generation task and timestamp, not hidden chain-of-thought.
- Regeneration should create a new version or preview, not overwrite manual edits.
- User can selectively accept/reject nodes and links.

## 14.7 Quality controls

- Avoid duplicate ideas.
- Preserve user-provided wording when requested.
- Label assumptions and open questions.
- Distinguish brainstorm suggestions from sourced factual claims.
- Allow constraints such as audience, budget, timeframe, tone, and scope.

## 14.8 Acceptance

- Generate a mind map from a topic and create editable nodes/edges.
- Generated output does not overlap excessively at default zoom.
- User can reposition and edit nodes.
- Existing document content remains unchanged unless user approves insertion.
- Partial acceptance is supported.
