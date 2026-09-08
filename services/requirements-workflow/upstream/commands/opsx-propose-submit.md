---
name: /opsx-propose-submit
id: opsx-propose-submit
category: Workflow
description: Submit the BA/SA proposal for review — commit the artifacts, open a review PR when the two-branch model is in use (single-branch just commits, no PR), and publish the FSD/ISD to Confluence after offer -> preview -> explicit yes (the agent runs the publish itself once confirmed; never without a yes)
---

Submit an OpenSpec change's completed BA/SA artifacts for review. This is the checkpoint between
`/opsx:propose` (which produces the artifacts and STOPS) and `/opsx:propose-approved` (which, after the
proposal is approved, generates `stories.md` and prepares Jira). Its job is to get the proposal in front
of the team and the client:

1. commit the BA/SA artifacts,
2. open a review pull request `requirements/<name>` → `feature/<name>` **only when the two-branch model
   is in use** (single-branch changes just commit on `feature/<name>` — no PR), and
3. publish the FSD/ISD to Confluence so the client and team can review the client-facing document.

This is an OFFER -> PREVIEW -> EXECUTE contract. The agent performs local reads, read-only Atlassian
discovery, and local markdown edits, and — only after an explicit yes — runs the mutating steps ITSELF:
a local git commit, a branch push + PR, and the Confluence publish. **The agent never mutates anything
without an explicit yes**, and it always previews exactly what will happen before executing. Jira issue
creation is NOT part of this command (it belongs to `/opsx:propose-approved`).

## Definitions

- **Local inventory**: filesystem-only reads under `openspec/changes/<name>/` (artifacts + FSD/ISD
  working copies). No Atlassian scripts, no project-context, no OpenSpec health checks.
- **External discovery**: read-only Atlassian calls through the scripts named below (`jira.mjs
  lookup-user`, `confluence.mjs list-children`, `confluence.mjs get-page`).
- **Confluence baseline**: the current live Confluence page downloaded as markdown next to the active
  working copy before updating an existing FSD/ISD page. File name: `<feature>.baseline.md`, with the
  page's binaries staged in a sibling `.<feature>.baseline-assets/` dir. Both are temporary
  delta/conflict inputs, gitignored, and never committed.
- **External mutation (agent-run, only after an explicit yes)**: the Confluence publish/update. The agent
  previews the exact operation (CREATE / UPDATE / REUSE+UPDATE, target page, mention) and, on a yes, runs
  `publish.mjs` itself; the script then stamps the working copy's `- Confluence:`/`- Page ID:` header.
  Jira issue creation is NOT part of this command (it belongs to `/opsx:propose-approved`).
- **Local/git mutation (agent-run, only after an explicit yes)**: local markdown edits (status flips),
  a git commit scoped to the change, and a branch push + PR when the user says yes to the PR.

## Input

The argument after `/opsx:propose-submit` is the change name (kebab-case). If omitted, infer it from
conversation context. If ambiguous, run `openspec list --json` once and use the **AskUserQuestion tool**
to select one active change.

Optional flag: `--dry` runs local inventory plus the Confluence external-discovery previews and stops
before every mutation (no commit, no push, no status flips).

## Preflight: sync is not forced here

`/project-sync` is **not** a mandatory preflight for submit — it runs at `/opsx:propose`, the phase that
produces the artifacts you submit. If you have not synced this session, or suspect the tooling scripts
(`publish.mjs`, `jira.mjs`) or artifacts are stale, run `/project-sync` manually first so you submit the
latest checkout. This does not change the "no project-context" scope of the Local inventory step below.

## State Machine

1. **Select change**

   Announce: `Using change: <name>`.

2. **Local inventory**

   Read only local files under `openspec/changes/<name>/`:
   - Which BA/SA artifacts exist (`discovery.md`, `design.md`, `proposal.md`, `grooming.md`,
     `specs/**/*.md`, and the client-facing doc `fsd/*.md` or `isd/*.md`).
   - For each FSD/ISD working copy, extract: doc type (`fsd`/`isd`), feature basename, title/WBS,
     Author field, status field, and local `- Page ID:` / `- Confluence:` metadata when present.
   - Confirm the proposal's grooming approval passed (do NOT re-open grooming here; if it has not
     passed, STOP and say what is unresolved — the change is not ready to submit).

   Print a short inventory summary (artifacts present, FSD/ISD working copies with their current
   status and whether a Page ID is present).

3. **Commit gate** (local git only, after an explicit yes)

   **Upstream actualization (before staging).** The FSD/ISD are the client-signed WHAT. If the settled
   FSD/ISD tailored or hardened anything the upstream `design.md` / `proposal.md` / `discovery.md` / delta
   specs assert, run the FSD/ISD upstream-actualization pass (design / proposal / discovery / specs ←
   FSD/ISD) via a fresh subagent BEFORE staging, so the commit and review PR carry upstream artifacts
   consistent with the signed contract — see
   [fsd-isd-upstream-actualization](.cursor/rules/workflow/openspec-workflow/references/fsd-isd-upstream-actualization.md).
   It is human-supervised and outside the eval contour; present any material upstream edit for re-approval
   before committing. If the FSD/ISD merely projected the upstream verbatim, skip it.

   **Branch-model detection (do this first):** check whether the edit branch exists in the code repo —
   `git rev-parse --verify requirements/<name>`. If it exists, this is the **two-branch** model (commit on
   `requirements/<name>`, review PR in step 4). If it does not, this is the **single-branch** model (no
   FSD/ISD, or the offer was declined at `/opsx:propose`): commit on `feature/<name>` and there is no PR.

   OFFER — do not commit automatically. Use the **AskUserQuestion tool**: `Commit the BA/SA artifacts` /
   `Leave uncommitted`. Show the exact commit message BEFORE running anything. On `Commit`, stage ONLY
   files under `openspec/changes/<name>/` and commit on the branch selected above (`requirements/<name>`
   in two-branch mode, `feature/<name>` in single-branch mode), following the `git-commit` skill for branch
   handling and the commit-message format.

   Never stage `openspec/changes/<name>/{fsd,isd}/*.baseline.md`. Never push in this step.

4. **PR gate** (two-branch model only; branch push + PR, only after an explicit yes that covers pushing)

   **Single-branch model (no `requirements/<name>` from the detection in step 3): SKIP this step
   entirely** — there is no review PR; the artifacts were simply committed on `feature/<name>` in step 3.
   Note "single-branch — no PR" and continue to step 5.

   Otherwise (two-branch), use the **AskUserQuestion tool**: `Open a review PR` / `Skip the PR`. Confirm
   the branches with the user first.

   On `Open a review PR` (this yes authorizes pushing and opening the PR, and nothing else): follow the
   `git-commit` skill's **OpenSpec branch workflow** — the review PR is `requirements/<name>` →
   `feature/<name>`; ensure `feature/<name>` is pushed/synced on the remote FIRST (so the PR base
   exists), then push `requirements/<name>`, then open the PR. Title `<name>: BA/SA proposal for review`;
   body summarizes the proposal:

   ```markdown
   ## Summary
   BA/SA proposal for review: discovery, design, proposal, grooming, specs, and the client-facing FSD/ISD.

   ## Review focus
   - Proposal scope lock + Delivery Breakdown
   - Client-facing FSD/ISD (also being published to Confluence for client review)
   ```

   Return the PR URL. Never force-push. Never push without a yes that explicitly covers pushing/PR (see
   `no-unsolicited-actions`). With no yes, leave the branch unpushed.

5. **Confluence publish** (offer -> preview -> on yes, the agent publishes FSD/ISD for client/team review)

   Only when an FSD/ISD working copy exists. Recommend publishing the client-facing document to the wiki
   so the client and team can review and comment. Use the **AskUserQuestion tool**:
   `Publish the FSD/ISD to Confluence` / `Skip publishing for now`.

   On `Publish`, for each FSD/ISD working copy first run external discovery and build the PREVIEW:
   - Resolve Author to accountId:
     ```bash
     node .cursor/skills/operate/atlassian/scripts/jira.mjs lookup-user "<Author>"
     ```
     Use the accountId only as the `--mention` value. Do not print credentials or `.env` values.
   - If local `- Page ID:` is present, the operation is `UPDATE`; download the Confluence baseline as
     canonical `<type>` markdown via the typed reverse pull (`--type` strips the wiki chrome so the
     baseline diffs cleanly against the working copy). Stage the live page's body images / diagram sources
     in a gitignored baseline-assets dir with `--assets-dir` (NOT the working copy's `./assets`, so a pull
     never clobbers local edits); the merge step localizes only the wiki-added ones from there:
     ```bash
     node .cursor/skills/operate/atlassian/scripts/confluence.mjs get-page <ID> --type=<type> \
       --into openspec/changes/<name>/<type>/<feature>.baseline.md \
       --assets-dir openspec/changes/<name>/<type>/.<feature>.baseline-assets
     ```
   - If local `- Page ID:` is absent, list existing children under the configured parent:
     ```bash
     node .cursor/skills/operate/atlassian/scripts/confluence.mjs list-children <CONFLUENCE_<TYPE>_PARENT_ID>
     ```
     The operation is `REUSE+UPDATE` when a matching WBS/feature page exists (download that page as the
     baseline); otherwise `CREATE`.
   - Pristine-base guard (before any merge, for `UPDATE`/`REUSE+UPDATE`): confirm the merge ancestor is the
     pristine discovery baseline — the initial commit on `feature/<name>`, not a later edit. This holds in
     either branch model (the pristine pull is committed on `feature/<name>` before any edit), so the file's
     first-add commit must be reachable from `feature/<name>`:
     ```bash
     seed=$(git log --diff-filter=A --format=%H -- openspec/changes/<name>/<type>/<feature>.md | tail -1)
     git merge-base --is-ancestor "$seed" feature/<name> && echo "pristine-base OK" \
       || echo "WARN: merge base is not the pristine discovery pull — the discovery bootstrap was likely skipped"
     ```
     On WARN the diff base is an already-edited copy, so the PR delta and the 3-way merge can be wrong.
     Surface it and let the user decide to proceed or re-bootstrap; do not silently continue as if clean.
   - For every `UPDATE`/`REUSE+UPDATE`, reconcile the freshly-pulled baseline into the working copy with
     the git-native, body-scoped 3-way merge (base = the working copy at its seed/first-appearance commit —
     the discovery baseline pulled onto `feature/<name>` — read from git history; no committed baseline).
     The pristine baseline MUST be committed on `feature/<name>` first (the discovery-time bootstrap) so the
     merge has an ancestor:
     ```bash
     node .cursor/skills/operate/atlassian/scripts/merge-baseline.mjs \
       openspec/changes/<name>/<type>/<feature>.md \
       openspec/changes/<name>/<type>/<feature>.baseline.md --type=<type> \
       --assets-from openspec/changes/<name>/<type>/.<feature>.baseline-assets
     ```
     `--assets-from` localizes any wiki-added images the merge pulled into the body (copied from the
     staging dir into the working copy's `./assets`); a `WARN — referenced asset(s) not found` line lists
     dangling links to fix by hand.
     - Exit 0 (clean): the merge reconciled local + live edits into the working copy; continue. Any
       `NOTE — wiki chrome changed` lines list header-card/approval/reference edits made on the wiki
       (chrome is kept from the working copy, never auto-merged) — port them into the working copy by hand
       if they must be preserved.
     - Exit 2 (conflict): the script only SIGNALS an overlap — it writes diff3 markers
       (`<<<<<<< working copy` / `||||||| base (discovery baseline)` / `>>>>>>> confluence (live)`) into
       `<feature>.md` and never auto-merges the overlap. Do NOT publish yet. The AGENT resolves each hunk
       from the three visible versions (base / ours = working copy / theirs = confluence live), in order:
       - `ours == theirs` -> keep that text.
       - one side equals `base` (unchanged) -> take the other (changed) side.
       - both changed but ADDITIVE / non-contradictory (each added distinct rows or sentences) -> UNION:
         keep both, ours first then theirs, de-duplicating identical lines.
       - both changed the SAME thing differently (a true contradiction) -> keep OURS and record a
         `REVIEW: wiki had "<theirs>"` note for the human.
       - a large or meaning-bearing hunk you cannot confidently reconcile (a whole section rewritten on the
         wiki, a requirement's meaning flipped) -> STOP and ask the user; do not guess.

       `REVIEW` notes go ONLY into the chat decision log, NEVER into the document (they would publish as
       page noise). After removing every marker, re-run the SAME merge with `--check` and require a clean
       report (0 markers) before continuing:
       ```bash
       node .cursor/skills/operate/atlassian/scripts/merge-baseline.mjs \
         openspec/changes/<name>/<type>/<feature>.md \
         openspec/changes/<name>/<type>/<feature>.baseline.md --type=<type> --check
       ```
       Include the resolution decision log (union / ours-won + wiki alternative / escalated) in the publish
       preview so the human approves the merge with the trade-offs in hand.
     - The `<feature>.baseline.md` pull and the `.<feature>.baseline-assets/` staging dir are throwaway
       3-way inputs: both are gitignored and never committed. Delete the staging dir once the merge (and
       any conflict resolution) is done — the wiki-origin images it held are now localized in `./assets`.
   - Make the working copy publish-ready (local edit): FSD -> set `FSD Status` to `astound review`; ISD ->
     set `ISD Status`/badge to `astound review`. Never set `<client> review`/`approved` here — those are later/manual.
   - Target Approval Date (local edit): if the card still carries the `dd/mm/yyyy` placeholder, ASK the
     human for the client's target sign-off date and write it into the `Target Approval Date` card row in
     `dd/mm/yyyy` form (e.g. `15/03/2026`). If the human has no date yet, KEEP the placeholder and note
     "Target Approval Date: not set" in the publish preview and summary. Never invent a date.
   - For every `UPDATE`/`REUSE+UPDATE`, run a read-only inline-comment risk preview so the user sees which
     open wiki comments this edit would orphan BEFORE the real publish (this `--dry` pass makes no
     changes):
     ```bash
     node .cursor/skills/operate/atlassian/scripts/publish.mjs openspec/changes/<name>/<type>/<feature>.md \
       --type=<type> --render=template --page-id=<ID> --dry
     ```
     Surface any `WOULD DROP comment …` lines to the user in the preview. The same `--dry` pass also
     prints the `Attachment plan (preview): … upload/update/unchanged/prune` line plus any
     `WOULD DELETE attachment …` (managed images/diagram sources removed or renamed since the last
     publish) and `NOTE unreferenced (not managed, kept)` — surface these too so the human sees which wiki
     attachments this publish would delete BEFORE approving.
   - Collect a short version comment (<=50 chars) describing this publish (e.g. `add checkout FRs`); it is
     REQUIRED and is threaded into Confluence's version history via `--comment` below. `publish.mjs` enforces
     this — a typed publish without `--comment` fails (only the `--dry` preview above is exempt).
   - Present the preview to the user — the operation (`CREATE` / `UPDATE` / `REUSE+UPDATE`), the target
     page (title or Page ID), the mention, the version comment, any inline-comment-drop risk, and the exact
     command that will run — and get an explicit yes.

   On that yes, the agent RUNS the publish itself (this is the offer -> preview -> execute step; do NOT
   hand the command off). A typed publish (`--type=fsd|isd`) renders the FSD/ISD chrome (the General
   FSD/ISD Information card, approval roster, References, status lozenges, Document Change Log footer) by default — the
   `--render=template` flag below is explicit for clarity; do NOT drop it, and never pass
   `--render=markdown` for an FSD/ISD (that publishes a bare page with no chrome):
   - Page ID present (update in place):
     ```bash
     node .cursor/skills/operate/atlassian/scripts/publish.mjs openspec/changes/<name>/<type>/<feature>.md \
       --type=<type> --render=template --page-id=<ID> --mention="<Author>=<accountId>" \
       --comment="<version comment>" --skip-attachments
     ```
     Omit `--skip-attachments` whenever the `--dry` attachment plan showed ANY change — an upload, update,
     OR prune (added/changed images, or removed/renamed ones whose managed attachments must be deleted).
     Only keep `--skip-attachments` when the plan was all-`unchanged`. A referenced diagram image
     (`![](./assets/<slug>.png)` with a sibling `<slug>.drawio`) auto-uploads its `.drawio` companion as
     the downloadable source (hash-gated, so unchanged files are skipped). With attachments enabled the
     publish also deletes managed orphans (see the preview) and never removes attachments it does not own.
   - No existing page (create): title `<WBS> (PhaseN) - <Feature> <FSD|ISD>`:
     ```bash
     node .cursor/skills/operate/atlassian/scripts/publish.mjs openspec/changes/<name>/<type>/<feature>.md \
       --type=<type> --render=template --title="<title>" --mention="<Author>=<accountId>" \
       --comment="<version comment>"
     ```
   - `publish.mjs` stamps the working copy's metadata header itself: on success it inserts/updates
     `- Confluence: <URL>` and `- Page ID: <ID>` at the top of the working copy (the same script-owned
     format used on a later pull). Recording the URL here matters: the functional-artifacts script
     reads this header to build the stories' acceptance-criteria references and FSD/ISD References — when
     unpublished the reference degrades to a local FSD reference (no block) and upgrades to the anchor on
     a re-run after publish. Confirm the header was stamped after the run.

   The publish flow may have updated the working copy four ways: the 3-way merge reconciled the wiki edits
   into `<feature>.md` (body), any wiki-added images were localized into `./assets`, the status flip, and the
   stamped `- Confluence:` / `- Page ID:` header. If the artifacts were already committed in step 3, OFFER a
   small follow-up commit on the same branch used there (`requirements/<name>` in two-branch mode, else
   `feature/<name>`) that captures ALL of these — the reconciled body AND the localized `./assets` (stage
   the new/changed binaries too) — so `ours` (and the review PR, when there is one) stays in sync with
   what was published and the same wiki edits do not re-surface as conflicts next round-trip. Do NOT stage
   the `.<feature>.baseline-assets/` staging dir (gitignored, deleted above). Otherwise mention the working
   copy stays uncommitted.

6. **Summary**

   Print:

   ```text
   ## Propose-Submit - <change-name>

   Commit: <sha (scoped to the change)|left uncommitted>
   PR: <url|skipped>
   Confluence: <published N doc(s): <url(s)>|skipped|no FSD/ISD>
   Target Approval Date: <dd/mm/yyyy set|not set (placeholder kept)|n/a>
   Next: share the PR and the Confluence page for team + client review. After the
     proposal is approved, run /opsx:propose-approved to generate stories.md and create Jira stories.
   ```

## Invariants

- The agent publishes to Confluence ONLY after offer -> preview -> an explicit yes; it never publishes
  without a yes, and always previews the exact operation first. It NEVER creates Jira issues here (that is
  `/opsx:propose-approved`).
- Publishing sets the document to `astound review` (FSD and ISD); this command never sets
  `<client> review`/`approved`. On success `publish.mjs` stamps the working copy's `- Confluence:`/
  `- Page ID:` header.
- Before publishing, the agent fills the card's `Target Approval Date` from the human (format
  `dd/mm/yyyy`) whenever it still holds the `dd/mm/yyyy` placeholder; when the human has no date, the
  placeholder stays and the summary flags it. The agent never invents a date.
- Git commit is scoped to `openspec/changes/<name>/`. Push + PR run only on a separate explicit yes that
  covers pushing. Never force-push.
- Baseline inputs (`{fsd,isd}/*.baseline.md` and `{fsd,isd}/.*.baseline-assets/`) are temporary,
  gitignored, and never committed; the publish deletes managed orphaned attachments (removed/renamed
  images) but never attachments it does not own.
- "Approved" is NOT decided here — this command SUBMITS for review. Story generation and Jira creation
  happen later in `/opsx:propose-approved` once the proposal is approved.

## Failure Recovery

Enter this section only after a command in the state machine fails.

- Missing OpenSpec CLI: run `openspec --version 2>&1`, then report the install command:
  `npm install -g @fission-ai/openspec@latest`.
- If OpenSpec reports the project is not initialized, tell the user to run `openspec init` once, then
  rerun `/opsx:propose-submit`.
- If an Atlassian script fails due to credentials/configuration, report the failing command and the
  missing config key name only. Do not print `.env` values.
