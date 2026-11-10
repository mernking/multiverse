# 11. Command-Line Interface Specification

**Status:** Draft 0.1

## 11.1 Goals

Provide scriptable access to the same application operations used by GUI, MCP, and AI. The CLI must work headlessly for document inspection, validation, conversion, export, and batch tasks.

Temporary binary name: `creative` (to be renamed after product naming).

## 11.2 Command groups

```text
creative
  init <path>
  project list|info|open
  document info|inspect|validate|repair|convert
  canvas info|bounds
  object list|inspect|create|update|delete|transform
  layer list|create|update|delete|reorder
  selection inspect|export
  import <input>
  export <document> --format <format>
  agent chat|ask|brainstorm|plan|execute
  mcp serve
  config show|set
  version
```

Exact syntax is provisional and will be tested for ergonomic consistency.

## 11.3 Output modes

- Human-readable terminal output by default.
- `--json` for stable machine-readable output.
- `--quiet` and `--verbose`.
- Progress to stderr; structured result to stdout.
- Never mix diagnostic logs into JSON stdout.

## 11.4 Exit codes

Suggested:
- 0 success;
- 1 general failure;
- 2 invalid arguments;
- 3 document validation failure;
- 4 unsupported format/operation;
- 5 permission denied;
- 6 I/O/storage failure;
- 7 provider/network failure;
- 8 cancellation/interruption.

Document exact mapping in implementation and tests.

## 11.5 Safety

- Read-only commands do not mutate files.
- Mutating commands require explicit target document and operation.
- Destructive operations support dry-run where meaningful.
- Overwrite requires explicit `--overwrite`.
- AI execution requires permission mode and can require interactive approval; non-interactive use defaults to preview-only unless explicit authorization is supplied.
- Secrets must not be accepted as command-line arguments because process listings/history may expose them. Use secure config, environment variables with warnings, or stdin prompt.

## 11.6 Batch and scripting

- Support multiple input files.
- Stable JSON schema and version.
- Deterministic output filenames.
- Atomic output writes.
- Per-file error reporting with non-zero aggregate exit code.

## 11.7 Configuration

Store user config in platform-standard directories. Separate provider credentials from ordinary config. Allow per-project settings only for non-secret values unless explicitly protected.

## 11.8 Acceptance

- CLI can validate and export a document without launching GUI.
- JSON output conforms to a versioned schema.
- Failed batch item does not corrupt other inputs.
- `--dry-run` makes no persistent changes.
- Help output documents examples and exit codes.
