# alf

**0 friction AI Spec Driven Development toolkit.**

`alf init` turns any repo into a ready-to-orchestrate agentic environment in seconds:
an `AGENTS.md`, seven personas (generic + detected verticals) copied where every
supported tool reads them, and a manifest/lock that pins exact versions. Clone-and-go
on any OS, with Claude Code, Gemini Antigravity, Cursor, or Copilot — no post-clone
steps, no per-developer configuration.

`alf` is a **scaffolder, not a runtime**: it never calls a model. The spec-driven
workflow runs afterward, in whatever tool the developer uses, guided by the skills
alf installed. Your catalog is your own git repo; your learnings stay yours and travel
with you across companies and machines.

## Install

**macOS / Linux** — one command, no Rust required:

```sh
curl -sSfL https://raw.githubusercontent.com/sancara/alf/main/install.sh | sh
```

**Windows** — download `alf-windows-x86_64.exe` from
[Releases](https://github.com/sancara/alf/releases/latest) and put it in your PATH.

**From source** (requires Rust):

```sh
cargo install --git https://github.com/sancara/alf
```

## Quickstart

```sh
# 1. Create your personal catalog (once, on a new machine)
alf catalog init --remote git@github.com:you/alf-catalog.git

# 2. Scaffold any repo
cd my-project
alf init

# 3. Open your agent (Claude Code, Cursor, Copilot…) and complete AGENTS.md.
#    The agent reads the personas in .agents/skills/ and .claude/skills/.

# 4. After a hard iteration, record what you learned:
alf plearn              # capture a learning in this project's skills
alf glearn <skill>      # promote a generalizable learning to your catalog
```

## Commands

| Command | What it does |
|---------|-------------|
| `alf catalog init [path] [--remote URL]` | Create or clone your master catalog |
| `alf init [path] [--with SKILL]` | Scaffold a repo with generic personas + detected verticals |
| `alf add NAME` | Add a skill from the catalog to the project |
| `alf update [NAME]` | Update installed skills to the catalog's current version |
| `alf list` | List installed and available skills; flags local drift as `glearn` candidates |
| `alf plearn` | Reconcile the agent's local edits into this project's skills |
| `alf glearn NAME [--bump major\|minor\|patch] [--push]` | Promote a learning to the shared catalog |

Global flags: `--dry-run`, `--catalog PATH`.

## How it works

```
~/.config/alf/catalog/       ← your personal catalog (git repo)
  skills/
    understand-the-problem/SKILL.md
    execution-plan/SKILL.md
    quality-reviewer/SKILL.md
    frontend-expert/SKILL.md
    backend-expert/SKILL.md
    devops-expert/SKILL.md
    security-expert/SKILL.md

my-project/
  AGENTS.md                  ← canonical instructions (complete on first use)
  CLAUDE.md                  ← one-line pointer for Claude Code
  .agents/skills/            ← personas — read by Antigravity, Cursor, Copilot
  .claude/skills/            ← mirror  — read by Claude Code
  .alf/
    manifest.toml            ← declared skills (intent, hand-editable)
    lock.toml                ← resolved versions + hashes (generated)
    config.toml              ← project config (stop-rule threshold, etc.)
```

The learning loop: the agent edits a skill in `.agents/skills/` → you run
`alf plearn` to reconcile → if it generalizes, `alf glearn` promotes it to
your catalog with a semver bump and a git commit. Your catalog grows with you.

## The personas

Seven skills ship in the catalog. Three are generic (always installed); four are
vertical (added when `init` detects the relevant stack):

**Generic:**
- `understand-the-problem` — establishes what/why/success criteria before any code; produces a Shape Up card
- `execution-plan` — turns understanding into an approved plan; includes a stop rule after ~3 failed attempts
- `quality-reviewer` — clean code, DRY/KISS, performance, and "no green tests = not done"

**Vertical:**
- `frontend-expert` — states, accessibility, perceived performance, design system consistency
- `backend-expert` — data modeling, API contracts, idempotency, consistency, observability
- `devops-expert` — reproducibility, safe deploys, observability, cost
- `security-expert` — defensive review: input handling, authz, least privilege, secrets

## Build from source

```sh
git clone https://github.com/sancara/alf
cd alf
cargo build
cargo test
```

## License

MIT
