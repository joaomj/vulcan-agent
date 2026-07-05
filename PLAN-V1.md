# Vulcan Agent Implementation Plan V1

This plan implements `docs/PRD.md`. The PRD is the product source of truth. If this plan conflicts with `docs/PRD.md`, update this plan.

Each step below is intentionally atomic for assignment to a lower-intelligence implementation agent. Do exactly one microstep at a time. Do not combine adjacent steps unless the user explicitly approves it.

## Ground Rules

- Build a pure Rust single-binary coding agent. Do not introduce Node, Bun, Python, Electron, or a frontend framework.
- Keep the web UI thin. The server is the source of truth.
- Use WebSocket for primary client communication. Use HTTP only for health checks and static assets.
- Persist every mutation as an append-only SQLite event. Project session state from events.
- Create one git worktree per session under the configured data directory.
- Use shell `git` through `tokio::process::Command`. Do not use `git2`.
- Keep recall read-only and tool-driven. Never inject recall by default.
- Discover skill name and description by default. Load full skill content only via the `skill` tool.
- Keep all tool inputs and outputs typed. No tool returns a raw `serde_json::Value` as its final API boundary.
- Make file mutations idempotent or fail safely with typed errors.
- Source all timeouts, limits, allowlists, provider settings, data directories, and log directories from `config.toml` defaults or overrides.
- Treat `/new`, `/sessions`, `/resume`, `/undo`, `/model`, `/permissions`, `/status`, `/clear`, and `/recall` as deterministic server actions that bypass the LLM.
- Defer MCP, subagents, Telegram/WhatsApp/Signal gateways, LSP/code intelligence, agent-curated memory, session forks, JSONL storage, desktop apps, IDE extensions, and Anthropic Messages protocol.

## MVP Definition

MVP is complete when these PRD success criteria pass:

1. A session can start from the web UI, receive project context, and answer or code using `read`, `write`, `edit`, `apply_patch`, `glob`, `grep`, and non-interactive `bash`.
2. The web UI works on desktop and mobile browsers through LAN, Tailscale, or SSH tunnel.
3. `/undo` reverts the last assistant turn's file changes by resetting the session worktree to the pre-turn snapshot.
4. Past sessions are searchable and replayable via `/recall` and trace tools.
5. Vulcan is reliable enough to dogfood on a real greenfield project.

## Implementation Status

Completed on current branch:

- Phase 0 baseline inspection was performed.
- `docs/PRD.md` exists and remains the product source of truth.
- Root workspace now contains only PRD MVP crates: `core`, `storage`, `llm`, `providers`, `tools`, `skills`, `guardrails`, `server`, `web`, and `cli`.
- Standalone `crates/recall` was removed from the workspace and its placeholder code was folded into `crates/storage` as `recall_error` and `recall_search`.
- Core session modes are restricted to `Ask`, `Plan`, and `Build`.
- Core approval mode vocabulary is `Never`, `OnMode`, and `Always`.
- Tool approval class vocabulary is `Never`, `OnMode`, and `Always`.
- Server config includes `[data_dir]` and `[logs]` defaults.
- Strict validation passed: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `cargo check --workspace`.
- Phase 1 durable session core implemented: typed events (`SessionMode`/`MessageRole`/`ContentBlock`), `version` field on events, `project_path`/`model` on `Session`, `WorktreeMetadata` type.
- Storage migrations added for version column, tool_calls, provider_calls, approvals, snapshots, and recall FTS5 tables.
- `EventStore::append` wrapped in SQLite transaction for atomicity.

Known workspace note:

- The repository also contains non-product docs such as `docs/inspiration.md` and `docs/architecture.html`. Treat `docs/PRD.md` as the only product source of truth.

## Execution Rules For Agents

- Start each assignment by reading this plan section and the files named in the microstep.
- Implement only the named minimal change.
- Add or update tests in the same microstep when the verification requires them.
- Run the exact verification listed for the microstep before reporting completion.
- If the verification fails because of unrelated pre-existing errors, report the exact failure and stop.
- Do not edit `docs/PRD.md` unless the user explicitly asks.
- Do not add dependencies unless a microstep explicitly says to add them.
- Do not add deferred features early.

## Cross-Phase Validation

Run before declaring a phase complete:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo check --workspace
```

## Phase 0: Baseline And PRD Alignment

Status: complete.

### Step 0.1: Confirm Baseline State [DONE]

Minimal change: Inspect root files, crate list, docs, current git status, and `cargo check --workspace`.

Defers: No code changes.

Verifies: `git status --short` and `cargo check --workspace` were run.

### Step 0.2: Reconcile Workspace Layout [DONE]

Minimal change: Remove standalone `crates/recall` from the workspace and expose placeholder recall modules from `crates/storage`.

Defers: Real FTS5 recall implementation.

Verifies: Root `Cargo.toml` has only PRD MVP crates and `cargo check --workspace` passed.

### Step 0.3: Align Core Domain Vocabulary [DONE]

Minimal change: Restrict session modes and approval vocabulary to PRD terms.

Defers: Full approval gate behavior.

Verifies: `cargo test -p vulcan-core` and `cargo test -p vulcan-tools` passed.

### Step 0.4: Add Config Surface Placeholders [DONE]

Minimal change: Add `data_dir` and `logs` config sections with defaults.

Defers: Full `config.toml` file loading and validation.

Verifies: `cargo test -p vulcan-server` passed as part of workspace tests.

## Phase 1: Durable Session Core

### Step 1.1: Add Missing Typed IDs [DONE]

WorktreeId already existed. No additional IDs needed.

Verifies: `cargo test -p vulcan-core id::tests` passed.

### Step 1.2: Add Project Path And Model To Session [DONE]

Minimal change: Added `project_path: Option<String>` and `model: String` to `Session`.

Defers: Worktree metadata and event projection.

Verifies: `cargo test -p vulcan-core session::tests` passed.

### Step 1.3: Add Worktree Metadata Type [DONE]

Minimal change: Added `WorktreeMetadata` struct with `source_project_path`, `worktree_path`, `head_revision`, and `dirty_summary`.

Defers: Git integration and snapshot events.

Verifies: `cargo test -p vulcan-core session::tests` passed.

### Step 1.4: Replace String SessionCreated Mode With Typed Mode [DONE]

Minimal change: Changed `EventKind::SessionCreated` to store `SessionMode` instead of `String` and updated all callers.

Defers: Other typed event payloads.

Verifies: `cargo test -p vulcan-core event::tests` passed.

### Step 1.5: Replace String Message Role With Typed Role [DONE]

Minimal change: Changed `EventKind::MessageAppended` to store `MessageRole` instead of `String` and updated projector/storage tests.

Defers: Message content persistence.

Verifies: `cargo test -p vulcan-storage projector::tests` passed.

### Step 1.6: Add Message Content To MessageAppended Event [DONE]

Minimal change: Added `content: Vec<ContentBlock>` to `EventKind::MessageAppended` and project it into `Session.messages`.

Defers: Provider/tool-specific content semantics.

Verifies: `cargo test -p vulcan-storage projector::tests` passed.

### Step 1.7: Add Tool Call Event Payload Fields [AFK]

Minimal change: Extend tool call request/result events with typed tool name, arguments JSON, status enum, output, error, and metadata fields.

Defers: Tool execution implementation.

Verifies: `cargo test -p vulcan-core event::tests`.

### Step 1.8: Add Approval Event Payload Fields [AFK]

Minimal change: Extend approval request/decision events with typed decision/status and redacted display data.

Defers: Approval gate logic.

Verifies: `cargo test -p vulcan-core event::tests`.

### Step 1.9: Add Provider Event Payload Fields [AFK]

Minimal change: Extend provider call events with model, request metadata, response metadata, usage, and error status fields.

Defers: HTTP provider implementation.

Verifies: `cargo test -p vulcan-core event::tests`.

### Step 1.10: Add Snapshot And Undo Event Variants [AFK]

Minimal change: Add event variants for snapshot recorded, undo requested, undo completed, and undo failed.

Defers: Git reset behavior.

Verifies: `cargo test -p vulcan-core event::tests`.

### Step 1.11: Add Slash Command Effect Event Variant [AFK]

Minimal change: Add a typed event variant for deterministic slash command state changes.

Defers: Slash command parser and server actions.

Verifies: `cargo test -p vulcan-core event::tests`.

### Step 1.12: Add Event Version Field [DONE]

Added `version: u64` field to `Event` (defaults to 1 in `Event::new`). Projector warns on `version > 1`.

Verifies: `cargo test -p vulcan-core event::tests` passed.

### Step 1.13: Add Event Ordering Test [DONE]

Added `test_event_ordering_by_sequence` unit test.

Verifies: `cargo test -p vulcan-core event::tests` passed.

### Step 1.14: Add Migration For Version Column [DONE]

Added `ALTER TABLE events ADD COLUMN version` migration.

Verifies: `cargo test -p vulcan-storage migration::tests` passed.

### Step 1.15: Add Migration For Tool Calls [DONE]

Added `tool_calls` table with session ID, tool call ID, name, status, and timestamps.

Verifies: `cargo test -p vulcan-storage migration::tests` passed.

### Step 1.16: Add Migration For Provider Calls [DONE]

Added `provider_calls` table with provider call ID, model, status, and timestamps.

Verifies: `cargo test -p vulcan-storage migration::tests` passed.

### Step 1.17: Add Migration For Approvals [DONE]

Added `approvals` table with approval ID, tool call ID, status, and decision.

Verifies: `cargo test -p vulcan-storage migration::tests` passed.

### Step 1.18: Add Migration For Snapshots [DONE]

Added `snapshots` table with worktree path, head revision, and dirty summary.

Verifies: `cargo test -p vulcan-storage migration::tests` passed.

### Step 1.19: Add Migration For Recall FTS Tables [DONE]

Added `recall_fts` virtual table using FTS5 with `porter unicode61` tokenizer.

Verifies: `cargo test -p vulcan-storage migration::tests` passed.

### Step 1.20: Append Event In Transaction [DONE]

Minimal change: Ensure `EventStore::append` writes the event and any immediate metadata in one SQLite transaction.

Defers: Writing all query tables.

Verifies: `cargo test -p vulcan-storage event_store::tests`.

### Step 1.21: Store Queryable Message Metadata On Append [AFK]

Minimal change: When appending `MessageAppended`, also upsert the `messages` table in the same transaction.

Defers: Tool/provider/approval metadata writes.

Verifies: Add and run one storage test for message metadata persistence.

### Step 1.22: Store Queryable Tool Metadata On Append [AFK]

Minimal change: When appending tool events, upsert the `tool_calls` table in the same transaction.

Defers: Recall indexing.

Verifies: Add and run one storage test for tool metadata persistence.

### Step 1.23: Store Queryable Provider Metadata On Append [AFK]

Minimal change: When appending provider events, upsert the `provider_calls` table in the same transaction.

Defers: Token budget enforcement.

Verifies: Add and run one storage test for provider metadata persistence.

### Step 1.24: Store Queryable Approval Metadata On Append [AFK]

Minimal change: When appending approval events, upsert the `approvals` table in the same transaction.

Defers: Approval gate behavior.

Verifies: Add and run one storage test for approval metadata persistence.

### Step 1.25: Fail Projector On Unsupported Event Version [AFK]

Minimal change: Make projector return an error or visible projection error for event versions newer than supported.

Defers: Migration support for older versions.

Verifies: Add and run one projector test for unsupported event version.

### Step 1.26: Project Current Model From Events [AFK]

Minimal change: Project current model into `Session` from session creation and model-change events.

Defers: `/model` command parser.

Verifies: Add and run one projector test for model changes.

### Step 1.27: Project Current Approval Mode From Events [AFK]

Minimal change: Project approval mode into `Session` from session creation and permissions-change events.

Defers: Full approval gate.

Verifies: Add and run one projector test for approval mode changes.

### Step 1.28: Project Clear Context State [AFK]

Minimal change: Make `/clear` effect events hide or truncate prior conversational context in projected state without deleting events.

Defers: WebSocket command handling.

Verifies: Add and run one projector test for clear behavior.

## Phase 2: Configuration, CLI, And Server Shell

### Step 2.1: Add Config File Path API [AFK]

Minimal change: Add `ServerConfig::load_from_path(path)` that reads TOML from a provided path.

Defers: CLI wiring and validation rules.

Verifies: Add and run one config test loading a minimal temp config file.

### Step 2.2: Preserve Existing Default Config API [AFK]

Minimal change: Keep `ServerConfig::load()` returning defaults when no config path is provided.

Defers: Search paths for config discovery.

Verifies: `cargo test -p vulcan-server config::tests::test_default_config`.

### Step 2.3: Validate Provider Config [AFK]

Minimal change: Reject empty provider kind, empty base URL, invalid base URL, empty model, and empty API key environment variable name.

Defers: Reading actual API key values.

Verifies: Add and run config tests for each invalid provider field.

### Step 2.4: Validate Tools Config [AFK]

Minimal change: Reject zero or out-of-range tool timeout values.

Defers: Per-tool timeout overrides.

Verifies: Add and run config tests for invalid tool timeout values.

### Step 2.5: Validate Data And Log Paths [AFK]

Minimal change: Reject empty `data_dir.root` and empty `logs.dir` values.

Defers: Directory creation.

Verifies: Add and run config tests for empty path values.

### Step 2.6: Validate Log Level [AFK]

Minimal change: Reject unknown log levels outside `trace`, `debug`, `info`, `warn`, and `error`.

Defers: Logging initialization.

Verifies: Add and run one config test for an invalid log level.

### Step 2.7: Add Provider API Key Lookup Method [AFK]

Minimal change: Add a method that reads the configured provider API key environment variable at request time.

Defers: Provider HTTP calls.

Verifies: Add and run one config test using a temporary environment variable.

### Step 2.8: Add CLI Argument Struct Tests [AFK]

Minimal change: Add tests proving `vulcan serve --project <path>` parses correctly.

Defers: Server startup behavior.

Verifies: `cargo test -p vulcan-cli`.

### Step 2.9: Validate CLI Project Path Exists [AFK]

Minimal change: Add a pure validation function that rejects missing project paths.

Defers: Git repository validation.

Verifies: Add and run one CLI test for missing project path.

### Step 2.10: Validate CLI Project Path Is Git Repository [AFK]

Minimal change: Extend CLI validation to require a `.git` directory or valid git worktree metadata.

Defers: Shell `git` command wrapper.

Verifies: Add and run one CLI test for non-git temp directory.

### Step 2.11: Add SQLite Initialization Function [AFK]

Minimal change: Add a server startup helper that opens SQLite under the configured data directory and runs migrations.

Defers: Session creation.

Verifies: Add and run one server test using a temp data directory.

### Step 2.12: Add Static Asset Route For Root [AFK]

Minimal change: Ensure `GET /` returns the web shell HTML.

Defers: CSS, JS, manifest, and service worker.

Verifies: Add and run one server integration test for `/`.

### Step 2.13: Add Static CSS And JS Routes [AFK]

Minimal change: Serve the initial CSS and JS assets over HTTP.

Defers: PWA assets.

Verifies: Add and run one server test for one CSS path and one JS path.

### Step 2.14: Initialize Structured Logging [AFK]

Minimal change: Add logging initialization from config log level without changing application behavior.

Defers: Log file routing.

Verifies: `cargo test -p vulcan-server`.

### Step 2.15: Define WebSocket Envelope Types [AFK]

Minimal change: Add typed client and server WebSocket envelope enums with serde tests.

Defers: WebSocket route.

Verifies: Add and run server protocol serde tests.

### Step 2.16: Add WebSocket Upgrade Route [AFK]

Minimal change: Add `GET /ws` upgrade route that accepts a connection and immediately sends a server hello event.

Defers: Message handling.

Verifies: Add and run one WebSocket integration test for connection and hello.

### Step 2.17: Return Typed Error For Malformed WebSocket JSON [AFK]

Minimal change: Parse incoming WebSocket text as the client envelope and return a typed protocol error on invalid JSON.

Defers: Supported commands.

Verifies: Add and run one WebSocket integration test for malformed input.

### Step 2.18: Return Typed Error For Unsupported WebSocket Command [AFK]

Minimal change: Return a typed unsupported-command error for well-formed but unimplemented commands.

Defers: Session and slash command behavior.

Verifies: Add and run one WebSocket integration test for unsupported command.

## Phase 3: Worktrees And Project Context

### Step 3.1: Add Git Command Result Type [AFK]

Minimal change: Define typed output for shell git commands: stdout, stderr, exit code, duration, and timed-out flag.

Defers: Command execution.

Verifies: `cargo test -p vulcan-server` or the crate where the type is added.

### Step 3.2: Add Async Git Command Runner Success Path [AFK]

Minimal change: Implement running one git command with `tokio::process::Command` and capturing output.

Defers: Timeout and redaction.

Verifies: Add and run one integration test for `git --version`.

### Step 3.3: Add Git Command Non-Zero Error Handling [AFK]

Minimal change: Return a typed git error when git exits non-zero.

Defers: Timeout behavior.

Verifies: Add and run one test for an invalid git subcommand.

### Step 3.4: Add Git Command Timeout Handling [AFK]

Minimal change: Enforce configured timeout around git command execution.

Defers: Redacted logging.

Verifies: Add and run one timeout test with a very small timeout.

### Step 3.5: Add Worktree Path Builder [AFK]

Minimal change: Build deterministic worktree paths under `<data_dir>/worktrees/<project>/<session-id>/`.

Defers: Creating worktrees.

Verifies: Add and run one unit test for the generated path shape.

### Step 3.6: Add Worktree Creation Function [AFK]

Minimal change: Use shell `git worktree add` to create a session worktree for a temp git repository.

Defers: Event persistence.

Verifies: Add and run one integration test that creates a worktree.

### Step 3.7: Record Worktree Created Event [AFK]

Minimal change: Persist source project path, HEAD, dirty-state summary, and worktree path as events after worktree creation.

Defers: Server session action.

Verifies: Add and run one storage/worktree integration test.

### Step 3.8: Load Missing Project Context As No-Op [AFK]

Minimal change: Add context loader that returns no context when `AGENTS.md` is missing.

Defers: Provider injection.

Verifies: Add and run one test for missing `AGENTS.md`.

### Step 3.9: Load Present Project Context [AFK]

Minimal change: Read project-root `AGENTS.md` content and metadata when present and non-empty.

Defers: Empty-file handling.

Verifies: Add and run one test for present `AGENTS.md`.

### Step 3.10: Treat Empty Project Context As No-Op [AFK]

Minimal change: Return no context for empty or whitespace-only `AGENTS.md`.

Defers: Provider injection.

Verifies: Add and run one test for empty `AGENTS.md`.

### Step 3.11: Add Pre-Turn Snapshot Event Recording [AFK]

Minimal change: Record current git HEAD and dirty-state metadata before assistant tool execution begins.

Defers: Actual agent loop.

Verifies: Add and run one integration test that records a snapshot event.

### Step 3.12: Resolve Latest Undo Snapshot [AFK]

Minimal change: Add storage/projector logic to find the latest valid pre-turn snapshot for a session.

Defers: Git reset.

Verifies: Add and run one storage test for snapshot resolution.

### Step 3.13: Implement Undo Git Reset [AFK]

Minimal change: Reset the session worktree to the latest snapshot using shell `git`.

Defers: Slash command wiring.

Verifies: Add and run one integration test that edits a temp repo file and restores it.

### Step 3.14: Persist Undo Result Events [AFK]

Minimal change: Persist undo requested, completed, and failed events around undo execution.

Defers: WebSocket display.

Verifies: Add and run one integration test for undo result events.

## Phase 4: Tool Contract And Built-In Tools

### Step 4.1: Add Tool Result Schema Field [AFK]

Minimal change: Add `result_schema` to `ToolDescriptor` and update descriptor construction tests.

Defers: JSON schema validation.

Verifies: `cargo test -p vulcan-tools`.

### Step 4.2: Add Tool Timeout Field [AFK]

Minimal change: Add a timeout field to `ToolDescriptor` sourced from config by callers.

Defers: Runtime enforcement.

Verifies: `cargo test -p vulcan-tools`.

### Step 4.3: Add Tool Source Field [AFK]

Minimal change: Add a typed source field distinguishing built-in tools from future external sources.

Defers: MCP and plugins.

Verifies: Add and run descriptor serde test.

### Step 4.4: Add Typed Tool Input Enum Boundary [AFK]

Minimal change: Define typed input structs/enums for built-in tools instead of exposing raw `serde_json::Value` at the final tool API boundary.

Defers: Provider argument validation.

Verifies: `cargo test -p vulcan-tools`.

### Step 4.5: Add Tool Result Envelope Type [AFK]

Minimal change: Define typed success, error, cancellation, and partial-output result envelopes.

Defers: Tool implementations.

Verifies: Add and run result envelope serde test.

### Step 4.6: Add Registry Collision Test For Source-Aware Descriptors [AFK]

Minimal change: Update registry tests so duplicate names still fail after descriptor field expansion.

Defers: Policy filtering.

Verifies: `cargo test -p vulcan-tools registry::tests`.

### Step 4.7: Add Session Policy Filtering Skeleton [AFK]

Minimal change: Add registry method that returns descriptors allowed by a provided session policy.

Defers: Approval gate implementation.

Verifies: Add and run one registry filtering test.

### Step 4.8: Implement Path Normalization Helper [AFK]

Minimal change: Add a helper that resolves user paths inside a worktree and rejects path escapes.

Defers: Individual tools.

Verifies: Add and run path helper tests for normal path and `..` escape.

### Step 4.9: Implement `read` Missing File Error [AFK]

Minimal change: Implement `read` enough to return a typed missing-file error.

Defers: Successful reads.

Verifies: Add and run one read tool missing-file test.

### Step 4.10: Implement `read` Success Path [AFK]

Minimal change: Read a regular file inside the worktree and return bounded content.

Defers: Directory rejection and truncation metadata.

Verifies: Add and run one read tool success test.

### Step 4.11: Implement `read` Directory Rejection [AFK]

Minimal change: Return a typed error when `read` targets a directory.

Defers: Output limits.

Verifies: Add and run one read tool directory test.

### Step 4.12: Implement `read` Output Truncation [AFK]

Minimal change: Enforce configured read output limits and set truncation metadata.

Defers: Binary file handling.

Verifies: Add and run one read tool truncation test.

### Step 4.13: Implement `glob` Success Path [AFK]

Minimal change: Return stable sorted relative paths for a glob pattern inside the worktree.

Defers: Truncation and include/exclude refinements.

Verifies: Add and run one glob tool success test.

### Step 4.14: Implement `glob` Truncation [AFK]

Minimal change: Enforce configured result limit and set truncation metadata.

Defers: Advanced pattern options.

Verifies: Add and run one glob truncation test.

### Step 4.15: Implement `grep` Invalid Regex Error [AFK]

Minimal change: Return a typed error for invalid regex input.

Defers: Successful grep.

Verifies: Add and run one grep invalid-regex test.

### Step 4.16: Implement `grep` Success Path [AFK]

Minimal change: Search files for a regex and return stable sorted file/line matches.

Defers: Include filters and truncation.

Verifies: Add and run one grep success test.

### Step 4.17: Implement `grep` Include Filter [AFK]

Minimal change: Restrict grep files using include patterns.

Defers: Truncation.

Verifies: Add and run one grep include-filter test.

### Step 4.18: Implement `grep` Truncation [AFK]

Minimal change: Enforce configured match limit and set truncation metadata.

Defers: Performance tuning.

Verifies: Add and run one grep truncation test.

### Step 4.19: Implement `write` Create Path [AFK]

Minimal change: Create a new file inside the worktree and return content hash metadata.

Defers: Replace and no-op behavior.

Verifies: Add and run one write create test.

### Step 4.20: Implement `write` Replace Path [AFK]

Minimal change: Replace an existing file and return old/new content hash metadata.

Defers: Idempotent no-op.

Verifies: Add and run one write replace test.

### Step 4.21: Implement `write` No-Op Path [AFK]

Minimal change: Detect identical content and return a no-op result without rewriting.

Defers: Parent directory creation.

Verifies: Add and run one write no-op test.

### Step 4.22: Implement `write` Parent Creation [AFK]

Minimal change: Create missing parent directories under the worktree.

Defers: Atomic write behavior.

Verifies: Add and run one write parent-creation test.

### Step 4.23: Implement `edit` No-Match Error [AFK]

Minimal change: Return a typed error when the target string is absent.

Defers: Successful replacement.

Verifies: Add and run one edit no-match test.

### Step 4.24: Implement `edit` Ambiguous-Match Error [AFK]

Minimal change: Return a typed error when the target string appears more than once.

Defers: Successful replacement.

Verifies: Add and run one edit ambiguous-match test.

### Step 4.25: Implement `edit` Single Replacement [AFK]

Minimal change: Replace exactly one target string and return content hash metadata.

Defers: Idempotent retry.

Verifies: Add and run one edit success test.

### Step 4.26: Implement `edit` Idempotent Retry [AFK]

Minimal change: Treat retry with already-applied replacement as a safe no-op when detectable.

Defers: Patch tool.

Verifies: Add and run one edit retry test.

### Step 4.27: Implement `apply_patch` Parse Error [AFK]

Minimal change: Return a typed parse error for malformed patch input.

Defers: Applying patches.

Verifies: Add and run one apply-patch parse-error test.

### Step 4.28: Implement `apply_patch` Add File [AFK]

Minimal change: Support adding a new file inside the worktree.

Defers: Update and delete operations.

Verifies: Add and run one apply-patch add-file test.

### Step 4.29: Implement `apply_patch` Update File [AFK]

Minimal change: Support updating an existing file with context matching.

Defers: Stale-target protection.

Verifies: Add and run one apply-patch update-file test.

### Step 4.30: Implement `apply_patch` Delete File [AFK]

Minimal change: Support deleting an existing file inside the worktree.

Defers: Multi-file patch behavior.

Verifies: Add and run one apply-patch delete-file test.

### Step 4.31: Implement `apply_patch` Stale Target Protection [AFK]

Minimal change: Reject a patch when its expected blob SHA or idempotency key does not match the current file state.

Defers: Provider integration.

Verifies: Add and run one stale-target test.

### Step 4.32: Implement Bash Success Path [AFK]

Minimal change: Run a non-interactive shell command inside the active worktree and capture stdout/stderr/exit code.

Defers: Timeout and policy blocking.

Verifies: Add and run one bash success test.

### Step 4.33: Implement Bash Non-Zero Result [AFK]

Minimal change: Return a typed non-zero result without panicking.

Defers: Timeout.

Verifies: Add and run one bash non-zero test.

### Step 4.34: Implement Bash Timeout [AFK]

Minimal change: Kill timed-out command execution and return timeout metadata.

Defers: Redaction.

Verifies: Add and run one bash timeout test.

### Step 4.35: Implement Bash Secret Redaction [AFK]

Minimal change: Redact configured secret values from bash stdout and stderr before returning results.

Defers: Persistence integration.

Verifies: Add and run one bash redaction test.

## Phase 5: Provider And Agent Loop

### Step 5.1: Define Provider Request Types [AFK]

Minimal change: Add typed provider request, message, content block, and tool descriptor mapping types.

Defers: HTTP provider.

Verifies: Add and run provider request serde tests.

### Step 5.2: Define Provider Response Types [AFK]

Minimal change: Add typed provider response, assistant message, tool call, finish reason, and usage types.

Defers: Streaming chunks.

Verifies: Add and run provider response serde tests.

### Step 5.3: Define Provider Streaming Chunk Types [AFK]

Minimal change: Add typed streaming chunk enum for assistant text, tool call delta, usage, and finish events.

Defers: WebSocket streaming.

Verifies: Add and run streaming chunk serde tests.

### Step 5.4: Map Projected Messages To Provider Request [AFK]

Minimal change: Convert projected session messages into provider request messages.

Defers: Tool descriptors and context injection.

Verifies: Add and run one mapping test.

### Step 5.5: Map Tool Descriptors To OpenAI-Compatible Schema [AFK]

Minimal change: Convert registered tool descriptors into OpenAI-compatible tool definitions.

Defers: Provider HTTP call.

Verifies: Add and run one descriptor mapping test.

### Step 5.6: Build OpenAI-Compatible HTTP Request [AFK]

Minimal change: Build the HTTP request body and headers without sending the network call.

Defers: HTTP client execution.

Verifies: Add and run fixture test for request construction.

### Step 5.7: Parse OpenAI-Compatible Assistant Text Response [AFK]

Minimal change: Parse a fixture response containing assistant text and usage.

Defers: Tool call parsing.

Verifies: Add and run fixture response parse test.

### Step 5.8: Parse OpenAI-Compatible Tool Call Response [AFK]

Minimal change: Parse a fixture response containing one tool call.

Defers: Agent tool execution.

Verifies: Add and run fixture tool-call parse test.

### Step 5.9: Classify Provider HTTP Errors [AFK]

Minimal change: Map auth, rate-limit, timeout, generic HTTP, and malformed response failures to typed provider errors.

Defers: Retry behavior.

Verifies: Add and run provider error classification tests.

### Step 5.10: Add Fake Provider For Tests [AFK]

Minimal change: Add a test-only fake provider that returns a fixed assistant response.

Defers: Agent loop.

Verifies: `cargo test -p vulcan-llm` or provider crate tests.

### Step 5.11: Implement One-Turn Text Agent Loop [AFK]

Minimal change: Given a user message and fake provider text response, append ordered user, provider start/finish, and assistant message events.

Defers: Tool calls.

Verifies: Add and run one fake-provider agent loop test.

### Step 5.12: Inject Project Context Into Provider Request [AFK]

Minimal change: Include loaded `AGENTS.md` context in provider request construction.

Defers: Skill summaries.

Verifies: Add and run one fake-provider request capture test.

### Step 5.13: Inject Skill Summaries Into Provider Request [AFK]

Minimal change: Include discovered skill names and descriptions, not full skill content, in provider request construction.

Defers: `skill` tool.

Verifies: Add and run one fake-provider request capture test.

### Step 5.14: Validate Provider Tool Call Name [AFK]

Minimal change: Return a typed error event when provider requests an unknown tool.

Defers: Input schema validation.

Verifies: Add and run one fake-provider unknown-tool test.

### Step 5.15: Validate Provider Tool Call Input [AFK]

Minimal change: Validate tool-call arguments against the tool input type/schema before execution.

Defers: Approval resolution.

Verifies: Add and run one fake-provider invalid-input test.

### Step 5.16: Execute One Read-Only Tool Call [AFK]

Minimal change: Execute a fake provider `read` tool call through the registry and persist request/result events.

Defers: Multiple tool calls and approval.

Verifies: Add and run one fake-provider read-tool test.

### Step 5.17: Execute Multiple Tool Calls In Order [AFK]

Minimal change: Execute multiple provider tool calls sequentially and persist events in order.

Defers: Parallelism.

Verifies: Add and run one multi-tool ordering test.

### Step 5.18: Stream Agent Events Over WebSocket [AFK]

Minimal change: Send persisted provider and tool progress events to the connected WebSocket client in event order.

Defers: Approval UI.

Verifies: Add and run one WebSocket ordering integration test.

## Phase 6: Guardrails, Approval, Skills, And Slash Commands

### Step 6.1: Define Approval Resolution Type [AFK]

Minimal change: Add a typed result for approval resolution: allow, deny, or prompt.

Defers: Mode matrix.

Verifies: `cargo test -p vulcan-guardrails`.

### Step 6.2: Implement Approval Matrix For Never Class [AFK]

Minimal change: `ApprovalClass::Never` always resolves to allow for all session modes.

Defers: OnMode and Always classes.

Verifies: Add and run approval matrix tests for `Never`.

### Step 6.3: Implement Approval Matrix For Always Class [AFK]

Minimal change: `ApprovalClass::Always` resolves to prompt or deny according to PRD mode semantics.

Defers: OnMode class.

Verifies: Add and run approval matrix tests for `Always`.

### Step 6.4: Implement Approval Matrix For OnMode Class [AFK]

Minimal change: `ApprovalClass::OnMode` resolves based on current `ask`, `plan`, or `build` mode.

Defers: Persistence.

Verifies: Add and run approval matrix tests for `OnMode`.

### Step 6.5: Persist Approval Request On Prompt [AFK]

Minimal change: When resolution is prompt, persist an approval request event.

Defers: User decision handling.

Verifies: Add and run one approval request persistence test.

### Step 6.6: Persist Approval Decision [AFK]

Minimal change: Persist approval approved or denied decisions as events.

Defers: WebSocket approval UI.

Verifies: Add and run one approval decision persistence test.

### Step 6.7: Add Command Allow/Deny Pattern Types [AFK]

Minimal change: Define config-backed allowed and denied command pattern types.

Defers: Bash integration.

Verifies: `cargo test -p vulcan-guardrails`.

### Step 6.8: Implement Denied Command Pattern Blocking [AFK]

Minimal change: Block bash commands matching denied patterns with a typed policy error.

Defers: Allowlist behavior.

Verifies: Add and run one denied-pattern test.

### Step 6.9: Implement Allowed Command Pattern Filtering [AFK]

Minimal change: If allowed patterns are configured, require bash commands to match at least one.

Defers: Approval interaction.

Verifies: Add and run one allowed-pattern test.

### Step 6.10: Block Autonomous Skill Authoring [AFK]

Minimal change: Reject `skills.write` or equivalent mutation in MVP policy.

Defers: Post-MVP skill authoring.

Verifies: Add and run one blocked-skill-authoring test.

### Step 6.11: Add Secret Redaction For Known Formats [AFK]

Minimal change: Redact common API key, token, and password formats from strings.

Defers: Configured environment values.

Verifies: Add and run redaction tests for known formats.

### Step 6.12: Add Secret Redaction For Configured Env Values [AFK]

Minimal change: Redact actual configured secret environment values from strings without logging them.

Defers: Recall indexing integration.

Verifies: Add and run one configured-env redaction test.

### Step 6.13: Discover Valid Skills By Directory [AFK]

Minimal change: Discover `SKILL.md` files from configured directories and return name/description only.

Defers: Full skill loading.

Verifies: Add and run one valid skill discovery test.

### Step 6.14: Reject Malformed Skill Metadata [AFK]

Minimal change: Return a typed error for a `SKILL.md` missing required name or description.

Defers: Duplicate detection.

Verifies: Add and run one malformed skill test.

### Step 6.15: Reject Duplicate Skill Names [AFK]

Minimal change: Return a typed error when two discovered skills have the same name.

Defers: Skill loading.

Verifies: Add and run one duplicate skill test.

### Step 6.16: Implement `skill` Tool Unknown Skill Error [AFK]

Minimal change: Return a typed error when the `skill` tool requests an unknown skill name.

Defers: Successful full content loading.

Verifies: Add and run one unknown skill tool test.

### Step 6.17: Implement `skill` Tool Success Path [AFK]

Minimal change: Load full `SKILL.md` content for a known skill by name.

Defers: Path escape hardening.

Verifies: Add and run one skill load test.

### Step 6.18: Reject Escaping Skill Paths [AFK]

Minimal change: Reject discovered or requested skill files that escape configured skill roots.

Defers: None.

Verifies: Add and run one skill path-escape test.

### Step 6.19: Add Slash Command Parser Enum [AFK]

Minimal change: Parse slash command names into typed enum variants without executing them.

Defers: Arguments and execution.

Verifies: Add and run parser tests for all command names.

### Step 6.20: Parse `/model <name>` Arguments [AFK]

Minimal change: Parse and validate `/model` argument as a non-empty model name.

Defers: Execution.

Verifies: Add and run parser tests for valid and invalid `/model`.

### Step 6.21: Parse `/permissions <mode>` Arguments [AFK]

Minimal change: Parse and validate `/permissions` mode values.

Defers: Execution.

Verifies: Add and run parser tests for valid and invalid `/permissions`.

### Step 6.22: Parse `/recall <query>` Arguments [AFK]

Minimal change: Parse and validate non-empty `/recall` query text.

Defers: Recall execution.

Verifies: Add and run parser tests for valid and invalid `/recall`.

### Step 6.23: Execute `/model` Server Action [AFK]

Minimal change: Persist model-change event and return typed status without calling provider.

Defers: WebSocket UI display.

Verifies: Add and run one server action test for `/model`.

### Step 6.24: Execute `/permissions` Server Action [AFK]

Minimal change: Persist permissions-change event and return typed status without calling provider.

Defers: Approval UI.

Verifies: Add and run one server action test for `/permissions`.

### Step 6.25: Execute `/clear` Server Action [AFK]

Minimal change: Persist clear-context event and return typed status without deleting events.

Defers: UI transcript behavior.

Verifies: Add and run one server action test for `/clear`.

### Step 6.26: Execute `/status` Server Action [AFK]

Minimal change: Return current projected session status without calling provider.

Defers: Rich status fields.

Verifies: Add and run one server action test for `/status`.

### Step 6.27: Execute `/sessions` Server Action [AFK]

Minimal change: Return stored sessions by recency without calling provider.

Defers: Search/filtering.

Verifies: Add and run one server action test for `/sessions`.

### Step 6.28: Execute `/resume <id>` Server Action [AFK]

Minimal change: Project an existing session by ID and return typed status.

Defers: Web UI session switch.

Verifies: Add and run one server action test for `/resume`.

### Step 6.29: Execute `/new` Server Action [AFK]

Minimal change: Create a new session, worktree, and initial events without calling provider.

Defers: Web UI creation flow.

Verifies: Add and run one server action test for `/new`.

### Step 6.30: Execute `/undo` Server Action [AFK]

Minimal change: Call undo implementation and return typed status without provider involvement.

Defers: UI rendering.

Verifies: Add and run one server action test for `/undo`.

## Phase 7: Recall In Storage

### Step 7.1: Add Recall Index Row Type [AFK]

Minimal change: Define typed rows for indexed recall content: session ID, field, text, timestamp, and source ID.

Defers: Index writes.

Verifies: `cargo test -p vulcan-storage`.

### Step 7.2: Index Message Events [AFK]

Minimal change: Index redacted message text into storage-owned FTS5 recall tables.

Defers: Tools and errors.

Verifies: Add and run one recall indexing test for messages.

### Step 7.3: Index Tool Events [AFK]

Minimal change: Index tool names and redacted tool input/output content.

Defers: Error and file path indexing.

Verifies: Add and run one recall indexing test for tools.

### Step 7.4: Index Error Events [AFK]

Minimal change: Index redacted error messages.

Defers: Summaries and titles.

Verifies: Add and run one recall indexing test for errors.

### Step 7.5: Index Session Summary Events [AFK]

Minimal change: Index session titles and summaries.

Defers: Search tool.

Verifies: Add and run one recall indexing test for summaries.

### Step 7.6: Implement `session_search` No-Match Path [AFK]

Minimal change: Return an empty typed result for a query with no matches.

Defers: Match snippets.

Verifies: Add and run one no-match search test.

### Step 7.7: Implement `session_search` Match Path [AFK]

Minimal change: Return session IDs, snippets, matching fields, and timestamps for matching indexed content.

Defers: Result limit enforcement.

Verifies: Add and run one search match test.

### Step 7.8: Enforce Recall Search Limits [AFK]

Minimal change: Apply configured result limits to `session_search`.

Defers: Ranking improvements.

Verifies: Add and run one search limit test.

### Step 7.9: Implement `session_open` Missing Session Error [AFK]

Minimal change: Return a typed error for missing session ID.

Defers: Successful open.

Verifies: Add and run one missing-session test.

### Step 7.10: Implement `session_open` Surrounding Turns [AFK]

Minimal change: Return selected surrounding turns for an existing session.

Defers: Full trace.

Verifies: Add and run one surrounding-turns test.

### Step 7.11: Implement `run_trace` Bounded Trace [AFK]

Minimal change: Return a bounded full event trace for a session.

Defers: UI replay.

Verifies: Add and run one bounded trace test.

### Step 7.12: Wire `/recall <query>` Server Action [AFK]

Minimal change: Execute `session_search` for `/recall` and return typed results without invoking the LLM.

Defers: Web UI rendering.

Verifies: Add and run one server action test for `/recall`.

### Step 7.13: Stream Recall Results Over WebSocket [AFK]

Minimal change: Send `/recall` results to the browser as typed WebSocket events.

Defers: UI styling.

Verifies: Add and run one WebSocket integration test for `/recall`.

## Phase 8: Web UI And Mobile Experience

### Step 8.1: Add Minimal HTML Shell [AFK]

Minimal change: Serve a vanilla HTML shell with transcript area, input, send button, and status area.

Defers: Styling and WebSocket behavior.

Verifies: Manual or server test confirms `/` contains expected element IDs.

### Step 8.2: Add Responsive CSS [AFK]

Minimal change: Add CSS that supports desktop and narrow mobile widths.

Defers: Visual polish.

Verifies: Manual browser smoke test at desktop and mobile widths.

### Step 8.3: Add WebSocket Connect JS [AFK]

Minimal change: Browser JS opens `/ws` and displays connection state.

Defers: Sending messages.

Verifies: Manual browser smoke test shows connected state.

### Step 8.4: Add Send Message JS [AFK]

Minimal change: Send typed user message envelope over WebSocket from the input form.

Defers: Rendering server replies.

Verifies: Manual browser smoke test shows outbound message reaches server logs or test hook.

### Step 8.5: Render Assistant And User Messages [AFK]

Minimal change: Render typed user and assistant message events into the transcript.

Defers: Tool events.

Verifies: Manual browser smoke test with fake events.

### Step 8.6: Render Tool Events [AFK]

Minimal change: Render tool request/result events distinctly from chat messages.

Defers: Approval UI.

Verifies: Manual browser smoke test with fake tool event.

### Step 8.7: Render Error Events [AFK]

Minimal change: Render protocol, provider, tool, and server errors visibly.

Defers: Retry controls.

Verifies: Manual browser smoke test with malformed WebSocket message.

### Step 8.8: Render Pending Approval [AFK]

Minimal change: Show pending approval details with redacted inputs.

Defers: Approve/deny actions.

Verifies: Manual browser smoke test with fake approval event.

### Step 8.9: Send Approval Decision [AFK]

Minimal change: Send approve or deny envelope over WebSocket from approval buttons.

Defers: Decision result rendering.

Verifies: Manual browser smoke test shows decision reaches server.

### Step 8.10: Add Manifest [AFK]

Minimal change: Add a valid web manifest served as a static asset.

Defers: Service worker.

Verifies: Browser application panel shows valid manifest.

### Step 8.11: Add Static-Only Service Worker [AFK]

Minimal change: Add a service worker that caches only static assets.

Defers: Offline session behavior.

Verifies: Browser application panel shows active service worker.

### Step 8.12: Add Dictation Feature Detection [AFK]

Minimal change: Show a dictation button only when browser speech recognition is available.

Defers: Speech-to-input behavior.

Verifies: Manual browser smoke test on supported and unsupported browser.

### Step 8.13: Add Dictation To Input [AFK]

Minimal change: Use Web Speech API dictation to fill the message input.

Defers: Voice command interpretation.

Verifies: Manual browser smoke test where supported.

## Phase 9: MVP Hardening And Dogfood Readiness

### Step 9.1: Add Dogfood Checklist Document [HITL]

Minimal change: Create a checklist for starting Vulcan against a real project and creating a session from the web UI.

Defers: Running the checklist.

Verifies: Checklist file exists and references exact commands.

### Step 9.2: Run Dogfood Session Start [HITL]

Minimal change: Start `vulcan serve --project <path>` and create one session from the web UI.

Defers: Tool execution and undo.

Verifies: Checklist records pass/fail for session creation.

### Step 9.3: Run Dogfood Context Injection [HITL]

Minimal change: Verify `AGENTS.md` context appears in a fake or captured provider request.

Defers: Edit and test flow.

Verifies: Checklist records pass/fail for context injection.

### Step 9.4: Run Dogfood Tool Flow [HITL]

Minimal change: Ask agent to inspect files, edit code, and run tests in a real project.

Defers: Undo and recall.

Verifies: Checklist records pass/fail for read/edit/bash flow.

### Step 9.5: Run Dogfood Undo Flow [HITL]

Minimal change: Run `/undo` after a file mutation and verify restoration.

Defers: Recall.

Verifies: Checklist records pass/fail for undo.

### Step 9.6: Run Dogfood Recall Flow [HITL]

Minimal change: Use `/recall` to find prior session content.

Defers: Release checklist.

Verifies: Checklist records pass/fail for recall.

### Step 9.7: Add Release Checklist [AFK]

Minimal change: Document formatting, linting, tests, docs review, WebSocket smoke test, web UI smoke test, mobile smoke test, and dogfood scenario.

Defers: Performance measurements.

Verifies: Checklist can be followed from a fresh clone.

### Step 9.8: Add Startup Timing Script [AFK]

Minimal change: Add a local script or documented command to measure server cold start without committing generated output.

Defers: RSS measurement.

Verifies: Command runs locally and writes output outside committed artifacts.

### Step 9.9: Add Idle RSS Measurement Command [AFK]

Minimal change: Add a local script or documented command to measure idle RSS without committing generated output.

Defers: Threshold tuning.

Verifies: Command runs locally and writes output outside committed artifacts.

## Deferred Post-MVP Tracks

These are explicitly out of MVP unless `docs/PRD.md` changes:

- MCP support.
- Subagents and delegation.
- Telegram, WhatsApp, or Signal gateway.
- LSP diagnostics and code intelligence.
- Tree-sitter, BM25, semantic, vector, or graph codebase indexing.
- Agent-curated memory or autonomous skill authoring.
- Self-improving loop.
- Session forks, trees, or JSONL storage.
- TUI, desktop app, IDE extensions, cloud hosting, OAuth, OpenTelemetry, WASM plugins, and Anthropic Messages protocol.

## Completed Slices

### Slice 1: Phase 0 — Baseline and PRD alignment [DONE]

Steps: 0.1, 0.2, 0.3, 0.4 (Workspace reconciliation, vocabulary alignment, config placeholders)

### Slice 2: Phase 1 — Durable session core (typed events and storage) [DONE]

Steps: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.12, 1.13, 1.14, 1.15, 1.16, 1.17, 1.18, 1.19, 1.20

## Next Implementation Slice

The next practical slice should implement remaining Phase 1 steps (tool event payloads, approval event payloads, provider event fields, snapshot/undo events, slash command events) and then begin Phase 2 (config.toml loading, CLI, server shell):

1. Step 1.7: Add tool call event payload fields.
2. Step 1.8: Add approval event payload fields.
3. Step 1.9: Add provider event payload fields.
4. Step 1.10: Add snapshot and undo event variants.
5. Step 1.11: Add slash command effect event variant.
6. Step 1.21: Store queryable message metadata on append.
7. Step 1.22: Store queryable tool metadata on append.
8. Step 1.23: Store queryable provider metadata on append.
9. Step 1.24: Store queryable approval metadata on append.
10. Step 1.25: Fail projector on unsupported event version.
11. Step 1.26: Project current model from events.
12. Step 1.27: Project current approval mode from events.
13. Step 1.28: Project clear context state.

After that slice, run the full cross-phase validation command set.
