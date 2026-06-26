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
- sha2 0.10 — content hashing (lockfile + project identity)
- tempfile 3 — integration tests

## Commands
- Build: `cargo build`
- Test: `cargo test`
- Single test: `cargo test <test_name>`
- Lint: `cargo clippy --all-targets -- -D warnings`
- Format check: `cargo fmt --all -- --check`
- Install locally: `cargo install --path .`

## CI
Every push to `main` and every PR runs `.github/workflows/ci.yml`:
`fmt --check` → `clippy -D warnings` → `build` → `test`.
The release workflow (`.github/workflows/release.yml`) compiles binaries for
4 platforms and publishes a GitHub Release on every `v*` tag.

## Project map
- `src/lib.rs` — public surface of the library; no clap, no stdout
- `src/main.rs` — thin CLI layer (clap dispatch, printing); `main()` calls `run()` for clean error output
- `src/commands.rs` — one function per command; returns data, never prints;
  `install_and_track` is the shared helper used by `add` and `memory install`
  to copy a skill and register it in manifest + lock
- `src/catalog.rs` — discover and resolve skills from the catalog repo
- `src/skill.rs` — load a SKILL.md, parse frontmatter, compute content hash
- `src/project.rs` — resolve project root (canonicalized hash → `~/.config/alf/projects/<hash>/`);
  uses SHA-256 via sha2 for a stable, cross-version identifier
- `src/scaffold.rs` — copy skills into `~/.claude/skills/` and the alf project dir
- `src/gitexclude.rs` — write patterns to `.git/info/exclude` (never touches .gitignore)
- `src/manifest.rs` — `manifest.toml` (intent: declared skills)
- `src/lock.rs` — `lock.toml` (fact: resolved versions + hashes)
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
- `tests/cli.rs` — integration tests; one test per command (9 tests)

## Conventions
- The library (`src/lib.rs` and everything it exposes) has zero dependency on clap or stdout.
  All commands return structured data; `main.rs` formats and prints.
- Every filesystem write goes through `Fs` in `fsops.rs`. Never call `std::fs` directly
  in command logic — `Fs` honors `--dry-run` and records actions for inspection in tests.
- `install_and_track` in `commands.rs` is the single path for installing a skill into a
  project. Use it in any new command that installs skills — never call `scaffold::install_skill`
  directly from command functions.
- `AlfError` for library errors (thiserror), `anyhow` only at the CLI boundary in `main.rs`.
- No `unwrap()` or `expect()` outside of tests.
- All commands are idempotent and support `--dry-run`.
- Skills are flat in v1: a skill never depends on another skill.
- Clippy with `-D warnings` is enforced in CI. No new warnings.

## Boundaries — do not touch
- Never write to the host repo directory. All alf state lives in `~/.config/alf/`.
  The only exception is `.git/info/exclude` (per-clone, never committed).
- Never commit secrets or API keys anywhere. alf never calls a model.
- The `skills/` directory at the repo root is embedded at compile time via `seeds.rs`.
  Editing a skill there requires rebuilding the binary to take effect.

## Definition of done
A change is done only when `cargo test` passes green AND `cargo clippy --all-targets -- -D warnings` produces zero warnings AND `cargo fmt --all -- --check` passes.
New commands need an integration test in `tests/cli.rs`. Use `tempfile::tempdir()` for
all filesystem fixtures; set `HOME` and `XDG_CONFIG_HOME` to a tempdir to keep tests
hermetic. For `memory install` tests, use a fake `cbm` binary so tests don't depend on
the network.

## How we work here
This repo uses spec-driven development. The standing rule: **the agent proposes,
the developer decides.** Always.

The full workflow for any task — including tasks on alf itself:

1. `understand-the-problem` — what/why/success criteria/constraints; Shape Up card
2. `execution-plan` — concrete plan, each step names its test; developer approves once
3. Execute + tests green — stop rule applies (~3 attempts per step)
4. `quality-reviewer` — clean, performant, self-documenting code
5. Capture learnings — `alf plearn` (local) or `alf glearn` (catalog)

See `~/.claude/skills/` for the full skill personas.

## Current state (v0.5.0)
All commands implemented, tested (9 tests), CI green, clippy clean, fmt clean.

What's pending:
- `plearn`/`glearn` UX: interactive confirmation prompt (currently: use `--dry-run` to preview)
- Testing the 4 vertical personas in fire (approved by description, not yet run-tested)
- `cargo install` / crates.io publication
- `alf catalog init` without `--remote` should offer to connect a remote after seeding
- PR-based `glearn` workflow (open a PR instead of pushing directly to catalog main)
- Windows: verify `.git/info/exclude` path handling
- Refactor: inject paths as parameters instead of via `std::env::set_var` in tests
  (current approach is stable but not rigorous for large test suites)
