# AGENTS.md

## Project
`alf` is a CLI tool written in Rust that scaffolds spec-driven, agentic development
environments. It installs skill personas and context files for AI coding agents
(Claude Code, Cursor, Gemini Antigravity, Copilot, Codex, Windsurf) without touching
the host repo. See README.md for the full picture.

## Stack
- Rust 2021, cargo 1.75+
- clap 4.4 (derive) — CLI layer
- serde + toml 0.8 — manifest/lock/config serialization
- anyhow 1 — error handling at CLI edges
- thiserror 1 — typed errors in the library core
- sha2 0.10 — content hashing for the lockfile
- tempfile 3 — integration tests

## Commands
- Build: `cargo build`
- Test: `cargo test`
- Single test: `cargo test <test_name>`
- Lint: `cargo clippy`
- Install locally: `cargo install --path .`

## Project map
- `src/lib.rs` — public surface of the library; no clap, no stdout
- `src/main.rs` — thin CLI layer (clap dispatch, printing); calls library functions
- `src/commands.rs` — one function per command; returns data, never prints
- `src/catalog.rs` — discover and resolve skills from the catalog repo
- `src/skill.rs` — load a SKILL.md, parse frontmatter, compute content hash
- `src/project.rs` — resolve project root (hash of repo path → `~/.config/alf/projects/<hash>/`)
- `src/scaffold.rs` — copy skills into `~/.claude/skills/` and the alf project dir
- `src/gitexclude.rs` — write patterns to `.git/info/exclude` (never touches .gitignore)
- `src/manifest.rs` — `.alf/manifest.toml` (intent: declared skills)
- `src/lock.rs` — `.alf/lock.toml` (fact: resolved versions + hashes)
- `src/config.rs` — project config + machine config; `config_home()` is public
- `src/detect.rs` — heuristic stack detection for vertical personas
- `src/seeds.rs` — 8 built-in skills embedded via `include_str!` at compile time
- `src/version.rs` — semver bump + frontmatter version rewriting for `glearn`
- `src/fsops.rs` — filesystem facade that honors `--dry-run`; records all actions
- `src/error.rs` — `AlfError` enum (thiserror)
- `templates/AGENTS.md` — the template `alf init` writes into each project
- `templates/CLAUDE.md` — the one-line Claude Code pointer
- `skills/` — the 8 built-in skill sources (embedded into the binary via seeds.rs)
- `catalog/skills/` — same skills, as the seed catalog for `alf catalog init`
- `tests/cli.rs` — integration tests; one test per command

## Conventions
- The library (`src/lib.rs` and everything it exposes) has zero dependency on clap or stdout.
  All commands return structured data; `main.rs` formats and prints.
- Every filesystem write goes through `Fs` in `fsops.rs`. Never call `std::fs` directly
  in command logic — `Fs` honors `--dry-run` and records actions for inspection in tests.
- `AlfError` for library errors (thiserror), `anyhow` only at the CLI boundary in `main.rs`.
- No `unwrap()` or `expect()` outside of tests.
- All commands are idempotent and support `--dry-run`.
- Skills are flat in v1: a skill never depends on another skill.

## Boundaries — do not touch
- Never write to the host repo directory. All alf state lives in `~/.config/alf/`.
  The only exception is `.git/info/exclude` (per-clone, never committed).
- Never commit secrets or API keys anywhere. alf never calls a model.
- The `skills/` directory at the repo root is embedded at compile time via `seeds.rs`.
  Editing a skill there requires rebuilding the binary to take effect.

## Definition of done
A change is done only when `cargo test` passes green. New commands need an
integration test in `tests/cli.rs`. Use `tempfile::tempdir()` for all filesystem
fixtures; never write to real home dirs in tests (use `std::env::set_var("HOME", ...)`).

## How we work here
This repo uses spec-driven development. The standing rule: **the agent proposes,
the developer decides.** Always.

For any non-trivial task: understand the problem first, produce a plan the developer
approves, execute with the stop rule (~3 attempts per step), and capture learnings.
See the skills in `~/.claude/skills/` or `~/.config/alf/projects/<hash>/skills/`
for the full personas.

## Current state (v0.4.0)
All mechanical commands are implemented and tested: `catalog init`, `init`, `add`,
`update`, `list`, `plearn`, `glearn`, `memory install`.

What's pending:
- `plearn`/`glearn` UX polish: interactive confirmation prompt (currently approve via --dry-run preview)
- Testing the 4 vertical personas in fire (approved by description, not tested in runs)
- `cargo install` / crates.io publication
- `alf catalog init` without --remote should offer to connect to a remote after seeding
- PR-based glearn workflow (open a PR instead of pushing directly to catalog main)
- Windows: verify `.git/info/exclude` path handling
