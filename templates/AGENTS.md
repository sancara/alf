<!--
  Scaffolded by alf. This is the canonical, tool-agnostic instruction file for this repo.
  FIRST TASK in this repo: complete every <...> section by reading the codebase, propose
  the result to the developer, and on approval remove this comment. Don't auto-fill and
  move on — the developer signs off (see "How we work here").
  As you complete each section, delete that section's <!-- ... --> guide comment; once
  the section is filled, the comment is just clutter.
  Keep this file command-first and short. Judgment lives in .agents/skills/, not here.
-->

# AGENTS.md

## Project
<one line: what this repo is and who it's for>

## Stack
<languages, frameworks, key libraries, and the versions that actually matter>

## Commands
<!-- Exact invocations with flags. Agents run these literally; vague commands cause failures. -->
- Setup: `<install dependencies>`
- Test (full): `<run the whole test battery>`
- Test (single): `<run one test or file>`
- Lint: `<lint command>`
- Build: `<build command>`
- Run / dev: `<start locally>`

## Project map
<!-- Directories mapped to responsibilities. This is what points agents at the files that matter. -->
- `<dir>/` — <responsibility>
- `<dir>/` — <responsibility>

## Conventions
<!-- Only what differs from the language's defaults. Skip the obvious. -->
- <e.g. "Errors: return a Result, never panic outside tests">
- <e.g. "Named exports only">

## Boundaries — do not touch
- Never commit secrets or `.env`; credentials come from the environment, not the repo.
- <generated dirs, frozen modules, vendored code, anything off-limits>

## Definition of done
A change is done only when its test battery is green (`<test command>`). Tests are proposed alongside the code and approved by the developer. No green, not done.

## How we work here
This repo uses spec-driven development. The standing rule: **the agent proposes, the developer decides.** Always.

The loop for any non-trivial task, using the skills in `.agents/skills/`:

1. **Understand** — `understand-the-problem`. Establish what / why / success criteria / constraints before any code. Produce a shaped card (Shape Up) and confirm it with the developer.
2. **Plan** — `execution-plan`. Turn the card into a small, verifiable plan where each step names the test that approves it. The developer approves the full plan once; that single approval authorizes end-to-end execution.
3. **Execute** — change, run the step's test, fix, rerun. An attempt is one execution of the step against its test. After about three attempts on the same step without reaching green (threshold in `.alf/config.toml`), stop and report what was tried, why, and how it failed, so the developer can redirect.
4. **Review** — `quality-reviewer`, plus any relevant vertical skill. Clean, readable, performant code; tests green.

## Learning
Don't let hard iterations evaporate. The handoff is the working tree: you edit the skill, the developer runs the command.

- A failure you resolved, or a redirect from the developer → edit the relevant skill in `.agents/skills/`, then tell the developer to run `alf plearn` to reconcile it into the project.
- A learning that generalizes beyond this repo → tell them to run `alf glearn <skill>` to promote it to the shared catalog.

You make the edit and flag it; the developer approves before anything lands.

## Skills available
Invoke index. If a listed skill isn't present in `.agents/skills/`, it wasn't installed for this repo — skip it.

- `understand-the-problem` — at the start of any implement / fix / design task.
- `execution-plan` — before modifying code.
- `quality-reviewer` — whenever code is written or changed.
- `frontend-expert` — when the task touches the UI.
- `backend-expert` — when it touches server logic, data, or APIs.
- `devops-expert` — when it touches build, deploy, infra, or config.
- `security-expert` — when it touches auth, untrusted input, or sensitive data.
