---
name: /opsx-propose-approved
id: opsx-propose-approved
category: Workflow
description: Approval gate - re-sync the FSD + stories.md deterministically from the approved artifacts (functional-artifacts script, preserving backfilled Jira keys), then after offer -> preview -> explicit yes the agent creates the Jira stories itself (jira.mjs create-stories, which backfills keys/links into stories.md), then commits and pushes to the working branch
---

Run the approval gate for an OpenSpec change whose BA/SA **proposal** is approved — the change merge
request has been reviewed and approved, and the client-facing FSD/ISD was submitted for review by
`/opsx:propose-submit`. "Approved" here means the proposal is approved, not that the client has formally
signed off the FSD/ISD document.

This command:

1. RE-SYNCS the FSD + `stories.md` deterministically from the approved artifacts (proposal + specs, plus
   design for Implementation notes) by running the openspec-workflow functional-artifacts script —
   `stories.md` was first produced at `/opsx:propose`; the agent never hand-authors it,
2. after offer -> preview -> an explicit yes, CREATES the Jira stories itself by running
   `jira.mjs create-stories` (which parses `stories.md`, creates/reuses issues, and backfills the keys and
   Jira links into `stories.md`) — **the agent creates issues only on a yes, never without one**,
3. commits the updated `stories.md`, and
4. pushes the work to the working branch (each only after an explicit yes).

Confluence FSD/ISD publishing is NOT part of this command — it happens earlier in `/opsx:propose-submit`.
Here the script reads each FSD's `- Confluence:` header to build the stories' acceptance-criteria
references and References: when the FSD is published the acceptance-criteria is a live anchor link to
its Acceptance Criteria wiki section; when it is not, a plain local reference (no block) that upgrades on
a later re-run after publish. Publishing stays a separate pipeline decision.

This is a RE-SYNC + OFFER -> PREVIEW -> EXECUTE contract. The agent runs the local functional-artifacts script, does
local reads, read-only Jira discovery, and — after an explicit yes — runs `jira.mjs create-stories`, a
local git commit, and a branch push. It never mutates Jira or git without an explicit yes.

## Definitions

- **Functional-artifacts re-sync (agent-run script)**: `node .cursor/skills/plan/openspec-workflow/scripts/generate-functional-artifacts.mjs
  openspec/changes/<name>` writes the FSD read-only sub-sections + `openspec/changes/<name>/stories.md`
  from `proposal.md`, the specs, and `design.md`. Deterministic; decides nothing; preserves backfilled
  Jira keys/links in `stories.md`.
- **Local inventory**: filesystem-only reads under `openspec/changes/<name>/` (parse the generated
  `stories.md`). No project-context, no OpenSpec health checks.
- **External discovery**: read-only Jira calls, including `jira.mjs create-stories --dry-run` (search-only
  preview) and, where useful, `jira.mjs search` / `jira.mjs lookup-user`.
- **External mutation (agent-run, only after an explicit yes)**: Jira story creation via
  `jira.mjs create-stories`. The agent previews with `--dry-run` first and, on a yes, runs the real
  command itself. `create-stories` reads `stories.md` blocks directly (no hand-off temp files), reuses
  existing issues by idempotency hash + dedup, and backfills the created/reused keys and Jira links back
  into `stories.md`.
- **Local mutation (agent-run)**: running the functional-artifacts script, the `stories.md` backfill performed by
  `create-stories`, and — after an explicit yes — git staging + commit and a branch push.

## Input

The argument after `/opsx:propose-approved` is the change name (kebab-case). If omitted, infer it from
conversation context. If ambiguous, run `openspec list --json` once and use the **AskUserQuestion tool**
to select one active change.

Optional flag: `--dry` runs story generation to stdout (does not overwrite `stories.md`) plus the Jira
external-discovery preview, prints the execution preview, and stops before every mutation.

## Preflight: sync is not forced here

`/project-sync` is **not** a mandatory preflight for this command — it runs at `/opsx:propose`. If you
have not synced this session, or suspect the tooling scripts (`generate-functional-artifacts.mjs`,
`jira.mjs`) or schema are stale, run `/project-sync` manually first so the re-sync runs the latest
tooling. This does not change the "no project-context" scope of the Local inventory step below.

## State Machine

1. **Select change**

   Announce: `Using change: <name>`. Confirm the proposal is approved before proceeding; if it is not,
   STOP and say so.

2. **Re-sync the functional artifacts (FSD + `stories.md`)**

   `stories.md` was already produced at `/opsx:propose` alongside the FSD by the functional-artifacts
   projection script. Re-run that script here to RE-SYNC both from the latest proposal/specs and pick up
   the enriched/published FSD; it PRESERVES any Jira keys/links already backfilled into `stories.md`, so
   a re-sync never clobbers them. For a small bug fix / tiny change, SKIP (Dev works from specs + design).

   ```bash
   node .cursor/skills/plan/openspec-workflow/scripts/generate-functional-artifacts.mjs openspec/changes/<name>
   ```

   - It writes the FSD + `stories.md` in one run. `--dry` shows the output without writing.
   - Publish-optional: if the FSD carries a `- Confluence:` header, each story's acceptance-criteria is a
     live Confluence anchor; if not, a plain local reference (no block) that upgrades on a later re-run
     after publish.
   - On a validation error it lists what to fix UPSTREAM (proposal slice / spec); fix and re-run. Do NOT
     hand-edit the generated bodies (proposal / spec / design are the source), then re-run.

   Print a one-line confirmation: `Re-synced FSD + N stories -> openspec/changes/<name>/`.

3. **Local inventory**

   Parse the generated `stories.md`: extract each `### <STORY-ID> — <title>` block (id + title + body).
   Print:

   ```text
   ## Approval Inventory - <change-name>

   Stories: <M>
   - <STORY-ID> "<title>"

   No external lookup has run yet.
   ```

   If `--dry` was passed, continue to the Jira external-discovery preview and stop after printing it.

4. **Intent gate**

   Use the **AskUserQuestion tool** once:

   > "What should I do for `<change-name>`? (On yes I'll create the Jira stories myself and backfill the
   > keys into stories.md.)"

   Options:

   - `Create Jira stories`
   - `Dry preview only`
   - `Cancel`

   Branch rules:

   - `Cancel`: stop with `Stories generated; no Jira created - cancelled before external discovery.`
   - `Dry preview only`: run the `--dry-run` preview, print it, and stop.
   - `Create Jira stories`: run the `--dry-run` preview, get the final yes, then run `create-stories`.

   Ask the Epic question once with the **AskUserQuestion tool** so it can be passed to `create-stories`:

   > "Should the Jira Story issue(s) be linked to an Epic?"

   Options:

   - `No Epic`
   - `Use existing Epic key` — require the user to provide the Epic issue key.

   If helpful, offer to search for a likely Epic first (`jira.mjs search '<jql>'`) so the user can pick
   one; never invent an Epic key.

5. **Jira preview** (`create-stories --dry-run`)

   Run the story-aware dry run — it parses `stories.md` blocks, searches Jira for each (idempotency hash +
   dedup), and reports the per-story CREATE/REUSE decision plus the backfill it would write, without
   mutating anything:

   ```bash
   node .cursor/skills/operate/atlassian/scripts/jira.mjs create-stories \
     openspec/changes/<name>/stories.md [--epic="<EPIC-KEY>"] --dry-run
   ```

   Present its preview to the user:

   ```text
   ## Jira Execution Preview - <change-name>

   - <STORY-ID> "<title>": <CREATE|REUSE <ISSUE-KEY>>
   Backfill: heading ids -> Jira keys, a "Jira tickets created via /opsx:propose-approved: <keys>" line,
     and a "- Jira: [<KEY>](<url>)" reference per story.
   ```

   If this is dry preview mode (or `--dry` was passed), stop here.

6. **Create the Jira stories** (offer -> preview -> on yes, the agent runs `create-stories` itself)

   After the dry-run preview, get an explicit yes to proceed. On that yes, the agent RUNS the real command
   itself (do NOT hand it off):

   ```bash
   node .cursor/skills/operate/atlassian/scripts/jira.mjs create-stories \
     openspec/changes/<name>/stories.md [--epic="<EPIC-KEY>"]
   ```

   `create-stories` does the rest itself: it creates each new Story from `JIRA_PROJECT_KEY` (summary =
   heading title; description = the block body verbatim), reuses issues detected by the idempotency hash /
   `--dedup`, and backfills `stories.md` — replacing each `TBD-n` heading id with the real key, inserting
   a `Jira tickets created via /opsx:propose-approved: <keys>` line above `## Stories`, and adding a
   `- Jira: [<KEY>](<browse-url>)` reference to each story. No temporary description files, no hand-off.

   Report the created/reused keys and confirm the `stories.md` backfill landed. If issues were already
   created on an earlier run and only the links need re-stamping, use the search-only refresh instead:

   ```bash
   node .cursor/skills/operate/atlassian/scripts/jira.mjs create-stories \
     openspec/changes/<name>/stories.md --refresh
   ```

7. **Offer to commit** (local git only, after an explicit yes)

   If no local file changed, report `No local changes - nothing to commit.` and skip.

   Otherwise show the changed files and OFFER to commit — do not commit automatically. Use the
   **AskUserQuestion tool**: `Commit now` / `Leave uncommitted`. On `Commit now`, stage only files touched
   by this run and commit:

   - `openspec/changes/<name>/stories.md`

   ```bash
   git commit -m "docs(<name>): add stories and link Jira tickets"
   ```

8. **Push to the working branch** (only after an explicit yes that covers pushing)

   OFFER to push the current working branch. Use the **AskUserQuestion tool**: `Push to the working
   branch` / `Leave unpushed`. On `Push`:

   ```bash
   git push
   ```

   Never force-push. Never push without a yes that explicitly covers pushing (see
   `no-unsolicited-actions`). With no yes, leave the branch unpushed.

9. **Summary**

   Print:

   ```text
   ## Propose-Approved - <change-name>

   Stories: <N generated -> stories.md | skipped (not story-worthy)>
   Jira: <created/reused M story(ies): <keys>|dry preview only|skipped|none>
   Backfill: <keys + Jira links written to stories.md|none>
   Commit: <sha|left uncommitted|none>
   Push: <pushed <branch>|left unpushed|none>
   Next: share stories.md and the Jira board for planning.
   ```

## Invariants

- "Approved" in this command means the BA/SA proposal is approved, not that the client formally signed
  off the document.
- `stories.md` is a deterministic projection produced by the functional-artifacts script (alongside the
  FSD), never hand-authored. Corrections go upstream (proposal / spec / design), then the script is re-run.
- The agent creates Jira stories ONLY after offer -> preview (`create-stories --dry-run`) -> an explicit
  yes; it never creates issues without a yes. On a yes it runs `create-stories` itself.
- Confluence publishing is NOT done here (it happens in `/opsx:propose-submit`); this command only reads
  the FSD's `- Confluence:` header the script uses for the acceptance-criteria links and References.
  An unpublished FSD does NOT block: the acceptance-criteria degrades to a local reference and upgrades
  on a re-run after publish.
- Backfill of keys/links into `stories.md` is performed by `create-stories` itself (no separate hand-off
  round-trip); `--refresh` re-stamps links for already-created issues without creating anything.
- Jira issues are searched before creation (idempotency hash + `--dedup`). Account IDs / Epic keys come
  from real lookups, never invented.
- Git commit is scoped to the change's `stories.md`. Push runs only on a separate explicit yes. Never
  force-push.

## Failure Recovery

Enter this section only after a command in the state machine fails.

- If the functional-artifacts script fails: report the failing command and the actionable finding it
  prints. Common causes are a missing `proposal.md` Delivery Breakdown, a spec heading without a WBS, or
  a slice with no covering spec. Fix the named upstream artifact (proposal / spec) and re-run.
- If an `openspec ...` command fails with a missing-CLI error, run `openspec --version 2>&1`, then report
  the install command: `npm install -g @fission-ai/openspec@latest`.
- If OpenSpec reports the project is not initialized, tell the user to run `openspec init` once, then
  rerun `/opsx:propose-approved`.
- If a Jira script fails due to credentials/configuration, report the failing command and the missing
  config key name only. Do not print `.env` values.
