---
name: /opsx-propose
id: opsx-propose
category: Workflow
description: BA/SA phase - create a change and generate BA/SA artifacts (stops before Planning)
---

Propose a new change - the BA/SA phase. Create the change, then generate discovery/design/proposal/grooming
one at a time — STOPPING after each artifact for human review and approval (the Per-Artifact Review Gate),
with a mandatory human grooming approval before generating specs and the optional handoff artifacts.

I'll create a change and generate artifacts in sequence (order driven by the schema's `requires:`).
The `feature` schema starts with discovery — gathering requirements before writing anything — then
`design` (the standard architecture/solution step) and `proposal` (the scope lock + delivery offer: story slices with acceptance-criteria seeds), and this
command STOPS before Planning artifacts. It does NOT generate `technical-design` or `tasks` (those are the Dev
Planning phase, owned by `/opsx:plan`). The client-facing document type follows the delivery scope
resolved at the scope gate and carried by discovery: a `functional` change publishes an **FSD**, an
`integration` change publishes an **ISD** — one change never produces both (a mixed ask is split at
the scope gate). The client-facing doc is conditional only in whether the change warrants it at all
(skip for small bug fixes / internal changes). `stories.md` IS produced here — deterministically,
alongside the FSD, by the openspec-workflow functional-artifacts script (never authored by the agent);
its Jira tickets are created later at `/opsx:propose-approved`. Grooming is never self-approved by the
agent: every question, prerequisite, Change Request disposition, and architecture/tech decision must
be answered and approved by a named human before specs / the client-facing doc are generated.

When the BA/SA artifacts are ready, run `/opsx:propose-submit` to commit them, open the review PR, and
prepare the FSD/ISD wiki publish for client/team review.

## Preflight (MANDATORY FIRST STEP): `/project-sync` + ground every phase in project-context

**This runs before anything else in this command. It is not optional and it is not skippable.** Do not
read change artifacts, ask the delivery-scope question, or generate any output before it completes.

1. **Run `/project-sync` first, always.** It fast-forwards the current project branch, force-refreshes
   the OpenSpec schema/config from the library, and syncs + reindexes project-context so retrieval
   reflects the latest project state. `/project-sync` is a preflight, not an OpenSpec artifact step: it
   works only on the current project branch, never merges the shared tooling base branch, does not
   compare OpenSpec templates to existing artifacts, and does not run `openspec status` or
   `openspec instructions`. Do not ask the user what to do unless a step is blocked. Stop on the first
   real failure — never start discovery on a stale or partially-synced index.

2. **Ground every phase in project-context — not just the first one.** BA/SA artifacts must be built
   from existing project context (Confluence, Jira, meeting notes, ADRs, prior specs; see the
   `project-context` skill), plus committed design assets and the FSD/ISD source docs for UI/visual
   requirements. Before you produce OR advance each artifact — discovery, design, proposal, grooming,
   specs, FSD/ISD — **actively re-query project-context for the sources that phase needs**. Each phase
   pulls on different sources, so a single lookup at the start is not enough. Do NOT coast on artifacts
   already in the change folder or on your memory of an earlier phase: when you move to the next phase,
   search project-context again for what that phase depends on.

---

**Input**: The argument after `/opsx:propose` is the change name (kebab-case), OR a description of what the user wants to build.

**Steps**

1. **If no input provided, ask what they want to build**

   Use the **AskUserQuestion tool** (open-ended, no preset options) to ask:
   > "What change do you want to work on? Describe what you want to build or fix."

   From their description, derive a kebab-case name (e.g., "add user authentication" → `add-user-auth`).

   **IMPORTANT**: Do NOT proceed without understanding what the user wants to build.

2. **Delivery scope & split gate**

   Before creating or resuming the change, classify the requested delivery scope:
   - `functional`: the change delivers a client/end-user feature surface. It may
     reference existing or separately-owned integrations, but it does not own their
     contracts.
   - `integration`: the change delivers an agnostic service integration
     capability. Name it for the service/capability (for example
     `google-maps-integration`), not for the first feature that consumes it.

   If the request bundles a substantial functional surface with a real connector
   (new/changed endpoints, auth scopes, external-service contracts, data mapping,
   credential/key/billing/security decisions, or cross-system sequencing), STOP
   before creating discovery. Explain that this would create an oversized mixed
   change, summarize the real scope, and recommend splitting it into a functional
   FSD change plus one or more agnostic ISD changes.

   Use the **AskUserQuestion tool** to ask which path to take. The options should
   be concrete and short, for example:
   - split as recommended;
   - proceed with the functional change only, referencing existing/separate
     integrations as dependencies;
   - proceed with the agnostic integration change only;
   - explicitly continue as a combined change despite the governance warning.

   Trivial client usage of an already-governed integration, or a one-line vendor
   snippet with no new contract/credential/data-mapping work, does not require a
   split by itself. The selected scope decision must be carried into discovery's
   Delivery Scope & Targets.

   The resolved scope also fixes the client-facing document type for the whole
   change: `functional` -> FSD (no ISD); `integration` -> ISD (no FSD). This is
   decided HERE and recorded by discovery — never re-decided at generation time.

3. **Discovery source readiness**

   Before creating or resuming the change, make sure the source inputs needed by the schema's
   `discovery` instruction are discoverable. Do not duplicate the discovery source policy here; follow
   `openspec instructions discovery --change "<name>" --json` for the artifact contract and record missing
   inputs in `discovery.md` instead of fabricating them.

   If the user reaches propose/explore and no `discovery.md` exists yet, propose creating it first. The
   detailed artifact-generation rules live in the schema's `discovery` instruction.

4. **Create the change directory**
   ```bash
   openspec new change "<name>"
   ```
   This creates a scaffolded change at `openspec/changes/<name>/` with `.openspec.yaml`.

5. **Get the artifact build order**
   ```bash
   openspec status --change "<name>" --json
   ```
   Parse the JSON to get:
   - `artifacts`: list of all artifacts with their status and dependencies
   - Note the BA/SA boundary: this command generates the BA/SA artifacts and STOPS before `technical-design`
     and `tasks`. `applyRequires` (e.g., `["tasks"]`) belongs to Planning/Implementation
     (`/opsx:plan` + `/opsx:apply`) — do NOT generate it here.

6. **Create discovery, enforce its gate, then create design, proposal and grooming**

   Use the **TodoWrite tool** to track progress through the artifacts.

   **Branch bootstrap (D1) — a STRICTLY ORDERED sequence interleaved with discovery, delegating the git
   steps to the `git-commit` skill's [OpenSpec discovery-time bootstrap](.cursor/skills/release/git-commit/references/openspec-discovery-bootstrap.md).**
   It is local-only (never pushes) and is the ONLY git work this command does. Follow the steps in order —
   do NOT bootstrap both branches up front. **NEVER create `requirements/<name>` until discovery is
   authored, its Discovery Gate has passed, AND the two-branch offer was explicitly accepted**; creating it
   earlier, or without asking, violates this command and the git-commit skill (the default is a single
   `feature/<name>`).
   - **Before** authoring `discovery.md`: create the feature branch off the integration branch
     (`git checkout -b feature/<name> origin/<integration> --no-track`) if it does not exist.
   - Author `discovery.md` here (its `Delivery Scope & Targets` table fixes each doc's Type + NEW/EXTEND +
     Confluence target). Leave it UNCOMMITTED — it rides along to whichever branch you work on.
   - **After** discovery is authored and its Discovery Gate passes, pick the branch model from the
     `Delivery Scope & Targets` table:
     - **No FSD/ISD target** — stay on `feature/<name>` (single-branch). Do NOT offer or create
       `requirements/<name>`; all BA/SA work is committed on `feature/<name>` at `/opsx:propose-submit`.
     - **At least one FSD/ISD target** — STOP and OFFER the two-branch model with the **AskUserQuestion
       tool**: `Set up a requirements branch (recommended)` / `Work directly on the feature branch`. Explain
       the trade-off in one line: a `requirements/<name>` branch makes the FSD/ISD changes show up as a
       clean, reviewable diff in git (review PR `requirements → feature`); the human may decline. Do NOT
       create `requirements/<name>` until this offer is explicitly accepted.
   - **EXTEND seeding (both models):** for each `EXTEND` target, typed-pull the pristine live page into the
     working-copy path and commit ONLY that file on `feature/<name>` as the INITIAL commit — that pristine
     first-add commit is the merge base `/opsx:propose-submit` relies on in either model (`NEW` seeds
     nothing). Do this BEFORE creating any edit branch.
   - **If the two-branch model was accepted:** create the edit branch off feature
     (`git checkout -b requirements/<name> feature/<name> --no-track`) and switch to it. Do ALL remaining
     artifact work (design, proposal, grooming, specs, FSD/ISD) on `requirements/<name>`.
   - **Otherwise (single-branch):** stay on `feature/<name>` and do all remaining artifact work there. No
     separate baseline branch or commit is needed — the EXTEND initial commit above already anchors the diff.

   Loop through artifacts in dependency order, but only until `grooming.md` has been created or loaded.
   The pre-grooming order is `discovery` -> `design` -> `proposal` -> `grooming` (`design` is the
   standard architecture / solution step for every change; `proposal` requires it).
   SKIP `technical-design`, `tasks` and any other Dev-phase artifact — they are generated by
   `/opsx:plan`, not here.

   **The client-facing doc (`fsd` OR `isd`) is CONDITIONAL — not every change needs it. `stories.md` is
   generated here by the projection script alongside the FSD or ISD (Jira tickets created at `/opsx:propose-approved`):**
   - The document TYPE is never a choice at this step: it was fixed by the delivery scope gate and is
     recorded in discovery's "Delivery Scope & Targets" (Type column). A `functional` change produces
     `fsd` only; an `integration` change produces `isd` only. Never generate both for one change — if
     both seem needed, the change was mis-scoped: go back to the split gate instead of generating.
   - Generate `fsd` (functional scope) only if the change has a client-facing surface a client or end
     user would review. Skip it for pure refactors / internal tech / infra-only changes. A
     cross-cutting change may produce several FSD files (one per affected area, e.g. PDP + PLP + cart) —
     one row each in "Delivery Scope & Targets". "Generate" here is SCAFFOLD-FIRST: run the FSD scaffold
     script to fill the read-only sub-sections (User Story / Acceptance Criteria / Key behaviours), then
     enrich only the non-owned sections — the agent never hand-authors those three (see step 8).
   - Generate `isd` (integration scope) for the agnostic service capability the change delivers. ISDs
     are service/capability specs, not children of the first feature that consumes them. "Generate" here
     is also SCAFFOLD-FIRST: run the ISD scaffold script (`generate-integration-artifacts.mjs`) to
     bootstrap the working copy and fill the read-only "## In Scope Functional Requirements" section
     (User Story / Acceptance Criteria / Key behaviours — NO Visualization), then author the rest (the
     agnostic Requirements table, Integration Specification, governance) — never hand-author those three
     sub-sections (see step 8).
   - Never hand-author `stories`. `stories.md` is a deterministic projection of the approved proposal +
     the client-facing doc (+ design), generated ALONGSIDE the FSD by `generate-functional-artifacts.mjs`
     (functional flow) or ALONGSIDE the ISD by `generate-integration-artifacts.mjs` (integration flow)
     (see step 8); Jira tickets are created from it at `/opsx:propose-approved`.
   - When in doubt about whether a client-facing doc is warranted AT ALL, ask the user with the
     **AskUserQuestion tool**. Do not ask which doc type — that is already decided.
   - Skipping the client-facing doc never blocks the pipeline — nothing downstream requires it. `specs`
     is the minimum required BA/SA output that Dev needs.

   a. **For each pre-grooming artifact that is `ready` (dependencies satisfied)**:
      - Get instructions:
        ```bash
        openspec instructions <artifact-id> --change "<name>" --json
        ```
      - The instructions JSON includes:
        - `context`: Project background (constraints for you - do NOT include in output)
        - `rules`: Artifact-specific rules (constraints for you - do NOT include in output)
        - `template`: The structure to use for your output file
        - `instruction`: Schema-specific guidance for this artifact type
        - `outputPath`: Where to write the artifact
        - `dependencies`: Completed artifacts to read for context
      - Read any completed dependency files for context
      - Create the artifact file using `template` as the structure
      - Apply `context` and `rules` as constraints - but do NOT copy them into the file
      - Show brief progress: "Created <artifact-id>"
      - If `<artifact-id>` is `discovery`, immediately read `discovery.md` and enforce the schema's
        Discovery Gate before creating `design.md` and `proposal.md`:
        - If the gate blocks, STOP before design/proposal. Tell the user which CRITICAL Open Questions
          are still open and route the follow-up through `/opsx:explore <change-name>` (or continue in
          chat if the user is already answering them).
        - Record any human answer/sign-off back into `discovery.md` before resuming.
        - Do not infer sign-off from source documents, meeting notes, or agent recommendations.
      - Then run the **Per-Artifact Review Gate** (see `openspec-workflow.mdc`) before creating the
        next artifact: STOP, post a short summary + this artifact's open questions, and offer the
        choice with the **AskUserQuestion tool** — Approve / Request changes, plus **Walk the questions
        (recommended) ONLY when this artifact actually has open questions**. When there are none, drop
        "Walk the questions" (never re-walk already-settled decisions — on an "empty" design that means
        an unraised (b) question, so raise it via the schema's `design` PIPELINE 7a instead). On "Walk
        the questions", handle them one at a time in chat (essence, options with
        trade-offs, labelled recommendation), writing each answer back into the artifact. On Approve,
        ask who approves and fill this artifact's `## Review & Approval` block (named human; never the
        agent). Do NOT create the next artifact until this one is approved. Exception: `grooming` — its
        review-and-approval IS the mandatory Grooming Approval Gate in step 7, so skip the generic gate
        for grooming and go straight there.
      - When `<artifact-id>` is `grooming`, STOP the generation loop immediately. Do NOT generate
        `specs`, `fsd`, or `isd` in the same pass until the human approval gate below is complete.

   b. **If an artifact requires user input** (unclear context):
      - Use **AskUserQuestion tool** to clarify
      - Then continue with creation

7. **Mandatory human grooming approval gate**

   Read `openspec/changes/<name>/grooming.md` fully. This gate is mandatory even if the agent believes
   the answer is obvious from documents. The agent may recommend an option, but it must not approve its
   own recommendation.

   **Block downstream generation** while any of these are true:
   - any Status `OPEN` or `BLOCKED` remains in Prerequisites & Access (P#), BA / Requirements
     Questions (Q#), Architecture / Tech Decisions (Q#), Change Request Decisions (CR#), or
     Scope & Offer Approval (O1/O2)
   - any item's `Decision` or `Approved by` cell is `—`/blank
   - any `Approved by` is `Agent`, `LLM`, `Cursor`, or another non-human approver
   - any Grooming Approval Gate checkbox is unchecked

   **If blocked, STOP downstream generation and immediately start a chat-based grooming session:**
   - Do not use the **AskUserQuestion** tool for grooming. Ask directly in chat.
   - Start with a short "Grooming Approval Required" summary grouped as Prerequisites, BA questions,
     Architecture/Tech decisions, CR dispositions, and the offer sign-off (O1/O2).
   - Then handle items one at a time. Do not dump the full questionnaire at once.
   - For each item, write a concise but complete prompt:
     - what the item is and why it matters;
     - what decision/approval is needed now;
     - the available options, with one short explanation of what each option means;
     - the agent recommendation, clearly labelled as a recommendation, not approval;
     - a direct request for the human answer/approval.
   - After the user answers, update `grooming.md` for that item, then ask the next item in chat.
   - Update `grooming.md`: fill `Decision` and `Approved by` (real name/role), set Status to
     `APPROVED` or `DEFERRED-APPROVED`.
   - Update `proposal.md` / Requirements Lock and `design.md` when an approved answer changes scope,
     disposition, approach, or external dependencies.
   - Check a Grooming Approval Gate box only when its item set is fully resolved; check all boxes
     only when the whole ledger satisfies the gate.

   **Do not proceed to specs/FSD/ISD until the user has approved the full grooming set.**
   If the user is not available or wants to postpone grooming, stop and say exactly what remains open.

8. **Continue BA/SA artifact generation after approved grooming**

   After the gate passes, resume the artifact loop:
   - Generate every ready artifact EXCEPT `technical-design`, `tasks` (Dev Planning/Implementation),
     the client-facing doc type the delivery scope excludes (`isd` on a functional change, `fsd` on an
     integration change), and any skipped conditional artifact (see the conditional rule above).
     `stories.md` is generated here by the projection script ALONGSIDE the FSD (a deterministic
     projection, never hand-authored); Jira tickets are created from it later at `/opsx:propose-approved`.
   - For `specs` and the client-facing doc, use only APPROVED or DEFERRED-APPROVED grooming decisions as
     source material.
   - **The client-facing doc's read-only sub-sections AND `stories.md` are SCAFFOLD-FIRST** — a
     deterministic projection, do NOT hand-author them. The functional flow uses
     `generate-functional-artifacts.mjs` (FSD); the integration flow uses
     `generate-integration-artifacts.mjs` (ISD). Produce them by:
     1. running the matching projection script FIRST — it bootstraps the working copy when absent, fills
        the client doc's read-only sub-sections (FSD: per-block User Story / Acceptance Criteria / Key
        behaviours; ISD: the same three under "## In Scope Functional Requirements", NO Visualization),
        AND writes `stories.md`, from the approved proposal slices (Story + Acceptance Criteria), the
        specs (Key behaviours table), and design (implementation notes), keeping the two files in sync:
        ```bash
        # functional change:
        node .cursor/skills/plan/openspec-workflow/scripts/generate-functional-artifacts.mjs openspec/changes/<name>
        # integration change:
        node .cursor/skills/plan/openspec-workflow/scripts/generate-integration-artifacts.mjs openspec/changes/<name>
        ```
     2. if it reports a validation error (critical), STOP and fix UPSTREAM — the proposal Delivery
        Breakdown slice (Story/AC) or the spec requirement (behaviour) — then re-run; never hand-fill or
        edit the script-owned content. Surface any warnings.
     3. then ENRICH only the client doc's non-owned sections: for the FSD the General FSD Information
        card, approval roster, Reference Materials, each block's Visualization, and framing; for the ISD
        the General ISD Information card, approval roster, Reference Materials, the agnostic Requirements
        table, the Integration Specification, and governance — leaving the script-owned sub-sections
        untouched.
     `stories.md` here is a projection whose acceptance-criteria reference and title refine once the
     client doc is enriched/published and the script is re-run; Jira tickets are created from it at
     `/opsx:propose-approved`.
   - After creating each of `specs` and the client-facing doc (`fsd`/`isd`), run the
     **Per-Artifact Review Gate** (openspec-workflow.mdc) before moving on: stop, summarize, offer
     Approve / Request changes (plus Walk the questions ONLY when open questions exist), and record approval. `specs`, `fsd`,
     and `isd` have NO in-file box (they leave the repo verbatim — specs merge into the timeless baseline;
     fsd/isd publish to Confluence), so approve at the stop itself. Named human only; never the agent.
   - After creating each artifact, re-run `openspec status --change "<name>" --json`.
   - Stop when the only remaining ready artifacts are `technical-design`, `tasks`, and skipped conditional
     BA/SA artifacts. This is the BA/SA handoff. Do NOT generate `technical-design` or `tasks`.

9. **Show final status**
   ```bash
   openspec status --change "<name>"
   ```

   This command does NOT commit the BA/SA artifacts, push, open a PR, or publish anything. Committing the
   BA/SA artifacts, opening the review PR (when the two-branch model is in use), and preparing the FSD/ISD
   wiki publish are the job of `/opsx:propose-submit`. The ONE local git action it performs is the
   discovery-time branch bootstrap (step 6): create `feature/<name>`, optionally `requirements/<name>`
   (offered only when an FSD/ISD exists; the human may decline), and, for an EXTEND, the single pristine-pull
   initial commit on `feature/<name>` — local-only, never pushed.

**Output**

After completing the BA/SA artifacts, summarize:
- Change name and location
- List of artifacts created with brief descriptions (note any conditional artifact skipped and why)
- Per-artifact review status: which artifacts are Approved and by whom (from each `## Review & Approval`)
- Grooming approval status (approved by whom; any DEFERRED-APPROVED items and their owners)
- What's ready: "BA/SA artifacts complete and ready for review."
- Prompt: "Run `/opsx:propose-submit` to commit these artifacts, open the review PR, and prepare the
  FSD/ISD wiki publish so the team and client can review and comment. After the proposal is approved,
  `/opsx:propose-approved` re-syncs the FSD + `stories.md` and creates the Jira tickets; `/opsx:plan`
  generates the implementation tasks."

**Artifact Creation Guidelines**

- Follow the `instruction` field from `openspec instructions` for each artifact type
- The schema defines what each artifact should contain - follow it
- Read dependency artifacts for context before creating new ones
- Use `template` as the structure for your output file - fill in its sections
- **IMPORTANT**: `context` and `rules` are constraints for YOU, not content for the file
  - Do NOT copy `<context>`, `<rules>`, `<project_context>` blocks into the artifact
  - These guide what you write, but should never appear in the output

**Guardrails**
- Stop after EVERY artifact for the **Per-Artifact Review Gate** (`openspec-workflow.mdc`): summarize,
  offer Approve / Request changes (plus Walk the questions ONLY when open questions exist), and get a named human's approval
  (recorded in the artifact's `## Review & Approval` marker) before generating the next one. Never
  self-approve. Grooming's gate (step 7) is grooming's review gate.
- Never publish to Confluence or create Jira tickets here, and never commit the BA/SA artifacts, push, or
  open a PR. Artifact commit + review PR and the FSD/ISD wiki publish prep are owned by
  `/opsx:propose-submit`; Jira story creation is owned by `/opsx:propose-approved`. Both are
  human-run/offer-only. The SINGLE exception is the discovery-time baseline bootstrap (step 6): creating
  `feature/<name>`, optionally `requirements/<name>` (offered only when an FSD/ISD exists; the human may
  decline), and the one pristine-pull initial commit on `feature/<name>` for an EXTEND — local-only, never
  pushed. That seed commit is the merge ancestor `/opsx:propose-submit` relies on in either branch model
  (see the `git-commit` skill's OpenSpec discovery-time bootstrap).
- Never hand-author the client doc's read-only sub-sections (FSD/ISD User Story / Acceptance Criteria /
  Key behaviours) OR `stories.md` — they are a deterministic projection produced by
  `generate-functional-artifacts.mjs` (FSD + stories) or `generate-integration-artifacts.mjs` (ISD's
  "## In Scope Functional Requirements" section + stories), each in one run. Run the matching projection
  script, fix any error upstream (proposal slice / spec), and enrich only the non-owned sections (FSD:
  General FSD Information, approval roster, Reference Materials, Visualization, framing; ISD: General ISD
  Information, approval roster, Reference Materials, Requirements table, Integration Specification,
  governance). Jira tickets are created from `stories.md` at `/opsx:propose-approved`.
- Create the BA/SA artifacts, then STOP before `technical-design` and `tasks` — never generate Planning artifacts here (that is `/opsx:plan`)
- `design` and `proposal` are standard pre-grooming steps for every change (discovery -> design -> proposal -> grooming); the client-facing doc follows the delivery scope (`functional` -> FSD only, `integration` -> ISD only — never both) and is generated only when the change warrants it; `specs` is the required BA/SA output
- Always read dependency artifacts before creating a new one
- Never self-resolve or self-approve grooming items. Human answer + human approval is required before
  specs/FSD/ISD.
- If context is unclear, ask the user. Do not use "reasonable decision" momentum to close questions or prerequisites.
- If a change with that name already exists, ask if user wants to continue it or create a new one
- Verify each artifact file exists after writing before proceeding to next

**Failure Recovery**
Enter this section only after a command in the normal workflow fails.
- Missing OpenSpec CLI: run `openspec --version 2>&1`, then tell the user to install it with
  `npm install -g @fission-ai/openspec@latest`.
- Project not initialized: tell the user to run `openspec init` once, then rerun `/opsx:propose`.
- Projection script (`generate-functional-artifacts.mjs` for FSD, `generate-integration-artifacts.mjs`
  for ISD) fails: on a join validation error it lists what to fix upstream (the proposal slice or the
  spec) — fix and re-run; if it cannot resolve the `fsd.md` / `isd.md` template, pass `--template <path>`
  or install the OpenSpec CLI (`npm install -g @fission-ai/openspec@latest`).
