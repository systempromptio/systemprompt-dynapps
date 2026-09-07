# Story Intake & Agent Routing

When the user pastes a Jira story, user story, or any feature requirement, follow this exact workflow.

---

## Step 1 — Parse the Story

Extract and confirm:

- **What** is being built (feature / automation / UI / integration / fix).
- **Object(s)** involved (standard and custom; use API names from `docs/PROJECT_SCHEMA.md` where they exist).
- **Fields** involved — list exact API names; for new fields/objects, note "to be added" and ensure they are added to the schema in the same implementation.
- **Acceptance criteria** (if present) — each AC becomes a row in the deployment / QA matrix.
- **Constraints** (performance, security, licensing, compliance, blackout windows).

**Schema check:** Before implementation, read `docs/PROJECT_SCHEMA.md`. Use existing object/field API names as-is; do not guess or invent. If the story introduces new custom objects or fields, the design note and implementation must add them to the schema.

---

## Step 2 — Route to the Correct Skill Agent

Use the decision tree below. A story may require **multiple agents in sequence**.

```
Does it require Apex logic, Triggers, Batch, or Queueable?
  -> YES --- Assign: Apex_Architect
  -> NO  --|

           Does it involve records that could be handled by Flow, Validation Rule, or Custom Metadata?
             -> YES --- Assign: Declarative_Lead   (Flow / Config / CMT before Apex)
             -> NO  --|

                      Does it need a UI, LWC, or Lightning page customization?
                        -> YES --- Assign: UI_UX_Specialist
                        -> NO  --- Assign: Feature_Lead (design note only)

After implementation by any agent above:
  -> ALWAYS assign: QA_Engineer  (test class + manual test plan + video capture requirement)
```

### Agent quick-reference

| Signal in the story | Assign |
|---|---|
| "trigger", "handler", "batch", "async", "callout", ">1000 records" | **Apex_Architect** |
| "screen flow", "approval", "automation", "no-code", "record-triggered", "validation rule" | **Declarative_Lead** |
| "component", "LWC", "tab", "page", "UI", "button", "display", "Dynamic Forms" | **UI_UX_Specialist** |
| Testing, coverage, regression, QA sign-off | **QA_Engineer** |
| Epic breakdown, design note, architecture only | **Feature_Lead** |
| Code review, risk, deploy approval | **PR_Reviewer** |
| Release, environment promotion, rollback | **Release_Manager** |
| Sharing, FLS, secrets, compliance review | **Security_Compliance_Lead** |
| Docs, runbooks, release notes | **Docs_Curator** |

---

## Step 2b — Branch (per practice)

Before implementing, the work must happen on a branch: **`feature/<story-id>`** (e.g. `feature/PROJ-123`). If the user has not already created it, instruct them to create and checkout the branch (see **`docs/BRANCH_AND_DEPLOY_PRACTICE.md`**). All commits for this story go on that branch.

---

## Step 3 — Generate Required Documents

After routing, produce the following documents automatically:

### For every story

1. **`stories/<story-id>/requirement.md`** — Saved copy of the original story text (verbatim).
2. **`stories/<story-id>/design-note.md`** — Design note including:
   - **Schema impact:** Objects and fields used (with API names from `docs/PROJECT_SCHEMA.md`) or to be created (and then added to the schema).
   - Impacted components & metadata.
   - Agent(s) assigned and rationale.
   - Deployment ordering constraints.
   - Rollback strategy.
   - **Last updated:** date (set whenever the story's implementation or docs change).
3. **`stories/<story-id>/deployment.md`** — Deployment order, commands, post-deploy steps (page-layout assignment in Setup, permission set assignment, manual activation if any).
4. **`stories/<story-id>/qa-test-request.md`** — Manual QA test cases (detailed steps and expected results), Apex test command, and **video capture**: a required section stating that final manual UI testing must be recorded on video, executing all manual test cases in order, with test case IDs identifiable; video file or link stored in the story folder (see `stories/README.md`).

### Per agent persona

| Agent | Additional outputs |
|---|---|
| Apex_Architect | Trigger + Handler + Test class stubs with TODOs, bulkification notes, security approach |
| Declarative_Lead | Flow element-by-element plan, fault path notes, custom metadata needs |
| UI_UX_Specialist | LWC scaffold (html/js/css/meta), LDS vs Apex wire justification, accessibility notes |
| QA_Engineer | Test class (positive, negative, bulk), Jest scaffold for LWC, manual test plan |
| Security_Compliance_Lead | Security review summary, FLS/sharing impact, secrets check |

---

## Step 4 — State Your Assumptions

Before writing any code, explicitly list:

- Which agent persona you are assuming.
- What Salesforce objects/fields are impacted (and whether they exist or must be added to the schema).
- Whether **Flow / Validation Rule / Custom Metadata / Configuration** was considered first (Apex priority rule).
- Any open questions that need clarification from the requester.

---

## Step 5 — Implementation

Follow the Core Rules in `always-active.md` throughout:

- **Use exact API names** from `docs/PROJECT_SCHEMA.md` for all custom objects and fields; when adding new ones, update the schema in the same change.
- Use the **`Salesforce_Project_Context`** skill (`.cursor/skills/Salesforce_Project_Context.md`) for this project's automations and conventions.
- One trigger per object with handler pattern.
- No SOQL/DML inside loops.
- 90%+ meaningful test coverage.
- `with sharing` + `Security.stripInaccessible()` by default.
- SLDS for all UI; LDS over Apex wire where possible.
- Validation rules: prefer narrow, single-purpose rules; use `ISPICKVAL` / `REGEX` patterns documented in `.cursor/skills/Validation_Rule_Patterns.md`.
- For app + utility-bar work, deploy the app and utility FlexiPage **together** and update `applicationVisibilities` per profile (see `.cursor/skills/App_UtilityBar_Pattern.md`).

---

## Step 6 — Story Folder Updates (always paired with code changes)

Whenever you change code or metadata for a story, update — **in the same commit**:

1. `stories/<story-id>/design-note.md` (set **Last updated:** date).
2. `stories/<story-id>/deployment.md` if deploy order or commands changed.
3. `stories/<story-id>/qa-test-request.md` if test steps changed.
4. `docs/PROJECT_SCHEMA.md` if custom objects/fields changed.

No separate manual step is required.
