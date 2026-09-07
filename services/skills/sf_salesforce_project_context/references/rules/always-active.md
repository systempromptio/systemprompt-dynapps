# Core Rules (Standing Instructions)

These rules are always active when working in any **Salesforce Core** project that uses this agent.

> Treat every section as a non-negotiable default unless the project's `docs/PROJECT_SCHEMA.md` or a story explicitly overrides it (and the override is captured in writing).

---

## Branch and deploy practice (every feature or bug)

For **every feature and every bug**:

1. **Branch:** Work happens on a branch named `feature/<STORY-ID>` or `bug/<TICKET-ID>` (e.g. `feature/PROJ-123`, `bug/PROJ-456`). Create the branch from `develop` (or `main` where there is no `develop`) before implementing. Do not commit feature/bug work directly to `develop` or `main`.
2. **Story folder:** If the work has a story or ticket ID, use and update **`stories/<STORY-ID>/`** (`requirement.md`, `design-note.md`, `deployment.md`, `qa-test-request.md`) in the same change as code. See `stories/README.md`.
3. **Deploy:** Deploy from the branch to a non-prod sandbox using an org alias (e.g. `--target-org <sandbox-alias>`). Production deploys only from `main` (or a release branch) with explicit approval and a release tag.
4. **Merge:** Changes are merged via PR into `develop` (or `main`). After merge, deploy the integration branch to the appropriate sandbox.

Full practice: **`docs/BRANCH_AND_DEPLOY_PRACTICE.md`**. Apply this for every feature and every bug; no exceptions.

---

## Story / Ticket Intake (Auto-Trigger)

When the user pastes a **Jira story, user story, or any feature requirement**, immediately follow the full workflow defined in `.cursor/rules/story-intake.md`:

1. Parse the story (What, Which objects, Acceptance criteria, Constraints).
2. Route to the correct skill agent(s) using the decision tree.
3. State assumptions and open questions before writing code.
4. Generate required documents under **`stories/<story-id>/`**: `requirement.md`, `design-note.md`, `deployment.md`, `qa-test-request.md` (see `stories/README.md`).
5. Implement using the correct agent persona standards.
6. Always conclude with QA/Test Engineer outputs (test class + test plan + manual test cases).

---

## Project context and accuracy

- **Always use the project schema** for object and field API names. When implementing or reviewing Salesforce metadata, Apex, LWC, or Flows that touch custom objects/fields, read **`docs/PROJECT_SCHEMA.md`** and use the exact API names listed there. Do not invent or guess field or object names.
- When adding new custom objects or fields, **update `docs/PROJECT_SCHEMA.md` in the same change** (description, type, picklist values if applicable, relationships).
- **Page layouts:** When adding new custom fields to an object (standard or custom), add or update that object's **page layout(s)** in the same change so the fields appear on the record page after deploy. Include layout in the deployment order and document **page layout assignment** in the story's deployment runbook (assign the new layout to profiles in Setup after deploy so users see the fields).
- For maximum accuracy, use the **`Salesforce_Project_Context`** skill (`.cursor/skills/Salesforce_Project_Context.md`) — fill it in for the current project so the agent knows this project's automations and conventions.

---

## Architectural priority

- **Always evaluate** whether a requirement can be met via **Flow, Validation Rule, Custom Metadata, or Standard Configuration** before suggesting Apex.
- Prefer declarative (Flow, validation rule, formula, custom metadata-driven config) when they satisfy the requirement.
- Use Apex when:
  - Logic spans multiple objects with complex bulk processing.
  - Async processing (Batch, Queueable, Schedulable) is required.
  - Callouts, custom security/sharing, or governor-limit-sensitive operations are needed.

---

## Apex standards

- **Triggers:** One trigger per object, using a **Handler pattern** (trigger delegates to a handler class). No business logic in the trigger body.
- **Bulkification:** No SOQL or DML inside loops. All logic must work with lists of records.
- **Test coverage:** Minimum **90%** with **meaningful assertions** (no `System.assert(true)` or empty assertions).
- **Test data:** Test methods must satisfy **all active validation rules** in the target org. Use a `TestDataFactory` (or equivalent) and centralise required-field defaults so tests remain stable when validation rules are added.
- **Security:** `with sharing` by default. Use `Security.stripInaccessible()` or `WITH SECURITY_ENFORCED` / `AccessLevel` checks for DML/query operations.
- **Static flag hygiene:** Static `Boolean` flags used for re-entrancy or bypass control must be **explicitly initialized to `false`** (never left null) — unboxing a null `Boolean` in `if (flag)` causes runtime errors.

---

## LWC standards

- Prefer **Lightning Data Service (LDS)** over Apex wire adapters when CRUD is the only need.
- Follow **SLDS (Salesforce Lightning Design System)** for all UI; no ad-hoc CSS that diverges from the design system.
- ESLint-clean; avoid direct DOM manipulation unless necessary.

---

## Declarative (Flow) standards

- Build robust **record-triggered** or **screen Flows** with explicit **error handling** (Fault paths) on every element that can fail.
- **Before-save** flows for fast field updates on the same record (no DML); **after-save** flows when related records are updated or async actions are needed.
- Use consistent naming (`Object_When_What`, e.g. `Opportunity_BeforeSave_Stage_Defaults`) and version intentionally.
- **Avoid hardcoded IDs**; use Custom Metadata Types or named credentials for environment-specific values.
- Screen Flows: the first screen must have at least one of `allowBack` or `allowFinish` set to `true` (Salesforce requires it).

---

## Validation rules

- Prefer narrow, single-purpose rules over multi-purpose mega-rules; combine related ACs in one rule only when they share the same activation condition and error message context.
- For stage/status transitions use `ISPICKVAL(Status, "X") && ISBLANK(field)`.
- For format checks (email, phone, name) use `REGEX(...)` with documented patterns; document each pattern's intent in the story's design-note.
- Standard compound-name fields (e.g. `FirstName`) cannot be set required via metadata; enforce required via a validation rule.
- Place blocking error on the most relevant field where possible (`errorDisplayField`); use top-of-page only for cross-field rules.

---

## Custom Metadata-driven configuration

When business rules are likely to change without a deploy, prefer a **Custom Metadata Type** (CMT) configuration model over hardcoding:

- Mapping tables (e.g. source value → target value, field → record type).
- Eligibility/condition rules (logic + atomic conditions referenced by an `Order__c`).
- Per-environment defaults that should not require a code change.

Document the CMT and its records in `docs/PROJECT_SCHEMA.md`. See `.cursor/skills/Custom_Metadata_Driven_Config.md`.

---

## Profiles, permission sets and applications

- **Permission set groups** (PSG) do not contain `<objectPermissions>`; object/field permissions live on the member permission sets.
- **Salesforce permission dependencies** (e.g. *Modify All* implies *Delete* and *View All*; `ManageUsers` requires `DelegatedTwoFactor` + `MonitorLoginHistory`) must be normalized in source. Deploys fail when a dependent permission is missing.
- **App Launcher visibility:** A custom application appears in the App Launcher only when each profile sets `<applicationVisibilities>` `visible=true`. **Retrieve the full profile from the org before editing** (especially `Admin`); deploying a minimal profile file can clobber unrelated permissions.
- **CustomApplication + UtilityBar FlexiPage** must be deployed together: the app's `<utilityBar>` value references a FlexiPage developer name; if the FlexiPage is missing, the app deploy fails.

---

## Security

- Enforce **`with sharing`** by default in Apex.
- Use **`Security.stripInaccessible()`** or `AccessLevel` checks for DML operations.
- Never store secrets in the repository; use Named Credentials, GitHub Environments/Secrets, or a secrets vault.
- Use **Custom Permissions** (assignable via permission set) for bypass logic instead of hardcoded profile checks where possible — they are cleaner and assignable to integration users.
- Never run destructive changes outside an approved destructive-changes workflow.

---

## Delivery and deployment

- Never deploy to production without an approved release tag, environment approval, and rollback steps.
- Prefer unlocked packages for reusable modules; use metadata deploy where packaging is not feasible.
- Treat managed package upgrades as controlled events with explicit testing and rollback.
- **Sandboxes:** prefer `--test-level NoTestRun` when validating non-Apex changes that would otherwise be rolled back by unrelated org validation/coverage failures (document why in the story's `deployment.md`).
- **Production / release branches:** always use `RunLocalTests` (or stricter) on promotion.
- Prefer **manifest-based deploys** (`--manifest manifest/<story>-package.xml`) for stories that touch many metadata types — the CLI's `--metadata Type1,Type2` joined-argument syntax is fragile and a common cause of "stuck" or silently-incorrect deploys.

---

## Story folder and doc freshness (no manual command required)

- **One folder per user story:** All documents for a user story live under **`stories/<STORY-ID>/`** (e.g. `stories/PROJ-123/`). See **`stories/README.md`** for the folder pattern (`requirement.md`, `design-note.md`, `deployment.md`, `qa-test-request.md`).
- **Whenever you change anything that affects a user story** (deployment fixes, schema changes, new or changed components, flow/metadata updates):
  1. Update **`stories/<STORY-ID>/design-note.md`** — components, deployment order, schema impact; set **Last updated:** date.
  2. Update **`stories/<STORY-ID>/deployment.md`** if deploy order, commands, or post-deploy steps changed.
  3. Update **`stories/<STORY-ID>/qa-test-request.md`** if test steps or prerequisites changed.
  4. Update **`docs/PROJECT_SCHEMA.md`** if new or changed custom objects/fields.
- Apply these updates **in the same change** that modifies the story's code or metadata. No separate manual step is required.
- **Video capture for manual UI testing:** Each story's `qa-test-request.md` must require that **final manual UI testing is recorded on video**, executing all detailed test cases in order with test case IDs identifiable; video file in the story folder or link documented in `qa-test-request.md` (see `stories/README.md`).

---

## What this agent must never do

- Hardcode org-specific IDs, usernames, employee names, customer names, or sandbox aliases in source.
- Deploy to production without an approved release path.
- Commit credentials, certificates, or any secret to the repository.
- Generate Apex without a test class targeting **≥90% coverage** with meaningful assertions.
- Add new custom fields/objects without updating `docs/PROJECT_SCHEMA.md` in the same change.
- Skip the story folder pattern when a story/ticket ID exists.
