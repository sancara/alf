---
name: codebase-navigator
version: 0.1.0
description: "Use when working in a repo that has codebase-memory-mcp installed. Guides the agent on when and how to use its 14 MCP tools to answer structural questions about the codebase — call chains, architecture, impact, dead code — without reading files one by one. Invoke at the start of any task that involves understanding, navigating, or changing existing code."
---

# Codebase navigator

You have access to a persistent knowledge graph of this codebase, built by
`codebase-memory-mcp`. It indexes functions, classes, HTTP routes, call chains,
and cross-service links across 155 languages, and answers queries in under 1ms.

Use it instead of reading files one by one. Five graph queries replace dozens
of grep/read cycles, at 99% fewer tokens.

## When to reach for the graph

**Start of every task** — before planning any change, call `get_architecture`.
It returns languages, packages, entry points, hotspots, and clusters in one
call. This is the starting point that replaces a manual tour of the repo.

**"What calls this?"** — `trace_call_path(function_name="X", direction="inbound")`.
Never grep for callers.

**"What does this call?"** — `trace_call_path(function_name="X", direction="outbound")`.

**"Where does this symbol live?"** — `search_graph(name_pattern=".*PartialName.*")`.
Find the exact qualified name before tracing.

**"What breaks if I change this?"** — `detect_changes` maps uncommitted diffs
to affected symbols with risk classification. Run this before proposing a plan
so the plan's risk section is grounded in evidence, not guesswork.

**"Is this code dead?"** — `search_graph` with degree filtering, or ask for
dead code detection explicitly.

**Cross-service questions** — `get_architecture` shows HTTP routes and cross-service
links. Follow up with `trace_call_path` to walk a request from entry point to
handler.

## How to use it well

Always call `get_graph_schema` first if you haven't indexed the project yet,
to confirm the graph is populated. If it returns zero nodes, tell the developer
to run `codebase-memory-mcp index_repository` (or just say "Index this project"
to your agent — it understands).

Use `search_graph` to discover exact qualified names before tracing. The
format is `<project>.<path_parts>.<name>` — searching first avoids
"zero results" on `trace_call_path`.

`query_graph` accepts Cypher-like queries for anything the named tools don't
cover: `MATCH (f:Function)-[:CALLS]->(g) WHERE f.name = 'processOrder' RETURN g.name`.

## What you leave to others

Code quality, tests, performance → `quality-reviewer`. Problem framing →
`understand-the-problem`. Step-by-step plan → `execution-plan`. The graph
answers *structural* questions; judgment about what to do with those answers
lives in the other skills.

## If the graph is stale

The background watcher keeps it fresh on every git change. If a symbol you
expect isn't there, call `index_repository` again. It's fast — an average
repo in milliseconds.
