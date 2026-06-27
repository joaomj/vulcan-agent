# Vulcan Agent

> A local-first, Rust-based coding agent designed for speed, stability, portability, and extensibility.

Vulcan Agent is a self-hosted coding assistant that runs as a native Rust binary. It exposes a minimal web UI for desktop use and a Telegram bot for mobile use, both talking to a single HTTP/WebSocket server that owns sessions, providers, tools, memory, and guardrails.

The project is inspired by [OpenCode](https://github.com/anomalyco/opencode), [Codex](https://github.com/openai/codex), and [Hermes Agent](https://github.com/NousResearch/hermes-agent), but rebuilt with a smaller core, stricter performance budgets, and worktree-based isolation by default.

---

## Priorities

1. **Performance** — native Rust binary, no Bun/Node/Electron runtime.
   - Server cold-start < 1 s.
   - Idle server < 128 MB RSS.
   - Per-session overhead < 50 MB RSS.
2. **Stability** — event-sourced sessions, idempotent tools, append-only audit log, full worktree reset for undo, and config-driven timeouts.
3. **Portability** — browser + Telegram clients; runs locally and can be exposed via Tailscale or SSH tunnel; only runtime dependency is the system `git` binary.
4. **Extensibility** — MCP servers and on-demand skills extend the agent without bloating the core.

---

## Features

- **Web UI** — vanilla HTML/CSS/JS, no frontend framework.
- **Telegram bot** — mobile client with text and voice message support.
- **WebSocket-first transport** — bidirectional streaming for prompts, approvals, and subagent status.
- **Worktree isolation** — every session starts in a fresh git worktree; subagents get nested child worktrees.
- **Deterministic control** — slash commands (`/ask`, `/plan`, `/build`, `/debug`, `/undo`, `/model`, `/permissions`, etc.) bypass the LLM.
- **Unified tool registry** — built-in tools, MCP tools, and the `delegate` subagent tool share one contract.
- **Minimal memory & skills** — human-readable `memory.md` and `SKILL.md` loaded on demand.
- **Policy engine & secret redaction** — guardrails inspect every tool result uniformly.

---

## Architecture

```text
┌─────────────────────────────────────────────┐
│              Agent Server (Rust)            │
│  ┌─────────┐ ┌─────────┐ ┌───────────────┐  │
│  │ Sessions│ │Providers│ │  Tool Registry│  │
│  │         │ │         │ │ (built-in+MCP)│  │
│  └─────────┘ └─────────┘ └───────────────┘  │
│  ┌─────────┐ ┌─────────┐ ┌───────────────┐  │
│  │Worktrees│ │ Memory  │ │  Guardrails   │  │
│  │         │ │         │ │               │  │
│  └─────────┘ └─────────┘ └───────────────┘  │
└─────────────────────────────────────────────┘
              ↑↓ WebSocket / HTTP
┌─────────────────┬───────────────────────────┐
│   Web UI        │      Telegram Bot         │
│ (desktop)       │  (mobile + voice)         │
└─────────────────┴───────────────────────────┘
```

A more detailed architecture diagram is available in [`docs/architecture.html`](docs/architecture.html).

---

## Tech Stack

| Layer | Technology |
|---|---|
| Language | Rust |
| Async runtime | Tokio |
| Web server | Axum |
| WebSocket | Axum WebSocket / `tokio-tungstenite` |
| Database | SQLite via `sqlx` or `rusqlite` |
| Telegram bot | `teloxide` |
| Web UI | Vanilla HTML/CSS/JS |
| Git operations | Shell `git` via `tokio::process::Command` |
| Terminal/PTY | `portable-pty` / `tokio-pty` |
| Serialization | `serde` + `serde_json` |
| CLI args | `clap` |
| Config | `toml` / `jsonc` |

---

## Installation

> The project is in early development. A pre-built binary and install script will be provided in Phase 6.

### Build from source

```bash
git clone git@github.com:joaomj/vulcan-agent.git
cd vulcan-agent
cargo build --release
```

### Requirements

- Rust toolchain (latest stable)
- `git` installed and available on `$PATH`
- SQLite (usually bundled by the chosen Rust crate)

---

## Usage

Start the server against a project:

```bash
agent serve --project /path/to/project
```

Then open `http://localhost:8080` in your browser, or interact via the configured Telegram bot.

### Slash commands

| Command | Effect |
|---|---|
| `/ask` | Read-only consultative mode |
| `/plan` | Proposes a plan before acting |
| `/build` | Full tool access; can edit and run code |
| `/debug` | Diagnostic mode; reads and runs tests |
| `/new` | Start a new session |
| `/sessions` | List recent sessions |
| `/resume <id>` | Resume a session |
| `/undo` | Revert the last assistant turn |
| `/model <name>` | Switch model |
| `/permissions <mode>` | Change approval mode |
| `/status` | Show session info |
| `/clear` | Clear conversation context |
| `/memory` | Show or edit memory |

---

## Configuration

Configuration lives in `~/.agent/config.toml` (or a project-local `.agent/config.toml`).

```toml
[server]
host = "127.0.0.1"
port = 8080

[provider]
kind = "openai-compatible"
base_url = "https://api.openai.com/v1"
model = "gpt-4o-mini"
# api_key is read from OPENAI_API_KEY env var by default

[tools]
default_timeout_ms = 30_000
bash_timeout_ms = 120_000
delegate_timeout_ms = 600_000
subagent_max_parallel = 4
always_approve_hosts = ["localhost", "127.0.0.1"]

[budgets]
# Reserved for post-MVP loop primitives (token caps, circuit breakers).
```

---

## Project Status

The project is currently in **Phase 1: Foundation**.

| Phase | Focus |
|---|---|
| 1 — Foundation | Workspace, domain types, SQLite event store, generic OpenAI-compatible provider, built-in tools, HTTP/WebSocket server, CLI |
| 2 — Worktrees & Agent Loop | Git worktree per session, agent loop, worktree-scoped shell, per-turn undo |
| 3 — Clients | Web UI, Telegram bot, voice handling |
| 4 — Memory, Skills, Guardrails | `memory.md`, `SKILL.md`, policy engine, schema validation, secret redaction |
| 5 — MCP & Subagents | MCP client, `delegate` tool, restricted subagent toolset, auto-merge |
| 6 — Polish | Install script, Docker image, docs, benchmarks |

See [`docs/SPEC.md`](docs/SPEC.md) for the full specification.

---

## Why Rust?

- **No runtime bloat** — single native binary, no Node/Bun/Electron.
- **Predictable memory** — important for long-running sessions and subagents.
- **Portability** — cross-compile to Linux, macOS, and eventually Windows.
- **Tooling** — strong type system and async ecosystem (Tokio + Axum) for reliable server code.

---

## License

This project is licensed under the MIT License — see [`LICENSE`](LICENSE).

---

## Acknowledgments

- [OpenCode](https://github.com/anomalyco/opencode) — skills format, provider abstraction, event sourcing.
- [Hermes Agent](https://github.com/NousResearch/hermes-agent) — Telegram gateway, memory, and subagent patterns.
- [Codex](https://github.com/openai/codex) — slash-command UX and worktree usage.
