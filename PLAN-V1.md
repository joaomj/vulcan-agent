# Vulcan Agent Implementation Plan V1

This plan turns `docs/ARCHITECTURE.md` into small, atomic implementation steps. It is ordered for incremental delivery: each step has a concrete output, validation, and a narrow boundary. MVP work is separated from post-MVP work.

## Ground Rules

- Keep the core small. Defer optional features unless a step explicitly includes them.
- Prefer one crate, one capability, one validation target per step.
- Every mutation must be idempotent or fail safely with a visible error.
- Every configurable duration, threshold, host allowlist, and limit must come from config.
- Do not silently swallow failures. Return typed errors, persist error events where relevant, and log structured application errors.
- Use SQLite as the only session storage format in MVP.
- Use shell `git` through async process execution. Do not use `git2`.
- Use WebSocket for client interaction in MVP. Reserve HTTP for health and static assets.
- Keep the web UI framework-free: static HTML, CSS, and JavaScript.
- Treat linters, tests, and build commands as authoritative validation.

## Phase 0: Repository And Delivery Setup

### Step 0.1: Confirm Project Baseline

Deliverable: Record the current repository state and decide whether this is a fresh scaffold or an existing Rust workspace.

Tasks:
- Inspect the repository layout.
- Identify existing Rust, docs, CI, and config files.
- Record the selected base branch and current worktree state before edits.

Validation:
- `git status --short`
- Repository layout is understood before scaffold changes begin.

### Step 0.2: Create The Rust Workspace Skeleton

Deliverable: A compiling Cargo workspace with empty crate boundaries matching the spec.

Tasks:
- Add root `Cargo.toml` workspace.
- Add crates: `core`, `storage`, `llm`, `providers`, `tools`, `recall`, `skills`, `guardrails`, `server`, `web`, and `cli`.
- Defer `mcp` and `code_index` crates until their phases unless empty placeholders are required by workspace policy.
- Add minimal `lib.rs` or `main.rs` files.

Validation:
- `cargo check --workspace`

### Step 0.3: Add Shared Tooling Configuration

Deliverable: Baseline formatting, linting, and test commands are documented and runnable.

Tasks:
- Add `rustfmt.toml` only if project-specific formatting is needed.
- Add Clippy expectations in docs or CI config.
- Add a short development section to docs or README if no README exists.

Validation:
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`

## Phase 1: Foundation

### Step 1.1: Define Core Identifiers And Error Types

Deliverable: Strongly typed IDs and shared domain errors in `crates/core`.

Tasks:
- Define IDs for sessions, messages, tool calls, events, provider calls, approvals, and worktrees.
- Define timestamp and ordering conventions.
- Define core error enums using typed variants.

Validation:
- Unit tests cover ID serialization and error formatting.
- `cargo test -p vulcan-core`

### Step 1.2: Define Session And Message Domain Types

Deliverable: Core session and message structs without persistence concerns.

Tasks:
- Define `Session`, `SessionMode`, `ApprovalMode`, `Message`, `MessageRole`, and `ContentBlock`.
- Include fields needed for future non-conversational sessions without assuming a human-created first message.
- Keep provider-specific fields out of core session types.

Validation:
- Unit tests cover serde round trips.
- No raw untyped maps at API boundaries.

### Step 1.3: Define Event Model

Deliverable: Append-only event enum in `crates/core`.

Tasks:
- Define events for session creation, message append, tool call requested, tool result received, approval requested, approval decided, provider call started, provider call finished, error recorded, and session summary updated.
- Include event sequence number and creation time.
- Ensure event payloads are typed.

Validation:
- Unit tests cover event serialization.
- Event ordering rules are documented in code comments or module docs.

### Step 1.4: Define Tool Contract Types

Deliverable: Shared tool descriptor, tool result, and policy types in `crates/core` or `crates/tools`.

Tasks:
- Define `ToolDescriptor`, `ApprovalClass`, `IsolationLevel`, `CancellationSupport`, `SubagentPolicy`, `ToolResult`, `ToolStatus`, `ToolError`, and `ToolMetadata`.
- Include typed result envelope for success, error, cancellation, and partial results.
- Keep `structured` constrained by declared result schema.

Validation:
- Unit tests cover descriptor filtering and result serialization.

### Step 1.5: Add Configuration Loader

Deliverable: Config module that loads `config.toml` with defaults.

Tasks:
- Support `[tools]` settings from the spec.
- Accept an empty `[budgets]` section.
- Add provider config for OpenAI-compatible base URL, model, and API key environment variable name.
- Add application log and data directory settings.
- Reject invalid values with clear errors.

Validation:
- Unit tests cover defaults, overrides, missing optional sections, and invalid values.

### Step 1.6: Create SQLite Schema And Migrations

Deliverable: Initial SQLite schema in `crates/storage`.

Tasks:
- Add migrations for sessions, events, messages, tool calls, provider calls, approvals, errors, token usage, and audit metadata.
- Store append-only events as typed serialized payloads plus queryable metadata.
- Add indexes needed for session listing and event replay.

Validation:
- Migration tests run against an in-memory or temporary SQLite database.
- Re-running migrations is safe.

### Step 1.7: Implement Event Store

Deliverable: Append and replay APIs for persisted events.

Tasks:
- Implement append with idempotency protection for event IDs or sequence numbers.
- Implement replay by session and list sessions by recency.
- Return typed storage errors.

Validation:
- Unit tests cover append, replay order, duplicate append, missing session, and transaction rollback.

### Step 1.8: Implement Session Projector

Deliverable: Project current session state from stored events.

Tasks:
- Build session state from event streams.
- Track messages, tool calls, approvals, provider calls, errors, and current mode.
- Handle unknown future event versions visibly instead of ignoring them.

Validation:
- Unit tests cover projection from realistic event sequences.

### Step 1.9: Define LLM Provider Trait

Deliverable: Provider abstraction in `crates/llm`.

Tasks:
- Define request, response, streaming chunk, tool-call, and usage types.
- Keep provider trait independent from HTTP implementation details.
- Support cancellation at request boundaries.

Validation:
- Unit tests cover serialization of provider request and response types.

### Step 1.10: Implement OpenAI-Compatible Provider

Deliverable: Generic OpenAI-compatible chat-completions route in `crates/providers`.

Tasks:
- Implement request mapping from domain messages and tool descriptors.
- Implement streaming response parsing if included in MVP step scope, otherwise implement non-streaming first and create a follow-up step for streaming.
- Read API key through configured environment variable name at runtime.
- Surface HTTP, auth, rate limit, timeout, and malformed response failures.

Validation:
- Unit tests cover request construction and response parsing using fixtures.
- No tests require a real provider API key.

### Step 1.11: Implement Built-In Read Tool

Deliverable: `read` tool with typed input and output.

Tasks:
- Read files inside the session worktree by default.
- Reject missing files, directories where files are expected, and out-of-worktree paths.
- Apply output size limits from config.

Validation:
- Unit tests cover normal read, missing file, directory input, and path escape.

### Step 1.12: Implement Built-In Glob Tool

Deliverable: `glob` tool with typed input and output.

Tasks:
- Search paths inside the session worktree.
- Return stable sorted relative paths.
- Apply configured limits and return partial status when truncated.

Validation:
- Unit tests cover matches, no matches, path escape, and truncation.

### Step 1.13: Implement Built-In Grep Tool

Deliverable: `grep` tool with typed input and output.

Tasks:
- Search file contents inside the session worktree.
- Support include patterns.
- Return file path, line number, and line content with configured limits.

Validation:
- Unit tests cover matches, invalid regex, include filters, path escape, and truncation.

### Step 1.14: Implement Built-In Write Tool

Deliverable: `write` tool with safe upsert semantics.

Tasks:
- Create or replace files inside the session worktree.
- No-op if content already matches.
- Reject path escapes.
- Return previous and new content hashes.

Validation:
- Unit tests cover create, replace, no-op retry, parent directory creation, and path escape.

### Step 1.15: Implement Built-In Edit Tool

Deliverable: `edit` tool for deterministic string replacement.

Tasks:
- Replace exact text in a file inside the session worktree.
- Fail if the target text is absent or ambiguous unless the input explicitly supports replace-all.
- No-op if the requested final content already exists.

Validation:
- Unit tests cover one replacement, no match, ambiguous match, idempotent retry, and path escape.

### Step 1.16: Implement Built-In Apply Patch Tool

Deliverable: `apply_patch` tool with stale-target protection.

Tasks:
- Apply patch operations inside the session worktree.
- Require target blob SHA or equivalent idempotency key.
- Fail with a typed patch conflict on stale target.

Validation:
- Unit tests cover add, update, delete, stale key, retry, and path escape.

### Step 1.17: Implement Built-In Bash Tool, Non-Interactive First

Deliverable: Non-interactive `bash` tool scoped to the worktree.

Tasks:
- Run commands in the active session worktree.
- Apply configured timeout.
- Capture stdout, stderr, exit code, and duration.
- Surface timeout and spawn failures visibly.
- Defer PTY and password prompt mode to a later step.

Validation:
- Unit tests or integration tests cover success, non-zero exit, timeout, and working directory enforcement.

### Step 1.18: Implement Tool Registry

Deliverable: Registry that registers and lists built-in tools.

Tasks:
- Register built-in tools at startup.
- Reject name collisions with a typed conflict error.
- Filter by session policy and subagent policy.

Validation:
- Unit tests cover registration, collision, parent list, child list, and blocked tools.

### Step 1.19: Add Minimal HTTP Server

Deliverable: Axum server with health endpoint.

Tasks:
- Add `GET /health`.
- Add app state wiring for config, storage, provider, and tool registry.
- Add structured request logging.

Validation:
- Integration test verifies `GET /health` returns success.

### Step 1.20: Add Minimal WebSocket Endpoint

Deliverable: WebSocket endpoint that accepts client connections and echoes typed protocol errors for unsupported messages.

Tasks:
- Define initial client-server envelope types.
- Add `GET /ws` upgrade route.
- Validate incoming message shape.
- Return typed error messages for unsupported commands.

Validation:
- Integration test connects to WebSocket and verifies typed error handling.

### Step 1.21: Add CLI Serve Command

Deliverable: `vulcan-agent serve --project <path>` starts the server.

Tasks:
- Add `crates/cli` binary with `clap`.
- Validate the project path exists and is a git repository.
- Load config and initialize storage.
- Start HTTP and WebSocket server.

Validation:
- CLI test covers argument parsing.
- Manual smoke test starts server and reaches `/health`.

## Phase 2: Worktrees And Agent Loop

### Step 2.1: Implement Git Command Wrapper

Deliverable: Async wrapper around shell `git` commands.

Tasks:
- Centralize command execution, timeout, stdout, stderr, and exit status handling.
- Return typed git errors.
- Log command metadata without leaking secrets.

Validation:
- Tests cover success, non-zero exit, timeout, and missing git binary where feasible.

### Step 2.2: Implement Session Worktree Creation

Deliverable: New sessions create worktrees under configured data directory.

Tasks:
- Create `~/.vulcan/worktrees/<project>/<session-id>/` or configured equivalent.
- Validate source project is clean or record dirty state policy explicitly.
- Persist worktree path in session events.

Validation:
- Integration test creates a worktree for a temporary git repository.

### Step 2.3: Implement Worktree Cleanup Metadata

Deliverable: Worktree lifecycle state is tracked, without destructive cleanup automation yet.

Tasks:
- Record active, closed, and failed worktree states.
- Expose metadata for future cleanup commands.
- Do not delete user worktrees silently.

Validation:
- Tests cover lifecycle state transitions.

### Step 2.4: Implement Turn Snapshot

Deliverable: Snapshot before each assistant turn.

Tasks:
- Record git HEAD and dirty-state metadata before tool execution.
- Persist snapshot event.
- Fail visibly if snapshot cannot be recorded.

Validation:
- Integration test records snapshot before a file-changing turn.

### Step 2.5: Implement `/undo` Worktree Reset

Deliverable: Undo resets the worktree to the last assistant-turn snapshot.

Tasks:
- Resolve latest valid snapshot.
- Reset the worktree to that snapshot using shell `git`.
- Persist undo requested, undo completed, or undo failed events.

Validation:
- Integration test edits a file, runs undo, and verifies file state is restored.

### Step 2.6: Implement Agent Loop Skeleton

Deliverable: One-turn loop that sends messages to provider and persists responses.

Tasks:
- Build provider request from projected session state.
- Include current tool descriptors.
- Persist provider call start and finish events.
- Persist assistant message events.
- Model the loop as a state machine (prompt → LLM → tool calls → execution → stream results) with typed transitions, not ad-hoc conditionals.

Validation:
- Test with a fake provider returns an assistant message and persists ordered events.

### Step 2.7: Add Tool Call Execution To Agent Loop

Deliverable: Agent loop executes provider-requested tool calls through the registry.

Tasks:
- Validate tool name and input schema.
- Resolve approval decision before execution.
- Execute tool with cancellation and timeout.
- Persist tool call and tool result events.
- Emit typed streaming events per tool call so the WebSocket layer can push progress to clients without buffering the full result.

Validation:
- Test with fake provider requests `read`; loop executes and persists result.

### Step 2.8: Add Approval Gate

Deliverable: Tool calls are allowed, denied, or paused based on tool approval class and session approval mode.

Tasks:
- Implement decision matrix for `Never`, `OnMode`, `Always`, and `Blocked`.
- Persist approval request and decision events.
- Pause pending user approval where required.
- Return a typed `ApprovalRequired` error variant so the agent loop and WebSocket layer can handle the pause uniformly.

Validation:
- Unit tests cover every approval class and approval mode combination.

### Step 2.9: Stream Agent Events Over WebSocket

Deliverable: Clients receive session, provider, tool, and error events in order.

Tasks:
- Add subscribe or active-session message to WebSocket protocol.
- Broadcast persisted events to connected clients.
- Handle reconnect by replaying missed events from SQLite.

Validation:
- Integration test creates a session, triggers a turn, and verifies ordered WebSocket events.

### Step 2.10: Implement Slash Command Dispatcher

Deliverable: Client commands map to server actions without LLM involvement.

Tasks:
- Implement `/new`, `/sessions`, `/resume <id>`, `/status`, `/clear`, `/model <name>`, `/permissions <mode>`, and `/undo`.
- Defer `/recall` until recall tools exist.
- Store command effects as events where they change session state.

Validation:
- Unit tests parse commands.
- Integration tests verify state-changing commands persist events.

### Step 2.11: Add Basic Session API Over WebSocket

Deliverable: WebSocket supports starting sessions and sending user messages.

Tasks:
- Define client messages for new session, resume session, send user message, approve, deny, and cancel.
- Define server messages for event replay, event appended, pending approval, and error.
- Validate all message schemas.

Validation:
- Integration test covers new session, user message, assistant response with fake provider.

## Phase 3: Web Client

### Step 3.1: Serve Static Web Assets

Deliverable: Server serves static files from `crates/web`.

Tasks:
- Add `index.html`, `styles.css`, and `app.js`.
- Serve assets over HTTP.
- Keep assets minimal and framework-free.

Validation:
- Integration test verifies `GET /` returns HTML.

### Step 3.2: Build WebSocket Client Connection

Deliverable: Browser connects to server WebSocket and shows connection state.

Tasks:
- Implement connect, disconnect, reconnect, and error display.
- Render incoming events in a simple transcript.

Validation:
- Manual browser smoke test verifies connection and event rendering.

### Step 3.3: Add Message Input And Slash Command Handling

Deliverable: UI sends user messages and slash commands.

Tasks:
- Add text input and submit behavior.
- Detect slash commands client-side and send typed command messages.
- Show command errors visibly.

Validation:
- Manual smoke test covers `/status`, `/new`, and normal message submission.

### Step 3.4: Add Approval Prompt UI

Deliverable: UI can approve or deny gated tool calls.

Tasks:
- Render pending approval details.
- Provide approve and deny controls.
- Preserve redaction boundaries in displayed inputs.

Validation:
- Manual smoke test triggers an approval-required tool and approves or denies it.

### Step 3.5: Add Responsive Layout

Deliverable: Usable desktop and mobile layout.

Tasks:
- Add responsive CSS for narrow and wide screens.
- Keep transcript, input, and approval prompts accessible on mobile.

Validation:
- Manual browser test at desktop width and mobile width.

### Step 3.6: Add Web Speech Dictation

Deliverable: Optional browser speech-to-text fills the message input.

Tasks:
- Use Web Speech API when available.
- Hide or disable dictation control when unavailable.
- Do not add server-side transcription.

Validation:
- Manual smoke test in a supported browser.
- Unsupported browsers degrade visibly.

### Step 3.7: Add PWA Shell

Deliverable: Installable PWA shell.

Tasks:
- Add web manifest.
- Add service worker for static asset caching only.
- Avoid offline agent behavior beyond loading the shell.

Validation:
- Browser application panel shows valid manifest and active service worker.

## Phase 4: Recall, Skills, Guardrails

### Step 4.0: Implement Project Context File Loading

Deliverable: Project-local AGENTS.md is loaded and injected into every session's system context.

Tasks:
- Discover AGENTS.md (or equivalent) in the project root directory at session start.
- Load and cache its content for injection into the system prompt on every turn.
- Handle missing or empty AGENTS.md gracefully (no-op, no error).

Validation:
- Integration test creates a project with AGENTS.md and verifies its content appears in the provider request built by the agent loop.

### Step 4.1: Add FTS5 Session Search Tables

Deliverable: Search index tables owned by the session projector.

Tasks:
- Add FTS5 table for messages, tool data, errors, file paths, titles, and summaries.
- Redact sensitive fields before indexing.
- Update index from persisted events.

Validation:
- Migration and projector tests cover indexing and redaction.

### Step 4.2: Implement `session_search` Tool

Deliverable: Read-only recall search tool.

Tasks:
- Query FTS5 index.
- Return session ID, title, matching fields, snippets, and timestamps.
- Apply configured result limits.

Validation:
- Tests cover matches, no matches, redacted content, and result limits.

### Step 4.3: Implement `session_open` And `run_trace` Tools

Deliverable: Read-only trace retrieval tools.

Tasks:
- Retrieve selected surrounding turns for a session.
- Retrieve complete trace where allowed by limits.
- Preserve typed event and message boundaries.

Validation:
- Tests cover surrounding turns, full trace, missing session, and output limits.

### Step 4.4: Wire `/recall <query>`

Deliverable: Slash command invokes recall without LLM involvement.

Tasks:
- Parse `/recall <query>`.
- Execute `session_search` as a server action.
- Render results through WebSocket events.

Validation:
- Integration test verifies `/recall` returns search results.

### Step 4.5: Implement `SKILL.md` Discovery

Deliverable: Skills loader reads names and descriptions only.

Tasks:
- Discover configured skills directories.
- Parse `SKILL.md` name and description.
- Ignore unknown frontmatter keys and sections.
- Do not inject full skill content into system context.

Validation:
- Tests cover valid skill, unknown keys, malformed skill, and duplicate names.

### Step 4.6: Implement `skill` Tool

Deliverable: On-demand skill content loader.

Tasks:
- Load full skill content by name.
- Reject unknown skills visibly.
- Return content as typed tool result.

Validation:
- Tests cover successful load, unknown skill, malformed file, and path escape.

### Step 4.7: Implement Policy Engine

Deliverable: Guardrails evaluate tool calls before execution.

Tasks:
- Support allowed and denied command patterns.
- Support required approvals.
- Block `skills.write` in MVP.
- Emit clear denial reasons.

Validation:
- Unit tests cover allow, deny, required approval, and blocked tool behavior.

### Step 4.8: Add Tool JSON Schema Validation

Deliverable: Tool arguments and structured results are validated.

Tasks:
- Validate provider tool-call arguments against descriptor schemas.
- Validate structured results before persistence.
- Persist validation errors as tool errors.

Validation:
- Tests cover valid input, invalid input, invalid result, and schema mismatch.

### Step 4.9: Add Secret Redaction

Deliverable: Redaction runs before transcripts, audit log metadata, and recall index storage.

Tasks:
- Redact API keys, passwords, tokens, and environment variable values by pattern and configured rules.
- Record redaction counts in tool metadata.
- Ensure redacted values are not persisted in model-facing content.

Validation:
- Tests cover known secret formats, user-entered password placeholders, and redaction counts.

### Step 4.10: Add Structured Application Logging

Deliverable: JSON logs with configurable level and daily rotation target.

Tasks:
- Log server startup, shutdown, session lifecycle, provider failures, tool failures, approval decisions, and guardrail denials.
- Avoid logging secrets.
- Store logs in configured logs directory.

Validation:
- Tests or smoke checks verify log initialization and redaction behavior.

## Post-MVP: MCP And Subagents

### Step 5.1: Add MCP Crate And Types

Deliverable: `crates/mcp` with typed server config and tool descriptors.

Tasks:
- Define stdio and SSE server config.
- Define MCP tool mapping into unified `ToolDescriptor`.
- Defer OAuth and complex auth.

Validation:
- Unit tests cover config parsing and descriptor mapping.

### Step 5.2: Implement MCP Stdio Client

Deliverable: Connect to stdio MCP servers and list tools.

Tasks:
- Spawn configured command with timeout.
- Perform MCP initialization.
- Register tools with namespaced IDs.
- Surface connection and protocol errors.

Validation:
- Integration test uses a local fake MCP stdio server.

### Step 5.3: Implement MCP SSE Client

Deliverable: Connect to SSE MCP servers and list tools.

Tasks:
- Connect to configured endpoint.
- Perform initialization.
- Register tools with namespaced IDs.
- Surface transport errors.

Validation:
- Integration test uses a local fake SSE MCP server.

### Step 5.4: Merge MCP Tools Into Registry

Deliverable: MCP tools are available through the same registry as built-ins.

Tasks:
- Register MCP tools per session.
- Reject name collisions with `ToolConflict` events.
- Use the same approval, timeout, cancellation, and result envelope path.

Validation:
- Tests cover collision, successful registration, failed registration, and session isolation.

### Step 5.5: Implement MCP Tool Execution

Deliverable: Agent loop can execute MCP tools through the unified contract.

Tasks:
- Convert typed tool call arguments into MCP calls.
- Convert MCP responses into `ToolResult`.
- Apply redaction and schema validation.

Validation:
- Integration test executes a fake MCP tool and persists result events.

### Step 5.6: Implement Child Worktree Creation

Deliverable: Subagents get isolated child worktrees.

Tasks:
- Create child worktree under parent worktree scope.
- Persist parent-child relationship.
- Enforce child write boundary.

Validation:
- Integration test verifies child cannot write outside child worktree.

### Step 5.7: Implement `delegate` Tool

Deliverable: Parent sessions can start child agent loops.

Tasks:
- Add `delegate_id` idempotency key.
- Enforce `subagent_max_parallel` from config.
- Block recursive delegate in children.
- Return cached result for repeated `delegate_id`.

Validation:
- Tests cover successful delegate, duplicate delegate ID, concurrency limit, and child delegate block.

### Step 5.8: Restrict Subagent Toolset

Deliverable: Children receive only allowed or restricted tools.

Tasks:
- Allow read, glob, grep, session search, edit, write, and apply patch.
- Restrict bash to non-interactive mode.
- Block delegate, skills.write, interactive PTY, and configured parent-only tools.

Validation:
- Tests cover child registry output and restricted bash behavior.

### Step 5.9: Implement Subagent Result Review

Deliverable: Subagent completion presents a diff summary before merge by default.

Tasks:
- Commit child worktree changes.
- Generate diff summary.
- Request user approval before merge unless policy enables auto-merge.

Validation:
- Integration test verifies review-required path and approved merge path.

### Step 5.10: Implement Auto-Merge Path

Deliverable: Approved subagent results merge into parent worktree.

Tasks:
- Cherry-pick or fast-forward child commit into parent.
- Pause on conflicts and surface resolution instructions.
- Persist merge completed or merge conflict events.

Validation:
- Integration tests cover clean merge and conflict pause.

## Phase 6: Polish And Distribution

### Step 6.1: Add Configuration Documentation

Deliverable: Document supported config keys and defaults.

Tasks:
- Document `[tools]`, `[budgets]`, provider config, log config, data directories, and MCP config if the post-MVP phase is complete.
- Include a minimal example config.

Validation:
- Docs match config loader tests.

### Step 6.2: Add Install Script

Deliverable: Local install script for supported Unix-like systems.

Tasks:
- Build release binary.
- Install binary to a user-selected path.
- Check for system `git` dependency.
- Fail visibly on unsupported environments.

Validation:
- Manual test in a clean temporary environment.

### Step 6.3: Add Docker Image

Deliverable: Minimal Docker image for server use.

Tasks:
- Add multi-stage Dockerfile.
- Include system `git`.
- Run as non-root user.
- Do not require privileged container mode.

Validation:
- `docker build .`
- Container starts and `/health` responds.

### Step 6.4: Add Benchmarks

Deliverable: Basic startup, idle memory, and multi-session benchmark scripts.

Tasks:
- Measure server cold start.
- Measure idle RSS.
- Measure per-session overhead.
- Document commands and expected target thresholds from the spec.

Validation:
- Benchmark script runs locally and records results without committing generated artifacts.

### Step 6.5: Add Release Checklist

Deliverable: Repeatable pre-release checklist.

Tasks:
- Include formatting, linting, tests, docs review, benchmark run, and smoke test.
- Include manual web UI and WebSocket checks.

Validation:
- Checklist can be followed from a fresh clone.

## Post-MVP Tracks

### Track A: Optional Code Intelligence

Deliverable: Local-only structural code index.

Steps:
- Add `code_index` crate.
- Store content hashes for indexed files.
- Add tree-sitter extraction for selected languages.
- Add SQLite FTS5 symbol and file summary tables.
- Add `code_explore` tool.
- Add stale-entry rejection when content hash changes.

Validation:
- Tests prove unchanged files are reused and changed files are re-indexed.
- Missing parser support degrades visibly.

### Track B: Optional LSP Diagnostics

Deliverable: Advisory post-write diagnostics.

Steps:
- Capture baseline diagnostics.
- Apply edit.
- Query diagnostics.
- Report only newly introduced semantic errors.
- Surface missing or crashed language servers visibly.

Validation:
- Integration test with a local language server where practical.
- Fallback to lint/build validation remains explicit.

### Track C: Loop Primitives

Deliverable: Architecture support for scheduling, goals, evaluators, state files, and budgets.

Steps:
- Add scheduled session origin type.
- Add stop-condition callback to agent loop.
- Add evaluator/reviewer agent mode.
- Add `./state/*.md` support separate from recall.
- Implement `[budgets]` token caps and circuit breakers.

Validation:
- Tests prove normal conversational sessions and non-conversational sessions share the same event model.

### Track D: Optional Telegram Gateway

Deliverable: Thin gateway to existing WebSocket server.

Steps:
- Define measurable mobile UX gap before starting.
- Proxy Telegram messages to WebSocket session messages.
- Reuse provider-agnostic transcription only if browser dictation is insufficient.
- Keep gateway separate from core server.

Validation:
- Gateway does not add core session logic.

## Cross-Phase Reference Consultation

Before implementing any step, consult the relevant source code from **OMP (Oh My Pi)** and **Hermes Agent** — the two closest projects to Vulcan's envisioned architecture — for existing solutions, edge cases, and design patterns. Each step's tasks note which project areas to inspect. File-specific paths are given where known; if a path has changed, search the project's source tree for the equivalent module. **OpenCode** may also be consulted where Rust-specific patterns or SQLite event store design are relevant.

Reasons to consult:
- **OMP (Oh My Pi)** — closest architectural match for agent loop, subagent orchestration, content-anchored edits, LSP/DAP integration, model-agnostic routing, and streaming protocol design. Note: OMP is TypeScript with a ~55k line Rust core for in-process tools — not pure Rust. Its Rust crates (shell, grep, AST, PTY) are primary references for in-process tool design.
- **Hermes Agent** — closest match for approval gating, session search, subagent delegation, shell handling, session recall, and Telegram gateway. Also a catalog of anti-patterns (feature bloat, premature optimization).
- **OpenCode** — secondary reference for Rust-specific patterns: SQLite event store, session projector, provider abstraction, tool registry, skills format, and slash command dispatch.

Document notable findings (both adopted patterns and consciously rejected alternatives) in the implementation description for each step.

## Cross-Phase Validation Commands

Run these before merging any implementation phase:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo check --workspace
```

Add phase-specific integration tests as each boundary becomes real:

- Storage: migration and replay tests.
- Worktrees: temporary git repository tests.
- Server: health and WebSocket tests.
- Agent loop: fake provider tests.
- Tools: temporary worktree tests.
- Web UI: manual desktop and mobile smoke tests until browser automation is added.
- MCP: fake stdio and SSE server tests.

## Architecture Decisions To Preserve

- The server is the source of truth.
- Sessions are event-sourced and persisted in SQLite.
- Every session runs in a git worktree.
- Slash commands bypass the LLM.
- The tool registry is the only execution integration point.
- Guardrails inspect all tool calls and results through the same contract.
- Recall is read-only and tool-driven, never injected by default.
- Skills load full content only on demand.
- MCP and subagents are deferred until the core loop is stable.
- Post-MVP features must not force a session model rewrite.

## Open Decisions Before Phase 1 Completion

- Confirm final binary and crate prefix name: `vulcan-agent` or another name.
- Confirm whether OpenAI Responses API is deferred or added beside Chat Completions.

## Resolved Decisions (from PRD)

- Config format: **`config.toml` only** per PRD config surface.
- Project-local AGENTS.md: **supported in MVP** per Step 4.0.

## First Implementation Slice

The first practical slice should be:

1. Step 0.1: confirm baseline.
2. Step 0.2: create compiling workspace skeleton.
3. Step 1.1: define IDs and errors.
4. Step 1.2: define session and message types.
5. Step 1.3: define event model.
6. Step 1.6: add initial SQLite migrations.
7. Step 1.7: implement event store.
8. Step 1.8: implement session projector.

This slice proves the durable session core before provider, tools, worktrees, or UI are added.
