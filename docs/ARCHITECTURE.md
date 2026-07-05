# Vulcan Agent — Specification and Implementation Plan

> **Project name:** Vulcan Agent.
>
> This document is a specification-focused plan for a Rust-based coding agent inspired by OpenCode, but redesigned for speed, simplicity, and the specific workflows described in the design discussions. It is intended to guide implementation by AI agents and human developers.

---

## 1. Purpose

Build a local-first, Rust-based coding agent with:

- A web UI as the primary desktop and mobile client.
- A core HTTP/WebSocket server that owns sessions, providers, tools, session recall, and guardrails.
- Worktree-based workspace isolation by default.
- Deterministic user control via slash commands.
- MCP client support for extensibility.

The agent prioritizes **performance (speed), stability, portability, and extensibility** over visual design or feature breadth.

---

## 2. Reference Projects

The following projects were analyzed during the design phase and inform this specification.

### Direct inspirations

| Project | URL | Why it matters |
|---|---|---|
| **OpenCode** | https://github.com/anomalyco/opencode | Primary inspiration. Skills format, provider abstraction, event-sourced sessions, SQLite persistence, worktree support, and MCP integration are all derived from studying OpenCode. |
| **Hermes Agent** | https://github.com/NousResearch/hermes-agent | Reference for Telegram gateway design, voice transcription as a pluggable tool, sophisticated sudo/PTY shell handling, session search, LSP diagnostics, and subagent delegation. Also a warning against feature bloat. |
| **OMP (Oh My Pi)** | https://github.com/ohmypi/omp | Reference for agent loop design, hash-anchored content edits, subagent orchestration, LSP/DAP integration, model-agnostic role-based routing, and time-traveling stream rules. The most architecturally complete terminal agent in the ecosystem. |
| **Codex (OpenAI)** | https://github.com/openai/codex | Reference for slash-command UX (`/plan`, `/permissions`, `/model`), worktree usage in the Codex App, and the concept of deterministic user control shortcuts. |
| **Pi Agent** | https://github.com/earendil-works/pi | Reference for a minimalistic agent runtime. Validates the idea of a small core with pluggable packages, though Pi is TypeScript-based and too raw for direct adoption. |

### Rust alternatives considered

| Project | URL | Why it matters |
|---|---|---|
| **jcode** | https://github.com/1jehuang/jcode | Closest existing Rust implementation to the desired end-state (TUI + server + swarm + recall). TUI-first architecture and lack of native Telegram were gaps. |
| **claurst** | https://github.com/Kuberwastaken/claurst | Clean-room Rust reimplementation of Claude Code behavior, TUI-focused. |
| **VTCode** | https://github.com/vinhnx/VTCode | Rust coding agent with TUI, multi-provider support, MCP, and editor extensions. |
| **steer** | https://github.com/BrendanGraham14/steer | TUI-oriented Rust coding agent with gRPC server and MCP. |
| **cersei** | https://github.com/pacifio/cersei | Rust SDK for coding agents with an `abstract` CLI. |
| **OpenCodeRust** | https://github.com/ChrisFeldmeier/OpenCodeRust | Early scaffold claiming "OpenCode in 100% Rust" but immature (38 stars, 2 commits). Not a GitHub fork of OpenCode. |

### Key takeaway from the landscape

No mature Rust fork of OpenCode exists today. Several independent Rust agents come close to the desired feature set, but none match the exact combination of web-first UI, worktree-by-default, and minimal core architecture defined here.

---

### Implementation methodology

Every implementation step **must** consult the relevant source code of **OMP (Oh My Pi)** and **Hermes Agent** — the two closest projects to Vulcan's envisioned architecture — before writing code. **OpenCode** is a secondary reference for Rust-specific patterns. These projects have already solved — or notably failed at — the same problems Vulcan faces. Specific files to inspect are referenced in each implementation step of `PLAN-V1.md`. Findings (both adopted and consciously rejected) are recorded as part of each step's deliverable.

Reference URLs (ordered by relevance):
- OMP (Oh My Pi): https://github.com/ohmypi/omp
- Hermes Agent: https://github.com/NousResearch/hermes-agent
- OpenCode: https://github.com/anomalyco/opencode

## 3. Goals

1. **Speed and low memory footprint** — Rust native binary, no Bun/Node/Electron runtime.
   - Server cold-start < 1 s on a typical laptop.
   - Idle server baseline < 128 MB RSS.
   - Per-session overhead < 50 MB RSS.
2. **Stability** — event-sourced sessions, idempotent mutations, append-only audit log, full worktree reset for undo, and config-driven timeouts.
3. **Simplicity** — small invariant core; everything else is optional or external.
4. **Portability** — runs locally, accessed via browser (desktop and mobile); remote access via Tailscale/SSH tunnel; zero runtime dependencies beyond the system `git` binary.
5. **Deterministic control** — slash commands and policy engine reduce reliance on the LLM following rules.
6. **Extensibility** — MCP servers and skills provide modularity without bloating the core; built-in tools only in Phases 1–2, MCP deferred to post-MVP.

---

## 4. Non-Goals (MVP)

- Desktop application (Electron/Tauri).
- VS Code / Zed / IDE extensions.
- Windows-specific client or installer.
- Complex vector/graph memory system.
- File-based memory (`memory.md`, `user.md`) in MVP.
- OAuth provider authentication.
- Cloud-hosted deployment.
- Advanced browser automation in core.
- Complex design system or polished UI.
- Autonomous skill authoring (`skills.write` blocked in MVP).
- Vector/semantic ranking over episodic traces (FTS5 recall only in MVP).
- JSONL session storage, session forks, or user-visible session trees.

### Loop primitives (post-MVP)

The following are explicitly deferred but the MVP **must not preclude** them:

- **Scheduling / automations** — timer and event triggers. The session model must support non-conversational sessions (e.g., a session started by a cron trigger, not a user message). Avoid assuming "session = human typed something."
- **`/goal` primitive** — run until a condition is met, judged by a fresh model. The agent loop contract should accept a stop-condition callback, not just a max-turn limit.
- **Generator/evaluator split** — separate reviewer agent. The tool contract's `SubagentPolicy` flags already support this architecturally.
- **State files** — `./state/*.md` for cross-round progress tracking. Distinct from session recall. No MVP work needed beyond not blocking it.
- **Token budgets** — per-run and daily caps with circuit breakers. Config surface (`config.toml`) should have a `[budgets]` section even if empty in MVP.

---

## 5. Architecture

```text
┌─────────────────────────────────────────────┐
│              Agent Server (Rust)            │
│  ┌─────────┐ ┌─────────┐ ┌───────────────┐  │
│  │ Sessions│ │ Providers│ │  Tool Registry│  │
│  │         │ │         │ │ (built-in+MCP)│  │
│  └─────────┘ └─────────┘ └───────────────┘  │
│  ┌─────────┐ ┌─────────┐ ┌───────────────┐  │
│  │Worktrees│ │ Recall  │ │  Guardrails   │  │
│  │         │ │         │ │               │  │
│  └─────────┘ └─────────┘ └───────────────┘  │
└─────────────────────────────────────────────┘
              ↑↓ WebSocket / HTTP
┌─────────────────────────────────────────────┐
│               Web UI (PWA)                  │
│  (desktop + mobile via Tailscale / SSH      │
│   tunnel / LAN, Web Speech API,             │
│   installable as PWA)                       │
└─────────────────────────────────────────────┘
```

### Core principles

- The server is the source of truth. All clients are thin.
- Communication is WebSocket-only in the MVP (HTTP used only for health/static assets).
- Every session runs in its own git worktree.
- Events are append-only and persisted in SQLite.
- Tool registry merges built-in tools and MCP tools.
- Session recall and skills are minimal by design.

---

## 6. Design Decisions

### 6.1 Clients

- **Web UI (PWA)** — universal client. Works on desktop and mobile. Minimal HTML/CSS/JS, no frontend framework. PWA-installable for offline shell and push notifications. Mobile access via Tailscale, SSH tunnel, or LAN.
- **No TUI** — Ratatui dropped to avoid flickering, freezes, and rendering complexity.

### 6.2 Communication

- **WebSocket** between server and clients.
- Why not SSE: approval prompts, password input, and subagent status updates require bidirectional communication. SSE would force a hybrid SSE+HTTP model that is more complex overall.
- REST/HTTP endpoints reserved for health checks and static web assets.

### 6.3 Worktrees

- Every session starts in a fresh git worktree under `~/.vulcan/worktrees/<project>/<session-id>/`.
- Subagents get child worktrees nested under the parent's worktree.
- Undo is implemented by resetting the worktree to a snapshot taken before the last assistant turn (full worktree reset, not file-level revert).
- Auto-merge: when a subagent finishes, its worktree is committed and cherry-picked / fast-forwarded into the parent worktree. Conflicts pause for user resolution.
- **Git implementation:** Shell `git` invoked via `tokio::process::Command`, not `git2`. Worktrees are a core abstraction and the CLI implementation is more complete and less error-prone for nested worktrees, dirty-state edge cases, and snapshot/reset operations. The only runtime dependency added is the system `git` binary.

### 6.4 Session Recall

- MVP has no `memory.md`, `user.md`, automatic memory injection, or autonomous memory writes.
- The durable state mechanism is the SQLite event store: sessions, messages, tool calls, tool results, approvals, provider calls, token usage, errors, and redacted audit metadata.
- A read-only `session_search` tool provides FTS5 recall over prior sessions. It can search message content, tool names, tool inputs/outputs after redaction, errors, file paths, session titles, and summaries.
- A read-only `session_open` / `run_trace` tool retrieves selected surrounding turns or a complete trace for a session. This is the basis for agent self-debugging and user auditability.
- Session recall is never injected by default. The agent must actively ask for past context through tools, so stale memories do not silently shape every run.
- Vector/semantic ranking over prior sessions is deferred. If added later, it must remain optional and sit behind the same recall tool contract.
- A future `state` mechanism may store loop progress (findings, task status). It is separate from session recall and is not part of MVP.

### 6.5 Skills

- OpenCode-style `SKILL.md` format.
- Only skill names and descriptions are injected into the system context.
- Full skill content is loaded on demand via a `skill` tool call.
- This avoids context bloat and conflicting instructions.
- The skill loader ignores unknown frontmatter keys and sections, so future
  additions (e.g., a `## Stop` boundary) do not break existing skills.

### 6.6 Providers

- **Phase 1 provider:** generic OpenAI-compatible protocol for OpenRouter, Groq, DeepSeek, Azure, Bedrock (OpenAI-compatible endpoint), and others.
- Anthropic Messages protocol deferred to a later phase to keep the initial provider surface minimal and stable.
- Auth via API keys / environment variables. OAuth deferred.

### 6.7 Slash Commands

Modes are implemented as slash commands, not first-class agents:

| Command | Effect |
|---|---|
| `/ask` | Read-only consultative mode |
| `/plan` | Proposes plan before acting |
| `/build` | Full tool access, can edit/run code |
| `/debug` | Diagnostic mode, reads and runs tests |
| `/new` | Start new session |
| `/sessions` | List recent sessions |
| `/resume <id>` | Resume a session |
| `/undo` | Revert last assistant turn |
| `/model <name>` | Switch model |
| `/permissions <mode>` | Change approval mode |
| `/status` | Show session info |
| `/clear` | Clear conversation context |
| `/recall <query>` | Search prior sessions |

Slash commands are client-side controls that map directly to server actions. They bypass the LLM.

### 6.8 MCP

- MCP client supports `stdio` and `SSE` transports.
- MCP servers configured in config file (`mcp_servers`).
- MCP tools merged into the tool registry at runtime.
- MCP OAuth and complex auth deferred.

### 6.9 Shell / Interactive Commands

- Shell commands run inside the session worktree by default.
- If a command needs to run outside the worktree, the user is asked.
- PTY mode for interactive programs (`vim`, `python`, REPLs).
- Prompt mode for commands requiring input (`sudo`, `ssh` password).
- Secrets entered by the user are sent to the process stdin but redacted from the model transcript.

### 6.10 Dictation / Voice

- Transcription is **not** a core component.
- Web UI uses browser Web Speech API on desktop and mobile.
- A server-side `transcribe` tool is deferred; revisit only if Web Speech API coverage proves insufficient.

### 6.11 Browser Automation

- Not in core.
- Simple `web_fetch` / `web_search` built-in tools.
- Advanced browser automation via an external MCP server (e.g., Playwright MCP).

### 6.12 Logging

- **Audit log** — append-only, per-session, stored in SQLite. Records tool calls, approvals, provider calls. Secrets redacted.
- **Application logs** — structured JSON, configurable levels, rotated daily, stored in `~/.vulcan/logs/`.
- **Secret redaction** — API keys, passwords, tokens, env vars redacted by default.

### 6.13 Code Intelligence

- MVP code awareness uses `read`, `glob`, `grep`, project instructions, and explicit validation commands such as `cargo test`, `cargo check`, `cargo clippy`, `ruff`, or `eslint` when appropriate.
- Linters and build/test commands remain the authoritative validation layer. They are deterministic, project-owned, and should be run through the shell tool inside the session worktree.
- Post-MVP may add an optional local code index backed by SQLite, tree-sitter extraction, and FTS5. The first target is structural awareness: symbols, definitions, imports, call/reference edges where available, routes, tests, and file summaries.
- The code index should use content hashes or a Merkle-style file tree to re-index only changed files and to reuse index entries across worktrees when file content is identical.
- The index is local-only. No source code is uploaded for indexing by default.
- A future `code_explore` tool may expose the index through a small tool surface instead of many narrow graph tools.
- LSP support is optional and post-MVP. It complements, but does not replace, linters and build/test commands.
- Initial LSP scope should be post-write diagnostics: capture baseline diagnostics, apply the edit, query diagnostics, and report only newly introduced semantic errors. Later tools may expose definitions, references, workspace symbols, code actions, and rename.
- Missing or crashed language servers must degrade visibly to lint/build validation. LSP failures must not silently mark an edit as valid.

### 6.14 Tool Registry & Tool Contract

The tool registry is the single integration point for built-in tools, MCP tools, the `delegate` tool, and any future tool classes. Every tool — regardless of origin — implements the same contract, so the agent loop, guardrails, wire protocol, and persistence layer treat them uniformly.

#### 6.14.1 Why a unified contract

- The agent loop should not branch on "is this an MCP tool?" or "is this `delegate`?". One dispatch path, one result envelope, one approval hook.
- Subagent restrictions are expressed as flags on the tool, not as ad-hoc checks scattered through the loop.
- Guardrails inspect the result envelope uniformly for secret leakage, regardless of which tool produced it.

#### 6.14.2 Tool descriptor (registered, in-memory)

```rust
struct ToolDescriptor {
    id: ToolId,              // stable, namespaced: "builtin.read", "mcp.filesystem.write", "builtin.delegate"
    name: String,            // LLM-facing name (unique within a session)
    description: String,     // injected into system context (short)
    arg_schema: JsonSchema,   // JSON Schema for the tool call args
    result_schema: JsonSchema,// JSON Schema for the structured result
    approval: ApprovalClass, // gating before execution
    isolation: IsolationLevel,// where the tool runs
    cancellation: CancellationSupport,
    subagent_policy: SubagentPolicy, // allowed / blocked / restricted-for-children
    timeout: Duration,       // from config, never hardcoded
    source: ToolSource,      // builtin | mcp | delegate | skill
}
```

All `Duration` and threshold values come from the config module and `config.toml` - no inline literals in source (OC014 equivalent).

#### 6.14.3 Approval classes

| Class | Meaning | Default applies to |
|---|---|---|
| `Never` | No prompt, no audit-only entry needed beyond normal logging | `read`, `glob`, `grep`, `list_sessions`, `session_search`, `session_open` |
| `OnMode` | Gated by the session's current approval mode (ask/plan/build) | `edit`, `write`, `apply_patch`, `bash` (non-interactive) |
| `Always` | Always prompts the user regardless of mode | `bash` (interactive / PTY), `delegate`, `web_fetch` to non-allowlist host |
| `Blocked` | Tool is never offered to the model in this session | recursive `delegate` for children, `skills.write` in MVP, per-policy tools |

Approval mode is set via `/permissions` (§6.7) and stored on the session, but the *class* lives on the tool. The guardrail resolves `(tool.approval, session.approval_mode) → Decision`.

#### 6.14.4 Isolation levels

| Level | Runs in |
|---|---|
| `InProcess` | Server process (no shell). `read`, `glob`, `grep`, `session_search`, `session_open` |
| `Worktree` | Shell spawned inside the active session worktree. `bash`, `edit`, `write`, `apply_patch` |
| `ChildWorktree` | Shell inside a freshly created child worktree. `delegate` |
| `OutOfTree` | Runs outside the worktree; requires `Always` approval. `bash` for cross-repo ops |

Children inherit the parent's worktree as their root but may not write outside their own child worktree.

#### 6.14.5 Cancellation

- Every tool execution returns a `CancellationToken` to the agent loop.
- `CancellationSupport::Cooperative` — tool polls the token between I/O steps (`bash`, `delegate`, `web_fetch`).
- `CancellationSupport::Immediate` — pure in-process tools (`read`, `glob`) check once at start.
- `CancellationSupport::Unsupported` — only for blocking syscalls that cannot be interrupted; must be paired with `timeout` from config.
- Cancellation triggered by: user request, parent turn end, or timeout. On cancel the loop emits `ToolCallCancelled` and does not retry automatically.

#### 6.14.6 Result envelope

All tool results — success, error, or cancellation — use the same typed envelope. No tool returns a raw `serde_json::Value` as its result; structured payload conforms to `result_schema` (mirrors OC001: no raw dicts at API boundaries).

```rust
struct ToolResult {
    call_id: ToolCallId,      // ties back to the ToolCallRequested event
    ok: bool,                 // false on error or cancellation
    status: ToolStatus,      // Success | Error | Cancelled | Partial
    content: Vec<ContentBlock>, // model-facing output (text, image, etc.)
    structured: Option<serde_json::Value>, // conforms to result_schema; for programmatic callers
    error: Option<ToolError>, // present iff !ok
    metadata: ToolMetadata,  // wall_duration, tokens_estimate, worktree_commit_sha, redactions_applied
}
```

`metadata.redactions_applied` records how many fields were redacted by the guardrail, so the audit log and the user can see that a secret was scrubbed without seeing its value.

#### 6.14.7 Registration & merge order

- Built-in tools register at startup from the `tools` crate. Phases 1–2 use built-in tools only so the core can be validated before MCP integration.
- MCP tools register per session when MCP servers connect (§6.8), namespaced `mcp.<server>.<tool>`.
- Name collisions on registration fail the registration (not the session): the second tool is rejected with a `ToolConflict` event and the existing tool is logged. No silent shadowing.
- The registry exposes `list_for(session, is_subagent) -> Vec<ToolDescriptor>`, filtered by `subagent_policy` and session policy, used to build the LLM tool spec each turn.

#### 6.14.8 Subagent policy

| Policy | Meaning |
|---|---|
| `Allowed` | Available to parent and children |
| `ParentOnly` | Blocked for children, e.g. recursive `delegate`, `skills.write` (MVP) |
| `Restricted` | Available to children but with reduced arg surface — e.g. `bash` for children may not run interactive/PTY |
| `Blocked` | Never offered |

Children therefore inherit `read`, `glob`, `grep`, `session_search`, `edit`, `write`, `apply_patch` (with restricted bash); they do **not** inherit `delegate`, `skills.write`, interactive PTY, or tools marked `ParentOnly` by config.

#### 6.14.9 Idempotency expectations

- `edit` / `write` use upsert + content-hash pre-check (no-op if the file already matches). Safe to retry.
- `apply_patch` carries an idempotency key (the target blob SHA); a stale key fails with `PatchConflict` instead of corrupting the file.
- `delegate` carries a caller-issued `delegate_id`; a second delegate call with the same id returns the cached result instead of spawning a new subagent.
- None of these are "best effort"; the audit log records each attempt with its idempotency key.

#### 6.14.10 Config surface

All tunables in this section live in `config.toml` under `[tools]`:

```toml
[tools]
default_timeout_ms = 30_000          # default for Worktree/InProcess tools
bash_timeout_ms = 120_000            # interactive-capable shell
delegate_timeout_ms = 600_000        # child agent loop
subagent_max_parallel = 4            # simple constant; no min(cpus,N) logic
always_approve_hosts = ["localhost", "127.0.0.1"]  # web_fetch allowlist

[budgets]
# Reserved for post-MVP loop primitives (token caps, circuit breakers).
# Empty in MVP; config parser must accept this section.
```

`subagent_max_parallel` is a single integer with a constant default 4 — no runtime CPU detection. Override via config, not per-call. (Resolves the concurrency question from gap analysis: keep it simple.)

---

## 7. Feature Set

### Phase 1 — Foundation
- Cargo workspace scaffold.
- Core domain types (`Session`, `Message`, `ToolCall`, `Event`).
- SQLite event store and session projector.
- Provider abstraction with a single generic OpenAI-compatible route.
- Built-in tools only: `read`, `write`, `edit`, `apply_patch`, `glob`, `grep`, `bash`.
- Basic HTTP/WebSocket server.
- CLI: `agent serve --project <path>`.

### Phase 2 — Worktrees and Agent Loop
- Git worktree creation per session.
- Agent loop: prompt → LLM → tool calls → execution → stream results.
- Worktree-scoped shell execution.
- Per-turn undo via worktree snapshot reset.
- Event sourcing for all actions.

### Phase 3 — Clients
- PWA-friendly web UI (HTML/JS) with WebSocket, Web Speech API, responsive layout, PWA manifest, service worker.

### Phase 4 — Recall, Skills, Guardrails
- FTS5-backed `session_search` and `session_open` / `run_trace` tools.
- `SKILL.md` loader and on-demand `skill` tool.
- Policy engine: allowed/denied command patterns, required approvals.
- Tool-call JSON Schema validation.
- Secret redaction from transcripts.

### Post-MVP — Optional Code Intelligence
- Local SQLite/tree-sitter/FTS5 code index with content-hash incremental updates.
- `code_explore` tool for symbol and structural search.
- Optional LSP post-write diagnostics. Linters and build/test commands remain authoritative.

### Post-MVP — MCP and Subagents
- MCP client (`stdio` + `SSE`).
- `delegate` tool with child worktrees.
- Restricted subagent toolset.
- Auto-merge via git.

### Post-MVP — Optional Telegram Gateway
- A thin Telegram gateway that proxies to the existing WebSocket, adding zero-setup mobile access and native push notifications.
- Server-side `transcribe` tool re-uses the provider-agnostic transcription interface.
- Only build if the web UI proves insufficient for mobile use.

### Phase 6 — Polish
- Install script.
- Docker image.
- Configuration documentation.
- Basic benchmarks (startup, memory, multi-session).

---

## 8. Technical Stack

| Layer | Technology |
|---|---|
| Language | Rust |
| Async runtime | Tokio |
| Web server | Axum |
| WebSocket | `tokio-tungstenite` or Axum WebSocket |
| Database | SQLite via `sqlx` or `rusqlite` |
| Web UI (PWA) | Vanilla HTML/CSS/JS |
| Git operations | Shell `git` via `tokio::process::Command` |
| Terminal/PTY | `portable-pty` / `tokio-pty` |
| Serialization | `serde` + `serde_json` |
| Error handling | `thiserror` + `color-eyre` |
| CLI args | `clap` |
| Config | `toml` / `jsonc` |
| Optional code index | `tree-sitter` + SQLite FTS5 |
| Optional LSP | External language servers managed outside the MVP core |

---

## 9. Repository Layout

```text
vulcan-agent/
├── Cargo.toml
├── crates/
│   ├── core/            # domain, sessions, worktrees, agent loop
│   ├── storage/         # SQLite persistence
│   ├── llm/             # provider abstraction + protocols
│   ├── providers/       # Anthropic, OpenAI, OpenAI-compatible
│   ├── tools/           # built-in tools
│   ├── recall/          # session search + trace inspection
│   ├── code_index/      # optional local code intelligence index
│   ├── skills/          # SKILL.md loader + skill tool
│   ├── guardrails/      # policy engine + validation
│   ├── mcp/             # MCP client
│   ├── server/          # axum HTTP + WebSocket server
│   ├── web/             # static web UI assets (PWA manifest, service worker)
│   └── cli/             # clap entrypoint
└── docs/
    └── ARCHITECTURE.md  # this file
```

---

## 10. Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Provider API drift | Keep protocols thin; generic OpenAI-compatible covers most providers. |
| Git worktree edge cases | Test nested worktrees, dirty state, and submodules early. |
| LLM ignoring rules | Layer policy engine + validation + approval gates + slash commands. |
| MCP transport complexity | Start with stdio and SSE only; defer OAuth and streaming HTTP. |
| Scope creep | Strictly defer OAuth, vector memory, IDE extensions, desktop app, JSONL storage, and session forks. |
| Skill write-back without eval (Hermes anti-pattern) | `skills.write` hard-blocked in MVP; post-MVP requires an eval harness and explicit user approval. |
| Episodic recall perf at scale | FTS5 derived table owned by projector; no separate vector index in MVP. Bench before adding semantic ranking. |
| Code index staleness | Use content hashes or a Merkle-style file tree so changed files are re-indexed and stale entries are rejected. |
| LSP false confidence | LSP is advisory and optional. Linters, build commands, and tests remain authoritative. LSP failures must be visible. |
| Subagent resource exhaustion | Bounded by `subagent_max_parallel` config (§6.14.10); children get restricted toolset, no recursion. |
| Subagent context drift | Trajectory compression applied to children too; first/last N turns preserved. |
| Mobile UX gap | PWA + Tailscale + Web Speech API cover it. Re-evaluate post-MVP if mobile usage is a real need. |
| Architectural lock-out of loops | Session model, tool contract, and config must accommodate scheduling, evaluators, and budgets later. Mitigated by explicit invariants in §4. |

---

## 11. Open Questions (to resolve during implementation)

1. Definitive project name.
2. Exact config file format (`config.toml`, `config.jsonc`, or both).
3. ~~Whether to support `AGENTS.md` project-local instructions like OpenCode.~~ **Resolved: supported in MVP per PRD.**
4. ~~Whether subagents should auto-merge silently or present a diff summary first.~~ Resolved: diff-review-first by default; `--auto-merge` policy flag available.
5. Whether to support OpenAI Responses API in addition to Chat Completions.
6. (Resolved) Subagent concurrency: simple `subagent_max_parallel` constant in config (default 4). See §6.14.10.
7. (Resolved) Evaluator / verification subagent from Loop-Engineering paper: **deferred to post-MVP**. See §4 loop primitives.
8. (Resolved) Undo granularity: **full worktree reset** per §6.3.
9. (Resolved) Git operation strategy: **shell `git`** via `tokio::process::Command` per §8.
10. (Resolved) Phase 1 provider scope: **generic OpenAI-compatible only**; Anthropic deferred per §6.6.
11. (Resolved) Skill authoring in MVP: **`skills.write` hard-blocked** per §4 and §6.14.3.
12. (Resolved) MVP memory model: **no `memory.md` or `user.md`**; use SQLite-backed read-only session recall.
13. (Resolved) Session storage format: **SQLite only**; no JSONL, session forks, or user-visible session trees.
14. (Resolved) Telegram bot in MVP: **deferred to post-MVP**. The web UI is the universal client; mobile access via PWA + Tailscale. Add a thin Telegram gateway only if a measurable mobile gap appears.

---

## 12. Next Step

Begin **Phase 1: Foundation** by scaffolding the Cargo workspace and implementing the core domain types, SQLite event store, generic OpenAI-compatible provider, built-in tools, and a minimal HTTP/WebSocket server.
