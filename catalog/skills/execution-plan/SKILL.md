---
name: execution-plan
version: 0.2.0
description: Use once the problem is understood and before executing changes agentically. Turns the understanding into a concrete, small, verifiable plan the developer approves before any code is touched. Includes a stop rule: after repeated failed attempts on the same step, halt and report what was tried, why, and how it failed so the developer can redirect. Invoke whenever a task involves modifying the project's code, even if it seems simple.
---

# Execution plan

You're the one who turns "I understand the problem" into "this is the path," so the developer can approve it at a glance before anything runs. The plan is a proposal, not an order: the developer is the orchestrator and has the last word.

## The plan must answer

1. **The steps**, in order, each as small as is useful. A step that can't be verified on its own is probably two steps.
2. **Which files** get touched. Lean on the project's navigation skill if it exists, instead of rediscovering the repo every time.
3. **How we'll know each step works**: which test approves it. If a step has no way to be tested, say so; it's usually a sign of soft design.
4. **Risks and future pain**: what can go wrong now, and what will be costly to maintain later. Flagging future pain early is cheap; discovering it in six months is not.

## Principles when planning

Choose the smallest change that satisfies the success criteria (KISS, YAGNI). Don't build for a future nobody asked for. If you see that a larger solution is worth it, offer it as an explicit alternative with its cost, and let the developer choose.

## After approval: execute end to end, but know when to stop

The developer approves the complete, detailed plan (or the card) once, with its important points laid out. That single approval is the green light to run from start to finish — you don't stop for per-step sign-off. The only thing that interrupts execution is the stop rule below.

The loop is: change, run the step's test, and if it isn't green, fix and run again. But runs are counted. **An attempt is one execution of the step against its test.** A run that passes ends the step; a run that errors or fails the test is a spent attempt; fixing the code and running again is the next attempt. After about three attempts on the same step without reaching green (unless the developer set another threshold), stop. Don't keep grinding: an agent that loops forever burns tokens, pollutes the context, and quietly erodes trust.

Not every failure ends at the stop rule. When a step fails and you resolve it within the limit, that's a learning too: recognize what the error was and what fixed it, and record it (see below), so the repo doesn't stumble over the same thing twice. The hard iterations are worth keeping whether they end in a fix or in a redirect.

When you stop, hand the developer a clear report:

- **What** you tried, attempt by attempt.
- **Why** you chose each approach.
- **How** it failed, with the actual error.

Then give control back so the developer can redirect or change the approach entirely. Example: connecting to a database and saving transactions fails three times. Rather than a fourth blind attempt, you surface the three tries and why each failed, and the developer proposes a different path.

That redirection is itself a learning, like the fixes resolved above. To record any of them, the handoff is the working tree, not a conversation: edit the relevant skill in `.agents/skills/` with what you learned, then tell the developer to run `alf plearn` to reconcile it into the project. If the learning generalizes beyond this repo, tell them to run `alf glearn <skill>` to promote it to the shared catalog. You make the edit and flag it; the developer approves — they own both.

## A good signal

A good plan is one the developer can approve, reject, or edit by reading it once. If they have to ask you to explain it, it's still too big or too vague.
