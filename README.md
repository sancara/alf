# alf

**0 friction AI Spec Driven Development toolkit.**

`alf` scaffolds a spec-driven, agentic development environment in seconds — without touching the repo you're working in. Every file it creates lives outside the codebase, so `git status` stays clean and your team's policies stay intact.

## The problem it solves

You join a company on day one. You want to work with spec-driven development and AI coding agents. Setting it up from scratch — skills, context files, tool configuration — takes hours and breaks every time you switch repos or machines.

`alf` reduces that to three commands.

## How it works

`alf` is a **scaffolder, not a runtime**. It never calls a model. The agentic workflow runs afterward, in whatever tool you use (Claude Code, Cursor, Gemini Antigravity, Copilot, Codex), guided by the skills alf installed.

```
~/.config/alf/
  catalog/                    ← your personal skill catalog (git repo)
    skills/
      understand-the-problem/
      execution-plan/
      quality-reviewer/
      frontend-expert/
      backend-expert/
      devops-expert/
      security-expert/
      codebase-navigator/

~/.claude/skills/             ← skills auto-discovered by Claude Code (all repos)

~/.config/alf/projects/
  <repo-hash>/                ← per-project state, outside the repo
    AGENTS.md                 ← canonical instructions (complete on first use)
    CLAUDE.local.md           ← Claude Code pointer
    skills/                   ← installed skills for this project
    manifest.toml             ← declared skills (intent)
    lock.toml                 ← resolved versions + hashes (generated)
    config.toml               ← project config (stop-rule threshold, etc.)

your-cloned-repo/             ← untouched — zero new files, clean git status
  .git/info/exclude           ← alf adds its patterns here (per-clone, never committed)
```

## Install

**macOS / Linux** — one command, no Rust required:

```sh
curl -sSfL https://raw.githubusercontent.com/sancara/alf/main/install.sh | sh
```

**Windows** — download `alf-windows-x86_64.exe` from
[Releases](https://github.com/sancara/alf/releases/latest) and add it to your PATH.

**From source** (requires Rust):

```sh
cargo install --git https://github.com/sancara/alf
```

## Quickstart

```sh
# 1. Create your personal catalog — seeds 8 built-in personas (once, per machine)
alf catalog init

# 2. Go to any repo — yours, a client's, a company's
cd any-repo
alf init

# 3. Install code intelligence (optional but recommended)
alf memory install
# → restart your agent and say "Index this project"

# 4. Open your agent (Claude Code, Cursor, Antigravity…)
# The personas are already in ~/.claude/skills/ — no setup needed.
# Your first task: complete AGENTS.md with the project's stack and commands.

# 5. After a hard iteration, capture what you learned:
alf plearn              # reconcile a local skill edit
alf glearn <skill>      # promote a generalizable learning to your catalog
```

## Commands

| Command | What it does |
|---------|-------------|
| `alf catalog init [--remote URL]` | Create your catalog (seeds 8 built-in personas) or clone an existing one |
| `alf init [path] [--with SKILL]` | Scaffold a repo: installs personas + detects verticals + writes .git/info/exclude |
| `alf add NAME` | Add a skill from the catalog to the project |
| `alf update [NAME]` | Update installed skills to the catalog's current version |
| `alf list` | List installed and available skills; flags local drift as `glearn` candidates |
| `alf memory install` | Install codebase-memory-mcp and configure it for this repo |
| `alf plearn` | Reconcile the agent's local skill edits into this project |
| `alf glearn NAME [--bump major\|minor\|patch] [--push]` | Promote a learning to the shared catalog |

Global flags: `--dry-run`, `--catalog PATH`.

## The personas

Eight skills ship in the catalog. Three are generic (always installed); four are vertical (added when `init` detects the relevant stack); one is for code intelligence.

**Generic — installed in every project:**
- `understand-the-problem` — establishes what/why/success criteria before any code; produces a Shape Up card; uses `get_architecture` when the knowledge graph is available
- `execution-plan` — turns understanding into an approved plan; stop rule after ~3 failed attempts; captures learnings via `plearn`/`glearn`
- `quality-reviewer` — clean code, DRY/KISS, performance, "no green tests = not done"

**Vertical — added based on detected stack:**
- `frontend-expert` — UI states, accessibility, perceived performance, design system consistency
- `backend-expert` — data modeling, API contracts, idempotency, consistency, observability
- `devops-expert` — reproducibility, safe deploys, observability, cost
- `security-expert` — defensive review: input handling, authz, least privilege, secrets

**Code intelligence:**
- `codebase-navigator` — guides the agent to use `codebase-memory-mcp`'s 14 MCP tools: `get_architecture`, `trace_call_path`, `detect_changes`, Cypher queries, and more — 99% fewer tokens than file-by-file exploration

## The learning loop

Skills are living documents. Every hard iteration is a learning opportunity.

```
agent edits .agents/skills/<name>/SKILL.md
    ↓
alf plearn          → reconciles the edit locally, refreshes ~/.claude/skills/
    ↓ (if it generalizes)
alf glearn <skill>  → bumps semver, commits to your catalog, re-tracks the project
    ↓
git push            → your catalog grows; any future project benefits
```

The `list` command flags skills whose installed content drifted from the lock hash — these are `glearn` candidates. The hash also detects tampering (instruction files are an attack surface).

## Zero repo pollution

`alf init` writes nothing to the repo. Everything lives in `~/.config/alf/`:

- Skills go to `~/.claude/skills/` (auto-discovered by Claude Code across all repos)
- Project state (manifest, lock, config, AGENTS.md) goes to `~/.config/alf/projects/<hash>/`
- `alf init` adds its patterns to `.git/info/exclude` — per-clone, never committed, invisible to teammates

Two developers on the same repo can have different skills and learnings without stepping on each other. The repo stays clean for everyone.

## Your catalog is yours

The catalog is your own git repo. It travels with you across companies and machines:

```sh
# New machine or new company
alf catalog init --remote git@github.com:you/my-catalog.git
cd any-repo
alf init
# → same personas, same learnings, same setup — in under a minute
```

## Code intelligence with codebase-memory-mcp

`alf memory install` installs [codebase-memory-mcp](https://github.com/DeusData/codebase-memory-mcp) — a high-performance MCP server that indexes your codebase into a persistent knowledge graph (155 languages, sub-ms queries, 99% fewer tokens than file-by-file search). It auto-configures Claude Code, Cursor, Antigravity, Codex, Windsurf, and more.

After installing, restart your agent and say **"Index this project"**. The `codebase-navigator` skill guides the agent on when to use the graph vs. when to read files directly.

All processing happens locally. Your code never leaves your machine.

## Build from source

```sh
git clone https://github.com/sancara/alf
cd alf
cargo build
cargo test
```

## License

MIT
