# alf

**0 friction AI Spec Driven Development toolkit.**

`alf` scaffolds a spec-driven, agentic development environment in seconds — without touching the repo you're working in. Every file it creates lives outside the codebase, so `git status` stays clean and your team's policies stay intact.

## The problem it solves

You join a company on day one. You want to work with spec-driven development and AI coding agents. Setting it up from scratch — skills, context files, tool configuration — takes hours and breaks every time you switch repos or machines.

`alf` reduces that to a handful of commands, run once.

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

## Full workflow — first day on a new repo

These steps are run **once per machine** (steps 1–2) and **once per repo** (steps 3–7).

```sh
# ── Once per machine ──────────────────────────────────────────────────────────

# 1. Create your personal catalog (seeds 8 built-in personas)
alf catalog init

# 2. Install code intelligence
alf memory install
# → installs codebase-memory-mcp and the codebase-navigator skill

# ── Once per repo ─────────────────────────────────────────────────────────────

# 3. Go to any repo — yours, a client's, a company's
cd any-repo

# 4. Scaffold the repo (detects stack, installs matching personas)
alf init

# 5. Open your agent (Claude Code, Cursor, Antigravity…)
#    Personas are already in ~/.claude/skills/ — no extra setup needed.

# 6. Build the knowledge graph
#    Say to your agent: "Index this project"
#    codebase-memory-mcp indexes the codebase in seconds.

# 7. Complete AGENTS.md — the most important step
#    Say to your agent:
#      "Read the AGENTS.md in ~/.config/alf/projects/ for this repo.
#       Complete all <...> sections using get_architecture to understand
#       the stack and structure. Propose the result for my approval."
#    The agent uses the graph to map the codebase and fills in the blanks.
#    You review and approve. This is the first exercise of the method:
#    the agent proposes, you decide.
```

After step 7, the full spec-driven loop is active for every subsequent task.

## The spec-driven loop (every task)

```
1. understand-the-problem
   → What / why / success criteria / constraints.
   → Produces a Shape Up card (ready to export to Jira/Trello).
   → Uses get_architecture to map the codebase before asking you anything structural.

2. execution-plan
   → Concrete, small, verifiable plan. Each step names its test.
   → You approve the full plan once — that authorizes end-to-end execution.
   → Stop rule: after ~3 failed attempts on the same step, agent stops and
     reports what was tried, why, and how it failed so you can redirect.

3. Execute + tests green
   → Agent runs, tests, fixes, reruns. No green = not done.

4. quality-reviewer
   → Clean code, DRY/KISS, performance, no magic numbers, self-documenting names.

5. Capture learnings (if anything hard was learned)
   → alf plearn    — local edit to this project's skill
   → alf glearn    — promote to your shared catalog (bumps semver, commits)
```

## Commands

| Command | What it does |
|---------|-------------|
| `alf catalog init [--remote URL]` | Create your catalog (seeds 8 personas) or clone an existing one |
| `alf init [path] [--with SKILL]` | Scaffold a repo: installs personas + detects verticals + writes .git/info/exclude |
| `alf add NAME` | Add a skill from the catalog to the project |
| `alf update [NAME]` | Update installed skills to the catalog's current version |
| `alf list` | List installed and available skills; flags local drift as `glearn` candidates |
| `alf memory install` | Install codebase-memory-mcp + codebase-navigator for this repo |
| `alf plearn` | Reconcile the agent's local skill edits into this project |
| `alf glearn NAME [--bump major\|minor\|patch] [--push]` | Promote a learning to the shared catalog |

Global flags: `--dry-run`, `--catalog PATH`.

## The personas

Eight skills ship in the catalog. Three are generic (always installed); four are
vertical (added when `init` detects the relevant stack); one for code intelligence.

**Generic — installed in every project:**
- `understand-the-problem` — what/why/success criteria before any code; Shape Up card; uses `get_architecture` when the graph is available
- `execution-plan` — approved plan with per-step tests; stop rule after ~3 failed attempts; feeds `plearn`/`glearn`
- `quality-reviewer` — clean code, DRY/KISS, performance, "no green = not done"

**Vertical — added based on detected stack:**
- `frontend-expert` — UI states, accessibility, perceived performance, design system consistency
- `backend-expert` — data modeling, API contracts, idempotency, consistency, observability
- `devops-expert` — reproducibility, safe deploys, observability, cost
- `security-expert` — defensive review: input handling, authz, least privilege, secrets

**Code intelligence:**
- `codebase-navigator` — installed by `alf memory install`; guides the agent to use
  `codebase-memory-mcp`'s 14 MCP tools (`get_architecture`, `trace_call_path`,
  `detect_changes`, Cypher queries) — 99% fewer tokens than file-by-file exploration

## The learning loop

Skills are living documents. Every hard iteration is a learning opportunity.

```
agent edits ~/.config/alf/projects/<hash>/skills/<name>/SKILL.md
    ↓
alf plearn    → reconciles the edit, refreshes ~/.claude/skills/
    ↓ (if it generalizes beyond this repo)
alf glearn    → bumps semver, commits to your catalog, re-tracks the project
    ↓
git push      → your catalog grows; every future project benefits
```

`alf list` flags skills whose installed content drifted from the lock hash —
these are `glearn` candidates. The hash also detects supply-chain tampering
(instruction files are an attack surface).

## Zero repo pollution

`alf init` writes nothing to the repo:

- Skills go to `~/.claude/skills/` (auto-discovered by Claude Code across all repos)
- Project state goes to `~/.config/alf/projects/<hash>/`
- `alf init` adds its patterns to `.git/info/exclude` — per-clone, never committed, invisible to teammates

Two developers on the same repo can have different skills and learnings without
stepping on each other. The repo stays clean for everyone.

## Your catalog is yours

```sh
# New machine, new company — same setup in under a minute
alf catalog init --remote git@github.com:you/my-catalog.git
cd any-repo
alf init
```

## Code intelligence with codebase-memory-mcp

`alf memory install` installs [codebase-memory-mcp](https://github.com/DeusData/codebase-memory-mcp) — a high-performance MCP server that indexes your codebase into a persistent knowledge graph (155 languages, sub-ms queries, 99% fewer tokens than file-by-file search). It auto-configures Claude Code, Cursor, Antigravity, Codex, Windsurf, and more. All processing is local; your code never leaves your machine.

## Build from source

```sh
git clone https://github.com/sancara/alf
cd alf
cargo build
cargo test
```

## License

MIT
