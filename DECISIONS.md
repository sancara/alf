# Design decisions

This document records the non-obvious design decisions made during alf's development,
and the reasoning behind them. It exists so that anyone picking up this project —
human or agent — understands not just *what* the code does but *why* it does it that way.

---

## alf is a scaffolder, not a runtime

**Decision:** alf never calls a model. It installs files and exits.

**Why:** The agentic workflow (understanding, planning, executing, reviewing) runs
inside the developer's chosen tool — Claude Code, Cursor, whatever. alf's job is
to provision that tool with the right skills and context before the session starts.
This keeps alf simple, deterministic, and provider-agnostic without building any
model abstraction layer. Provider-agnosticism is free because alf never talks to
a provider.

---

## The catalog is a git repo owned by the developer

**Decision:** The skill catalog lives at `~/.config/alf/catalog/` as a git repo
with the developer's own remote.

**Why:** Skills are living documents that improve over time. Git gives version
history, diffs, and a promotion path (`glearn` = a commit) for free. The developer
owns their catalog — it travels between companies and machines. There is no central
registry; the catalog is yours.

**Consequence:** `alf catalog init --remote <url>` clones it; without `--remote`
it creates a fresh repo seeded with the 8 built-in personas embedded in the binary.

---

## Everything lives outside the host repo

**Decision:** `alf init` writes nothing to the repo being scaffolded. All state
(skills, AGENTS.md, manifest, lock, config) lives in `~/.config/alf/projects/<hash>/`.
Skills also go to `~/.claude/skills/` for global Claude Code discovery.

**Why:** Repos belong to companies and clients. Adding files like `CLAUDE.md` or
`.agents/` to a repo you don't own can violate policies, create review noise, and
force tooling on teammates who didn't ask for it. The right place for a developer's
personal agentic setup is their own machine, not the shared codebase.

**Consequence:** Two developers on the same repo can have completely different skills
and learnings without conflict. The repo stays clean for everyone.

---

## .git/info/exclude instead of .gitignore

**Decision:** alf adds its patterns (CLAUDE.md, .agents/, .alf/, etc.) to
`.git/info/exclude`, not to `.gitignore`.

**Why:** `.gitignore` is a shared file committed to the repo — adding personal
tooling entries there forces them on every teammate and every clone. `.git/info/exclude`
is per-clone, never committed, and invisible to the rest of the team. It's the
correct mechanism for "things I happen to ignore on this machine."

**Reference:** Same syntax as `.gitignore`, lives in `.git/info/exclude`.

---

## Skills are copied, not symlinked

**Decision:** Skills are copied to `~/.claude/skills/` and to the alf project dir.
No symlinks.

**Why:** Symlinks created on Unix break on Windows git clones where `core.symlinks`
is often off. The breakage happens on the *consuming* machine (at clone time), not
on the machine that created the link — so OS detection at `alf init` time doesn't
help. Copying is KISS: files are small, the duplication is negligible, and everything
is git-diffable and self-contained.

---

## The lockfile hash serves two purposes

**Decision:** `lock.toml` stores a `sha256:...` hash per installed skill.

**Why — drift detection:** If a skill's installed content no longer matches its
locked hash, it has local modifications — exactly the candidate to promote with
`glearn`. The `list` command flags these as "modified — glearn candidate."

**Why — tamper detection:** Instruction files (SKILL.md, AGENTS.md) are an attack
surface. A malicious skill in a shared catalog could redirect agent behavior,
exfiltrate data, or inject backdoors. The hash makes tampering visible. Owning
your own catalog repo is the primary defense; the hash is the second layer.

---

## plearn / glearn: two scales of learning

**Decision:** Two separate commands for local vs. global learnings.

- `plearn` — captures a learning in *this project's* skills. Scope: one repo.
- `glearn` — promotes a generalizable learning to the *shared catalog*. Scope: all future projects.

**Why:** Not every learning generalizes. "This repo uses an in-memory SQLite fixture
for tests" belongs in the project skill, not the catalog. "Never skip the stop rule"
belongs in the catalog. Merging them would either pollute the catalog with
project-specific noise or lose project-specific context.

**The handoff is the working tree:** the agent edits the skill file directly in
`~/.config/alf/projects/<hash>/skills/<name>/SKILL.md`, then tells the developer
to run `alf plearn` or `alf glearn`. alf reconciles; it doesn't draft. The agent
is the author; alf is the bookkeeper.

---

## glearn bumps semver minor by default

**Decision:** `alf glearn <skill>` bumps the minor version (e.g. 0.1.0 → 0.2.0)
by default. Overridable with `--bump major|minor|patch` or `--set-version`.

**Why:** Minor bumps signal "new capability, backward compatible" — which is what
most learnings are. The developer can override when a learning is a breaking change
(major) or a tiny correction (patch). The default is a proposal; the developer decides.

---

## Vertical personas are detected, not mandated

**Decision:** `alf init` uses heuristic file detection to choose which vertical
personas to install (frontend, backend, devops, security). Detection failures are
silent skips, not errors. Explicit `--with <skill>` always wins.

**Why:** Detection is a guess. A repo with `Cargo.toml` is probably a backend — but
it could be a CLI tool that doesn't need `backend-expert`. A wrong detection wastes
tokens; a missing detection is easy to fix with `alf add`. The heuristic should be
helpful, not mandatory. The rule: required skills (generics + `--with`) fail loudly
if missing; detected skills skip silently.

---

## codebase-memory-mcp is integrated, not reimplemented

**Decision:** `alf memory install` invokes `codebase-memory-mcp`'s own installer
rather than reimplementing code graph indexing.

**Why:** `codebase-memory-mcp` has 155 tree-sitter grammars, LSP-style type resolution
for 4 languages, 14 MCP tools, and multi-agent configuration logic. Building something
equivalent would take years. The project is MIT-licensed, open source, single static
binary with zero runtime dependencies, SLSA Level 3, scanned by 72 antivirus engines,
and processes everything locally. The risk profile is acceptable and the value is high.
alf's job is orchestration, not reimplementation.

**The `codebase-navigator` skill** guides the agent on *when* to use the graph
(structural questions: call chains, architecture, impact) vs. when to read files
directly (understanding semantics, editing). It bridges alf's skill system with
codebase-memory-mcp's MCP tools.

---

## AGENTS.md is the canonical instruction file

**Decision:** The canonical, tool-agnostic instruction file is `AGENTS.md`.
Tool-specific files (CLAUDE.md, GEMINI.md, etc.) are either pointers or thin shims.

**Why:** AGENTS.md is the emerging cross-tool standard (OpenAI, now Linux Foundation
Agentic AI Foundation). Claude Code, Cursor, Codex, Antigravity, Copilot all read it
natively as of mid-2026. Writing one file that all tools understand beats maintaining
N per-tool files. CLAUDE.md exists only because Claude Code uses it as its primary
file — it's a one-liner that points to AGENTS.md.

---

## The `understand-the-problem` skill uses Shape Up framing

**Decision:** The output of `understand-the-problem` is a shaped card in Shape Up
format (Problem, Appetite, Solution, Rabbit holes, No-gos, Open questions).

**Why:** Shape Up separates "how much is this worth" (appetite) from "how long will
it take" (estimate). That inversion is exactly what prevents scope creep in agentic
sessions: you fix the time, flex the scope. The card doubles as a Jira/Trello ticket
— an artifact stakeholders can approve before code starts, which eliminates the most
common source of wasted agentic work (building the right thing in the wrong direction).

---

## The stop rule is ~3 attempts, configurable per project

**Decision:** `execution-plan` stops after ~3 failed attempts on the same step and
reports what was tried, why, and how it failed. The threshold is in `.alf/config.toml`
(defaults to 3).

**Why:** An agent that loops forever burns tokens, pollutes context, and quietly erodes
trust. Three attempts is enough to distinguish "I'm figuring it out" from "I'm stuck
and need a human redirect." The per-project config (`stop_after_attempts`) lets teams
tune this — a research spike might warrant 5; a routine CRUD endpoint, 2.

**What counts as an attempt:** One execution of the step against its test. A run that
errors or fails the test is a spent attempt. Fixing the code and re-running is the
next attempt. Superficially reframing the same approach to dodge the counter is not
a new attempt.
