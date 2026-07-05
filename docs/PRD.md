# Vulcan Agent — Product Requirements Document

> **Status:** Draft
> **Date:** 2026-07-05
> **Replaces:** `docs/SPEC.md` as the product-defining document.

---

## 1. Problem Statement

Existing AI coding agents impose a runtime tax. OpenCode, OMP, and Pi run on TypeScript/Bun; Hermes runs on Python. All of them embed a second language runtime (Node, Python, Bun) that dominates startup time, memory, and binary size. The developer pays for this tax on every cold start, every tool invocation, every session resume.

Beyond the runtime tax, existing agents share a set of open pain points:

| Pain point | Observed in |
|---|---|
| Fragile session persistence — SQLite files scattered across the host, agent can wipe its own session data mid-task | OpenCode |
| Bloated system prompts that inflate every turn's context window | OpenCode |
| No persistent memory across sessions — agent starts blank on every new session | OpenCode, OMP (Pi-based) — OMP has Hindsight but it's not integrated | OpenCode |
| No synchronous voice dictation for mobile use | OpenCode, OMP |
| No mobile-oriented client — terminal is the primary interface | OpenCode (TUI primary), OMP (TUI primary) |
| Feature bloat in agents that support mobile/Telegram | Hermes |

## 2. Target User

**Primary user (MVP):** A single power developer — you — who builds local software projects, primarily greenfield. You want to prompt an agent from your desktop browser and from your phone while away from the machine. You currently use OpenCode but are frustrated by its runtime overhead, fragile persistence, lack of memory, and lack of mobile access. You considered Hermes for its Telegram client and self-improving loop, but found it too bloated and Python-based.

**Secondary users (post-MVP):** Other power developers who share the same frustrations and want a fast, Rust-native, web-first coding agent.

## 3. Value Proposition

Vulcan is a **pure Rust coding agent** — one language, one binary, no runtime dependencies — with a **web UI** as the universal client, **event-sourced session durability**, **worktree isolation**, **FTS5 session recall**, and **browser-based voice dictation**. Designed for a single power user who values speed, trust, remote access, and learning from history over feature breadth.

## 4. Target Experience

The user experience has four pillars:

1. **Frictionless flow** — Type a prompt, get fast responses. Cold start < 1s. No waiting for a JavaScript runtime to boot. Tool calls execute in-process (no fork/exec for search/grep). Round-trip latency is minimal.

2. **Trust and control** — Every session is worktree-isolated and event-sourced. Any turn can be undone via worktree reset. An append-only audit trail records every tool call, approval, and provider interaction. Approval gates prevent destructive actions unless explicitly allowed.

3. **Remote capability** — The web UI (PWA) is accessible from a phone browser via Tailscale/SSH tunnel. Desktop and mobile share the same interface. No app store, no desktop app install needed.

4. **Learning from history** — FTS5 session recall lets you search past sessions, replay traces, and inspect what the agent did. Project context files (AGENTS.md) shape every session. Skill descriptions are injected into context; full skill content is loaded on demand.

## 5. MVP Scope

### 5.1 In Scope

| Layer | MVP Capability |
|---|---|
| **Runtime** | Pure Rust, single binary, no Node/Bun/Python dependency |
| **Provider** | Single OpenAI-compatible protocol (covers OpenRouter, Groq, DeepSeek, Azure, Bedrock, etc.). API key via environment variable. |
| **Tools** | `read`, `write`, `edit`, `apply_patch`, `glob`, `grep`, `bash` (non-interactive). Typed input/output, configurable timeouts, worktree-scoped. |
| **Client** | Web UI (PWA): HTML/CSS/JS, responsive desktop+mobile, WebSocket to server, installable as PWA. |
| **Sessions** | Event-sourced SQLite store. Append-only event log. Projected session state. Per-session git worktree isolation. |
| **Undo** | Per-turn worktree snapshot and reset. |
| **Recall** | FTS5 session search (`session_search`) and trace replay (`session_open`, `run_trace`). Read-only, agent must actively query. |
| **Context** | Project-local `AGENTS.md` (or equivalent) loaded on session start. |
| **Skills** | `SKILL.md` discovery from configured directories. Name+description injected into system context. Full content loaded on-demand via `skill` tool call. |
| **Slash commands** | `/new`, `/sessions`, `/resume`, `/undo`, `/model`, `/permissions`, `/status`, `/clear`, `/recall`. Client-side dispatch, bypass LLM. |
| **Voice** | Browser Web Speech API dictation fills message input. No server-side transcription. |
| **Approval** | Tool approval classes: Never (read tools), OnMode (edits/bash), Always (interactive bash/delegate). Configurable approval mode per session. |
| **Config** | `config.toml` with tool timeouts, provider settings, allowed hosts, data/log directories. |
| **CLI** | `vulcan serve --project <path>` starts the server. |

### 5.2 Explicitly Deferred

| Capability | Rationale |
|---|---|
| Self-improving loop | Agent introspecting traces, creating skills, spotting inefficiencies. Requires stable MVP first. |
| LSP / code intelligence | In-process LSP diagnostics, symbols, rename. Deferred to post-MVP optional track. |
| MCP support | Model Context Protocol for external tool servers. Adds transport complexity. Deferred until core loop is validated. |
| Telegram / WhatsApp / Signal gateway | Web UI is the primary mobile client. Add a thin gateway only if a measurable mobile gap appears post-MVP. |
| Subagents / delegation | Child agent loops with isolated worktrees. Adds concurrency, merge, and review complexity. |
| Agent-curated memory | Agent writing facts, nudges, Honcho dialectic user modeling. Deferred to a dedicated memory track post-MVP. |
| Codebase indexing | Tree-sitter extraction, BM25/semantic hybrid index. Post-MVP optional track. |
| TUI / desktop app | Text terminal UI or Electron/Tauri desktop app. Not needed — web UI serves desktop and mobile. |
| Anthropic Messages protocol | Additional provider surface. Deferred until OpenAI-compatible route is stable. |
| Session forks / trees / JSONL storage | One session, one linear event stream in SQLite. Forks and trees complicate every projection. |

## 6. User Stories

### 6.1 Sessions

1. As a user, I start a new session via the web UI or CLI, and the server creates an isolated git worktree under my configured data directory.

2. As a user, I resume an existing session from the session list and continue where I left off, with the full message history restored.

3. As a user, I undo the last assistant turn and the worktree is reset to the pre-turn snapshot, restoring all files to their prior state.

4. As a user, I see the session's current model, approval mode, and project path in the status display.

### 6.2 Prompting and Tools

5. As a user, I type a message in the web UI and the agent responds by reasoning, calling tools (read, write, edit, grep, glob, bash), and streaming the results back in order.

6. As a user, the agent reads files inside the session worktree, with output size limits applied.

7. As a user, the agent writes new files and edits existing files inside the session worktree, with idempotent operations (no-op if content matches).

8. As a user, the agent searches file contents with regex (grep) and finds file paths by glob pattern.

9. As a user, the agent runs shell commands inside the session worktree with a configurable timeout, capturing stdout, stderr, and exit code.

### 6.3 Recall

10. As a user, I search past sessions with a natural language query via `/recall <query>` and see matching session titles, summaries, and snippets.

11. As a user, I open a past session's trace to see the full conversation replay including tool calls and results.

### 6.4 Skills

12. As a user, I place `SKILL.md` files in configured directories and the agent discovers and lists them.

13. As a user, the agent can load a skill's full content on demand when it needs expert guidance for a specific task.

### 6.5 Project Context

14. As a user, I place an `AGENTS.md` (or equivalent) in the project root and its content is injected into every session's system context.

### 6.6 Slash Commands

15. As a user, I type `/model groq/llama-4` to switch the provider and model mid-session without restarting.

16. As a user, I type `/permissions build` to allow all tool calls without per-call approval.

17. As a user, I type `/undo` to revert the last turn's file changes.

18. As a user, I type `/sessions` to list recent sessions with status and last message preview.

19. As a user, I type `/clear` to clear the conversation context while keeping the worktree intact.

### 6.7 Voice

20. As a user on a supported browser (desktop or mobile), I press a dictation button and my speech is transcribed into the message input via the Web Speech API.

### 6.8 Approval

21. As a user, I am prompted to approve or deny tool calls that match the current approval mode (e.g., write/edit on ask mode, bash on plan mode).

22. As a user, I select an approval mode that matches my current intent: /ask (read-only), /plan (proposes first, acts after), /build (full access).

## 7. Implementation Decisions

### 7.1 Language and Runtime

- Pure Rust for all server components. No Node, Bun, Python, or Electron.
- `tokio` async runtime.
- `axum` for HTTP and WebSocket.
- SQLite via `rusqlite` or `sqlx`.
- Shell `git` via `tokio::process::Command` (not `git2`).
- `clap` for CLI argument parsing.
- `serde` + `serde_json` for serialization.
- `thiserror` + `color-eyre` for error handling.

### 7.2 Architecture

- **Server is the source of truth.** All clients are thin. WebSocket is the primary communication channel.
- **Event sourcing.** Every mutation is an append-only event in SQLite. Session state is projected from events.
- **Worktree isolation.** Every session gets its own git worktree. Undo is a full worktree reset to a pre-turn snapshot.
- **Tool registry.** Built-in tools register at startup via a unified contract (typed descriptor, approval class, isolation level, result envelope). No tool returns a raw `serde_json::Value`.
- **Proxy model for providers.** A generic OpenAI-compatible chat completions route covers most providers. Anthropic Messages protocol deferred.
- **Skills are on-demand.** Only name and description are injected into system context. Full content loaded by tool call.
- **Recall is read-only and tool-driven.** Never injected by default. Agent must actively call `session_search`.

### 7.3 Project Layout

```
vulcan-agent/
├── Cargo.toml
├── crates/
│   ├── core/            # domain types, IDs, errors, session state machine
│   ├── storage/         # SQLite event store, migrations, FTS5 recall
│   ├── llm/             # provider trait + request/response types
│   ├── providers/       # OpenAI-compatible provider implementation
│   ├── tools/           # built-in tools (read, write, edit, glob, grep, bash, etc.)
│   ├── skills/          # SKILL.md loader + skill tool
│   ├── guardrails/      # approval engine, policy, redaction
│   ├── server/          # axum HTTP + WebSocket server
│   ├── web/             # static web UI assets
│   └── cli/             # clap entrypoint
└── docs/
    ├── PRD.md           # this file
    ├── ARCHITECTURE.md  # technical architecture (extracted from former SPEC.md)
    └── ADR/             # architecture decision records
```

### 7.4 Testing Strategy

- Unit tests for domain types, serialization, event projection, tool execution, and approval matrix.
- Integration tests for SQLite migrations, worktree operations, WebSocket protocol, and CLI.
- Fake provider in agent loop tests (no real API key required).
- Manual smoke tests for web UI on desktop and mobile browsers.
- Voice dictation tested manually in supported browsers.

### 7.5 Config Surface

```toml
[tools]
default_timeout_ms = 30_000
bash_timeout_ms = 120_000
always_approve_hosts = ["localhost", "127.0.0.1"]

[provider]
base_url = "https://api.openai.com/v1"
model = "gpt-4o"
api_key_env = "OPENAI_API_KEY"

[data_dir]
path = "~/.vulcan"

[logs]
level = "info"
dir = "~/.vulcan/logs"

[budgets]
# Reserved for post-MVP token caps and circuit breakers
```

## 8. Out of Scope (Explicit)

- Windows-specific client or installer (MVP targets macOS/Linux)
- Desktop application (Electron/Tauri)
- IDE extensions (VS Code, Zed)
- Cloud-hosted deployment
- OAuth provider authentication
- Complex vector/graph memory
- Autonomous skill authoring (`skills.write`)
- JSONL session storage, session forks, session trees
- Vector/semantic ranking over episodic traces
- OpenTelemetry or distributed tracing
- WASM plugin system

## 9. Success Criteria

MVP is achieved when the user (you) can dogfood Vulcan on a real greenfield project end-to-end, replacing OpenCode for that project. Concrete signals:

1. A session can be started from the web UI, the agent receives the project context, and the agent answers questions and performs coding tasks using read/write/edit/grep/glob/bash.

2. The web UI is usable from both desktop Chrome/Safari and mobile Safari/Chrome via Tailscale/SSH tunnel.

3. Any turn can be undone, reverting file changes.

4. Past sessions are searchable and replayable via `/recall`.

5. The user prefers Vulcan's speed and reliability over OpenCode for that project.

## 10. Further Notes

- The `docs/SPEC.md` file should be renamed to `docs/ARCHITECTURE.md` to house the technical spec, repo layout, tool contract details, and phase-level implementation notes.
- `PLAN-V1.md` remains the implementation plan and should be updated to reflect the PRD scope.
- Future ADRs should be recorded under `docs/adr/` when tradeoffs warrant explicit records.
- Reference consultation with OpenCode, OMP, and Hermes agent source code should continue during implementation — particularly:
  - **OpenCode** for SQLite event store patterns, session projector, worktree handling, and skill system
  - **OMP** for in-process Rust tool execution, content-hash edits, and stream rules
  - **Hermes** for approval gating, FTS5 session search, and multi-platform gateway architecture (post-MVP)

---

## Appendix A: Resolved Domain Terms

| Term | Definition |
|---|---|
| Session | A single conversation with the agent, running in its own git worktree, backed by an event-sourced SQLite log. |
| Worktree | A git worktree created for a session, providing file-level isolation. |
| Event sourcing | Append-only log of all state mutations (messages, tool calls, approvals), projected into current session state. |
| Skill | A `SKILL.md` file containing structured instructions for the agent. Only name+description injected by default. |
| Recall | FTS5-based search over past session content, surfaced via `session_search` and `session_open` tools. |
| Context file | A project-local `AGENTS.md` file whose content is injected into every session's system context. |
| Approval gate | A guardrail that resolves (tool.approval_class, session.approval_mode) into allow/deny/prompt. |
| Slash command | A client-side command (e.g., `/undo`, `/model`) that bypasses the LLM and maps to a deterministic server action. |
| Tool registry | The single integration point for all tools — built-in and future MCP tools — implementing a unified contract. |

---

## Appendix B: Reference Projects

| Project | Format | Stars | Primary Insight for Vulcan |
|---|---|---|---|
| OpenCode | TypeScript/Bun | 182k | Event-sourced SQLite store, worktree isolation, skill system, slash commands |
| Hermes Agent | Python | 209k | Self-improving loop (deferred), FTS5 recall, approval gates, Telegram gateway (deferred), trajectory compression |
| OMP (oh-my-pi) | TypeScript + Rust core | 16k | In-process Rust tools, content-hash edits, LSP/DAP (deferred), stream rules |
| Pi | TypeScript | ~1k | Slim core, maximum extensibility, minimal architecture |
