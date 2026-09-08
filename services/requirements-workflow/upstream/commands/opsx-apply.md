---
name: /opsx-apply
id: opsx-apply
category: Workflow
description: Implement an OpenSpec change phase by phase (Experimental)
---

Implement an approved OpenSpec change, one delivery phase at a time.

**Input**: Optionally a change name and/or an implementation scope (e.g. `/opsx:apply add-auth BE`,
`/opsx:apply frontend`). If omitted, infer from conversation context. If vague or ambiguous you MUST
prompt for available changes and scope.

**The shape of this command in one paragraph.** `tasks.md` is the execution law. It is a queue of
`### P#` delivery phases, each an independent, human-approved increment carrying its own `Verify` plan,
`tier` and `Status`. You work the phases sequentially in `Depends on:` order, in THIS window — there is
no orchestrator mode and no implementation subagents. The per-phase procedure is not restated here: it
lives in that file's `## Phase Processing Rules` block, and you execute it literally.

## Preflight: consult project-context (sync is not forced here)

`/project-sync` is **not** a mandatory preflight for apply — it runs at `/opsx:propose` and `/opsx:plan`,
the phases that precede implementation. If you have not synced this session, or suspect the tooling,
schema, or specs are stale, run `/project-sync` manually first; otherwise proceed on the current
checkout.

While implementing, **consult project-context** (the `project-context` skill) whenever you need product
intent, prior decisions, or integration details the tasks/specs reference but don't fully spell out —
don't guess from memory when the answer is retrievable.

**Steps**

1. **Select the change**

   If a name is provided, use it. Otherwise:
   - Infer from conversation context if the user mentioned a change
   - Auto-select if only one active change exists
   - If ambiguous, run `openspec list --json` and use the **AskUserQuestion tool** to let the user select

   Always announce: "Using change: <name>" and how to override (e.g. `/opsx:apply <other>`).

2. **Read status and apply instructions**

   ```bash
   openspec status --change "<name>" --json
   openspec instructions apply --change "<name>" --json
   ```

   From `status`, take `schemaName` and which artifact holds the tasks. From `instructions apply`, take
   the context file paths, the progress counts, the task list, and the schema's own apply instruction —
   that instruction is the schema-level contract for this stage; follow it.

   **Handle states:**
   - `state: "blocked"` (missing artifacts): show the message and suggest the command that owns them —
     `/opsx:plan` (for `technical-design` / `tasks`) or `/opsx:propose` (for missing BA/SA artifacts).
   - `state: "all_done"`: congratulate, suggest archive.
   - Otherwise: proceed.

3. **Entry gate — is the plan approved?**

   Apply implements decided work only. Verify all three, and BLOCK with the exact open items if any fails:
   - `tasks.md` `## Review & Approval` reads approved for implementation, with a NAMED human in
     `Approved by` — never `Agent`, `LLM`, `Cursor`, or blank. If it still reads no, the queue was never
     signed off: run `/opsx:plan` and complete its review gate first.
   - `technical-design.md` "Open Questions": every CRITICAL question is APPROVED or DEFERRED-APPROVED by a
     named human.
   - `grooming.md` (when present): no OPEN/BLOCKED items, no unchecked readiness boxes, no missing or
     invalid approver.

   Do not implement against unapproved requirements or undecided technical questions.

4. **Read the queue and the context**

   Read `tasks.md` in full — it is short by design and you need its shape before planning anything else:

   - `## Prerequisites` — external access, accounts, credentials and capabilities (`0.x`), which gate the
     phases that consume them.
   - `## Test Setup` — the unit tests each phase drives from red to green.
   - `## Phase Processing Rules` — the per-phase procedure you will execute. Read it now; it is the
     authority for how every phase below is processed, including verification, the human approval stop,
     and the phase commit.
   - `## Delivery Phases` — the `### P#` blocks. Each carries `Story`, `Delivers`, `Areas`, `Depends on`,
     `Process` (backlink + `tier`), a frozen `Verify` plan, `Status`, and its child tasks.

   Build the phase order from each phase's `Depends on:` slot (consumer → producer). That slot is the ONLY
   sequencing source: there is no dependency map and no inline `[depends: ...]` metadata — if you find
   either, treat it as a stale artifact and report it rather than obeying it.

   Read the remaining context files listed in `contextFiles` as each phase needs them: `technical-design.md`
   is the primary developer HOW, `specs/**/*.md` bound the behaviour, the approved grooming ledger records
   the decisions the plan rests on. `design.md`, `stories.md`, `fsd/**/*.md` and `isd/**/*.md` are
   conditional and may be absent.

5. **Select implementation scope**

   Scope filters PHASES, not individual tasks — a phase is an atomic increment and is never split across
   sessions or roles.

   **Infer from the user's wording when explicit:**
   - `BE`, `backend`, "BE developer", "бек", "server", "custom API", "cartridge", "SFCC" → `BE`
   - `FE`, `frontend`, "front", "фронт", "Storefront Next", "React", "route", "component" → `FE`
   - `full`, "all", "всё", "full-stack" → `FULL`

   If still unclear, use the **AskUserQuestion tool** offering `BE only`, `FE only`, `Full implementation`.

   **Scope contract:**
   - A phase is in scope when its `Areas` sits entirely inside the selected role. A phase whose `Areas`
     mixes roles, or is `@SHARED`, needs `FULL` — or an explicit instruction from the user to take it.
   - Prerequisites and Test Setup entries are included when they unblock an in-scope phase.
   - There is no `@QA` work in this queue. A `- [ ]` task tagged `@QA` is a contract violation in the
     artifact: report it, do not implement it.

6. **Show the plan before touching code**

   Display: the schema, the selected scope, progress (`N/M tasks complete`), the ordered list of in-scope
   phases with their `tier` and current `Status`, any phase blocked on an unmet prerequisite, and the
   phases left to another role.

7. **Run the phase loop (the core of this command)**

   For each in-scope phase, in `Depends on:` order, ONE AT A TIME:

   > Execute the phase exactly as the `## Phase Processing Rules` block in `tasks.md` prescribes. Do not
   > paraphrase that block from memory and do not substitute a shortcut for any of its steps.

   Never start a phase whose producer phase is not yet approved, and never interleave two phases. What
   this command adds on top of that block is the session-level conduct:

   - **Announce the phase** before starting: its number, title, `tier`, and the tasks it contains.
   - **Implement task by task**, top to bottom, keeping each change minimal and scoped to the task it
     belongs to. Flip a task's `- [ ]` to `- [x]` only when that task's own `Done-when` is genuinely met.
   - **Verify with real execution.** The phase's `Verify` block is the acceptance and it is a plan you
     RUN: execute the phase's Test Setup unit tests and attach the output, then walk the phase's Scenario
     live on the running app per `verification-standards.mdc` — an @FE phase drives a real browser with
     the `playwright-cli` skill and captures the required screenshots and a clean console; an @BE phase
     captures request/response, logs and cache behaviour. Code review is not verification, and a checkbox
     is never justified by static reading, `--list`, a parse check, or "tests registered".
   - **Hand the human the evidence and the verdict, then STOP.** You never approve your own phase. Wait.
   - **After approval, commit that phase** as the block prescribes, then move to the next phase.
   - **Record the outcome** in the phase's `Status` slot every time it changes.

   **Why the commit lands per phase.** This supersedes the older uncommitted-by-default habit, and the
   reason is reviewability: each approved phase becomes exactly one revision, so a later phase cannot bury
   an earlier one in an undifferentiated working tree, a reviewer reads the increment the human actually
   approved, and a phase that turns out wrong is undone by dropping a single commit. Follow the
   `git-commit` skill for branch naming, message format and the OpenSpec flow. An unapproved, `ITERATE`
   or `BLOCKED` phase is never committed. A tasks file with NO phase structure keeps the old default: an
   uncommitted, review-ready tree, committed only when the user asks.

   **Pause and ask instead of improvising** when: a task is ambiguous; implementation contradicts the
   technical design or a spec bound (suggest the artifact update, do not silently diverge); a needed
   capability, credential or account has not been delivered (record it as that phase's `Status: blocked`
   with the blocker); the verify pass fails three iterate rounds; or you hit an error you cannot attribute.
   If failures are non-obvious or reproduce inconsistently, use the `systematic-debugging` skill before
   guessing at fixes.

8. **On completion or pause, show status**

   Display: selected scope; phases completed and approved this session with their commits; overall
   progress `N/M tasks complete`; remaining in-scope phases; blocked phases with their blockers; phases
   handed to another role. If all done, suggest archive. If paused, explain why and wait.

**Output During Implementation**

```
## Implementing: <change-name> (schema: <schema-name>)
Scope: FE — 4 of 7 phases in scope

### P2 — Address autocomplete  ·  tier: standard  ·  3 tasks
[...implementation...]
✓ P2.1 complete   ✓ P2.2 complete   ✓ P2.3 complete

Verify (independent pass):
- Unit: 12 passed / 0 failed
- Scenario: walked live, screenshots captured (suggestions, results, fallback), console clean
- Verdict: SHIP

STOPPING for your approval of P2 before P3 starts.
```

**Output On Completion**

```
## Implementation Complete

**Change:** <change-name>
**Schema:** <schema-name>
**Scope:** FE
**Progress:** 4/4 in-scope phases approved; overall 18/26 tasks complete

### Phases Approved This Session
- P1 — Search, results list, locator shell (sensitive) — a1b2c3d
- P2 — Address autocomplete — e4f5g6h

Remaining: P4, P6 are @BE and were left to that role.
```

**Output On Pause**

```
## Implementation Paused

**Change:** <change-name>
**Scope:** FE
**Progress:** P1 approved; P3 in progress

### Issue
P3 consumes the Cartora map SDK, and prerequisite 0.2 (Cartora account) is not delivered.
Recorded as `P3 Status: blocked — Cartora account pending (ref 0.2)`.

**Options:**
1. Continue with P5, which does not depend on 0.2
2. Stub the seam per planning Q1 and re-verify on delivery
3. Stop here

What would you like to do?
```

**Guardrails**
- The phase is the unit of work: one at a time, in `Depends on:` order, never interleaved, never split
- Implement in THIS window — no implementation subagents, no orchestrator/dispatcher mode
- Never self-approve a phase; every phase stops for a human before the next one starts
- Never mark a verification task `[x]` without real execution evidence from this session
- Never start a phase whose producer phase is not approved, or whose prerequisite is not delivered
- Do not create public functions, hooks, utilities, or patterns that `technical-design.md` / `tasks.md`
  do not carry; pause and report the design gap instead
- A `tier: sensitive` phase always goes to a human with its security pass — it can never auto-ship
- Formal QA (authored E2E suites, manual QA, accessibility audit, UAT) is QA-owned and out of this
  pipeline; do not queue or implement it
- Use `contextFiles` from the CLI output; don't assume file names
- For the feature schema, `technical-design.md` is the primary developer HOW; `design.md`, FSD, ISD and
  stories are conditional context and may be absent
- Pause on errors, blockers, or unclear requirements — don't guess

**Failure Recovery**
Enter this section only after a command in the normal workflow fails.
- Missing OpenSpec CLI: run `openspec --version 2>&1`, then tell the user to install it with
  `npm install -g @fission-ai/openspec@latest`.
- Project not initialized: tell the user to run `openspec init` once, then rerun `/opsx:apply`.

**Fluid Workflow Integration**

This command supports the "actions on a change" model:

- **Can be invoked anytime**: after partial implementation, or interleaved with other actions, as long as
  the entry gate passes
- **Allows artifact updates**: if implementation reveals a design issue, suggest updating the artifact —
  the queue is not frozen, but it is the authority while it stands
