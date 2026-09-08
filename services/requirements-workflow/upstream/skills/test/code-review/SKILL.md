---
name: code-review
description: Use when the user asks for a code review, wants to review changes before merge, asks to check code quality, or mentions reviewing a PR, branch, or diff.
---

# Code Review

You are a **Senior Reviewer**. For project-specific context (tech stack, integration branch, environments), see `project-info.mdc`. Coding conventions live in `.cursor/rules/architecture/coding-standards.mdc`.

## Review Workflow

### 1. Determine Scope

Ask the user or infer which diff to review:

- **Branch diff:** `git diff <integration-branch>...HEAD` (all changes in the current branch vs the project's integration branch — commonly `develop` or `main`; see `project-info.mdc`)
- **Staged changes:** `git diff --cached`
- **Working changes:** `git diff`
- **Specific files:** review only files the user mentions

Run the appropriate git command, then read the changed files for full context.

**Scope discipline:** Only review code within the diff. Do not flag pre-existing issues outside the change scope unless they are directly affected by or related to the changes.

### 2. Gather Requirements Context

Extract the Jira ticket key from the branch name (pattern: `feature/PROJ-<number>` or `fix/PROJ-<number>`) or ask the user for it.

If a ticket key is available, run these in parallel:

```bash
node .cursor/skills/operate/atlassian/scripts/jira.mjs get-issue <key>
node .cursor/skills/operate/atlassian/scripts/jira.mjs remote-links <key>
```

From the issue: capture **summary**, **description**, and **acceptance criteria**.

From remote links: identify linked **FSD/ISD/TSD** pages. Search `project-context` with
`source_type: ["confluence"]` and open the cited `inbox/confluence/...` file/lines. If the active OpenSpec
change has an FSD/ISD working copy, read that too. Use the committed design assets under the change
`assets/` folder for approved visual context.

If no ticket key is extractable and the user hasn't provided one, **skip this step** — do not block the review. Mention in the output that requirements verification was skipped due to missing ticket reference.

**What to extract from requirements:**

- Acceptance criteria (functional requirements the code must satisfy)
- UI specifications from FSD screenshots (layout, states, copy)
- Integration details from ISD (API contracts, data flows, error handling)
- Edge cases or constraints mentioned in the spec but not obvious from code alone

### 3. Quality Gate (run before reviewing)

Run lint and type checks per [quality-gate](mdc:.cursor/rules/workflow/quality-gate.mdc) rule on all modified workspaces. Report any errors as **Blocker** findings if introduced by the change. Pre-existing errors: note but don't block.

### 4. Code Review Rules

If the project ships a shared review checklist (commonly `docs/development/review-rules.md` — the checklist used by both this skill and any gitStream automated reviews), read it and apply **all** rules from it. If it is absent, proceed with the generic checks in this skill.

### 5. Requirements Compliance (when requirements context is available)

This section is in addition to the project's shared review checklist (if present):

**Integration Completeness — New CMS-Rendered Components**

When the diff introduces a new React component that is rendered from CMS data (Page Designer or a headless CMS), verify the project's full component-registration pipeline. If the project documents this pipeline in a dedicated rule (see `project-info.mdc` → Related Rules), follow it; otherwise verify the generic contract below:

1. Schema/type definition for the component's content model — exists?
2. Type guard or validator for the content payload — exists?
3. Aggregation/mapping wired into the project's content-to-component mapping — created AND registered?
4. Component imported and added to the project's component map/registry?
5. The `component` key emitted by the content layer matches the registry key exactly?

Missing any step = component silently never renders. Mark as **Blocker** if any step is missing.

- Does the implementation satisfy all acceptance criteria from the Jira ticket?
- Does the UI match the approved design assets from the FSD?
- Are integration contracts from the ISD respected (API endpoints, payloads, error codes)?
- Are edge cases mentioned in the spec handled (empty states, error states, loading states)?
- Are there acceptance criteria that have no corresponding code change?

### 5.5. Delivery Readiness Checklist

Beyond code quality and AC mapping, verify that the implementation is **delivery-ready** — not just "code exists" but "feature works end-to-end". Output a checklist table (see Output Format below).

**A. Feature Completeness**

- Can the user complete the full happy path without leaving the feature?
- Are transitions between states handled? (e.g., add-to-cart -> mini-cart update -> cart page reflects)
- If multi-step flow: is back/forward navigation handled?
- Key question: "If I were a user, could I accomplish the goal end-to-end without hitting a dead end?"

**B. UI States Coverage**

For each new/modified component, verify these states are handled:

- `loading` — skeleton or spinner, NOT blank
- `empty` — explicit empty state UI, NOT hidden
- `error` — user-visible feedback (toast/inline), NOT just console
- `populated` — normal render
- `SSR` — no loading-only tree that skips query registration (per `ssr-query-registration.mdc`)

For mutations: error path shows user feedback, not silent fail.

**C. E2E Test Presence**

- Search `playwright/tests/` for specs covering the modified user flow
- New page/route: must have at least smoke-level Playwright spec
- Existing flow modified: check existing spec still covers the change
- No spec exists: flag as `[TEST_GAP]` with suggested test outline

**D. A11y Verification**

- New interactive element (button, form, modal, carousel): keyboard nav + aria attributes present
- New page/route: should be in axe-core smoke list (if available)
- ESLint jsx-a11y passes with zero new suppressions
- Flag: `[A11Y_GAP]` if interactive element lacks aria-label/role or keyboard handler

**E. Design Verification**

- If PR touches UI component styling (Tailwind classes, layout, spacing):
  - Cross-reference with the committed design assets under the change `assets/` folder
  - Check: spacing, colors, typography, border-radius, breakpoint behavior
  - Screenshot comparison where possible
  - Flag mismatches as `[DESIGN_DRIFT]` with the design asset reference
- Utility-only or no design asset available: mark "N/A"

**F. 3PI Integration** (only when diff touches 3PI code)

- Consent: OneTrust category assigned? Withdrawal cleans up?
- Error handling: Timeout + retry? User-visible degraded state?
- Locale: Passed per vendor spec? Fallback?
- SSR: No double-init? No flash before consent?
- E2E coverage: Playwright test for consent flow + degraded state?
- None applies: mark "N/A"

**G. Edge Cases & Boundaries**

- Numeric: 0, 1, max tested? (quantity, pagination, stock)
- Mobile: If conditional rendering by viewport — both paths covered?
- Concurrent: Race conditions in async flows?
- Idempotency: Can the action be safely repeated?
- Reference: [fe-edge-cases-checklist](mdc:.cursor/rules/workflow/fe-edge-cases-checklist.mdc)

**H. Spec Assumptions Log**

- If behavior is found that is NOT documented in FSD/ISD:
  - Log as `[ASSUMPTION]: "<what was assumed>" — not specified in <doc>`
  - NOT a blocker — signal for BA/PM to confirm or create spec update
  - Prevents "client expects X, dev implemented Y" bugs

### 6. Project-Specific Checks

Before reviewing, read the project coding conventions from `.cursor/rules/architecture/coding-standards.mdc` (source of truth — always current). Check each changed file against those conventions. Violations are **blockers or high** depending on severity.

**Project lessons learned (blocking for "ready to merge" ratings)**

- **OpenSpec hygiene:** If the branch touches or fulfills an OpenSpec change under `openspec/changes/<name>/`, the review must either (a) note that the change should be **archived** post-merge via `/opsx:archive`, or (b) state why it stays active. Silence here is a process miss.
- **Documentation hygiene:** If the diff includes `.md` or `.mdc` files, apply §13 from `review-rules.md` — check for local machine paths (`/Users/...`), broken relative links (wrong `../` depth), stale hardcoded counts, IDE-specific URLs, and incorrect `npm run` command scope. These leak personal info or break navigation.

## Output Format

### Requirements Coverage (when ticket/spec available)

Map each acceptance criterion to the code that fulfills it. Use a checklist:

```
**Ticket:** PROJ-XXX — <summary>
**Spec:** <FSD/ISD page title if found>

- [x] AC 1: <criterion> → implemented in `file.tsx:line`
- [x] AC 2: <criterion> → implemented in `file.tsx:line`
- [ ] AC 3: <criterion> → **NOT FOUND in diff** ← flag as gap
```

If the FSD includes design screenshots, note whether the UI structure in the code matches the design (component hierarchy, states, copy text). Flag visible mismatches.

If requirements context was unavailable, add a single note:
`Requirements verification skipped — no Jira ticket linked to this branch.`

### Delivery Readiness Checklist

| Check | Status | Details |
|-------|--------|---------|
| Feature completeness | PASS/GAP | ... |
| UI states (L/E/Em/P) | PASS/GAP | ... |
| E2E tests | PASS/TEST_GAP | ... |
| A11y | PASS/A11Y_GAP | ... |
| Design match | PASS/DRIFT/N-A | ... |
| 3PI integration | PASS/GAP/N-A | ... |
| Edge cases | PASS/GAP | ... |
| Spec assumptions | NONE/LOGGED | ... |

**Gaps requiring attention:** (list items with GAP/DRIFT/TEST_GAP/A11Y_GAP status)
**Assumptions logged:** (items where behavior is not specified — needs BA/PM confirmation)

### Strengths

Call out what's done well — specific files, patterns, decisions. Good reviews build trust and show the reviewer understood the code, not just scanned for issues.

### Executive Summary

3-6 bullet points highlighting the most critical issues.

### Blocking Issues (Must Fix Before Merge)

```
**[Blocker]** File:line — Issue description

- **Why:** Explanation of the problem
- **Fix:** Concrete patch or pseudocode
```

### Important Improvements (Should Fix Soon)

```
**[High]** File:line — Issue description

- **Why:** Explanation
- **Fix:** Suggestion
```

### Nice-to-Have

```
**[Medium/Low]** File:line — Issue description
- **Fix:** Suggestion
```

### Quick Wins

Fast fixes with high impact — low effort, high value changes.

### Suggested Refactors

Larger improvements with examples of better patterns. For god file/component issues, include a concrete decomposition sketch (what to extract, where to put it, how pieces connect).

### Verdict

End every review with a clear merge assessment:

- **Ready to merge** — no blockers, all acceptance criteria covered, code is solid
- **Ready with fixes** — minor issues listed, none block functionality, requirements met
- **Not ready** — blockers must be resolved first (code issues or missing requirements)

Include 1-2 sentence reasoning. If acceptance criteria are partially implemented, this is a blocker unless the ticket is explicitly scoped to partial delivery.

## Clean Code Standards

Generic, project-agnostic quality bars. For project-specific context (stack, integration branch) see
`project-info.mdc`. Storefront Next (Vite + React 19) does **not** auto-import React or hooks — import
them explicitly; flag redundant explicit imports only if the project configures an auto-import setup.

### Thresholds

Flag when exceeded:

| Metric | Review (Medium) | Blocker (Critical) |
|---|---|---|
| File LOC | > 300 | > 500 |
| Component JSX LOC | > 150 | > 250 |
| Component props | > 8 | > 12 |
| Function LOC | > 40 | > 80 |
| Function parameters | > 4 | > 6 |
| Nesting depth | > 3 | > 5 |
| Module exports (unrelated) | > 10 | > 15 |
| Hook responsibilities | > 2 | > 3 |

### Component Patterns

- **Presentational** components: receive props, render UI, no side effects.
- **Container** components: fetch data, manage state, delegate rendering.
- Avoid mixing both in a single component — if a component fetches AND renders complex UI, split it.
- Hooks should follow single-concern: `useProductData` (fetch), `useProductForm` (form state),
  `useProductTracking` (analytics) — not `useProduct` doing all three.

### File Organization

- One primary export per file (component, hook, utility, or type).
- Co-located tests: `component.test.tsx` next to `component.tsx`.
- Related types in the same file or a sibling `types.ts` — not a distant shared types barrel.
- Index files for the public API of a folder only — no deep re-exports.

### Security Expectations

- No secrets or credentials in source code — use environment variables.
- All user input must be sanitized before rendering as HTML.
- API responses should be validated/typed before use — never trust external data shape.
- Authentication tokens must not be logged or exposed in error messages.
- Third-party scripts must be reviewed before inclusion.

## Severity Calibration

Assign severity honestly — inflated severity wastes developer time and erodes trust.

| Severity | Meaning                                        | Examples                                                              |
| -------- | ---------------------------------------------- | --------------------------------------------------------------------- |
| Blocker  | Breaks functionality, security risk, data loss | XSS, `any` in public API, race condition, missing AC, god file > 500 LOC |
| High     | Significant impact, should fix before merge    | Missing error handling, architecture violation, UI mismatch vs design asset, god component |
| Medium   | Improvement worth making, not urgent           | Magic numbers, missing key prop, minor spec deviation, naming issues  |
| Low      | Nitpick, style preference                      | Naming suggestion, minor readability                                  |

Do NOT mark nitpicks as Blocker/High. If unsure, go one level lower.

## Rules

- Every issue: severity + file:line + why + fix
- If context is missing, mark as "Assumption" — do not invent context
- Propose minimal changes with maximum effect — don't rewrite entire code
- Reference project conventions from `.cursor/rules/architecture/coding-standards.mdc`
- Use English for all technical terms and code examples

## After Review: Verify Completeness

Before presenting the review, confirm:

1. Every changed file in the diff was actually reviewed (list them)
2. No file was skimmed or skipped
3. Every acceptance criterion was mapped to code or flagged as missing
4. The verdict matches the issues found (no blockers + all ACs covered = ready, otherwise not ready)
5. Pre-checks in §5 (OpenSpec) were applied — if skipped, say so explicitly

If you skipped files, couldn't review something, or couldn't verify requirements, state it explicitly.

## After Review: Process Retrospective

If the review uncovered a bug, silent failure, missing integration, or undocumented pattern — run a brief root cause analysis per [process-retrospective](mdc:.cursor/rules/workflow/process-retrospective.mdc):

1. Was this a one-off mistake or a repeatable pattern?
2. If repeatable — what rule, skill, or schema update would prevent it next time?
3. Propose and implement the structural fix (don't just note it).

## Example

````
### Strengths
- Clean separation of search logic into a custom hook (use-search.ts)
- Good error boundary usage in the product listing page
- Consistent i18n usage across the feature

### Executive Summary
- TypeScript strict violation: `any` type in API service
- God component: product listing exceeds the JSX threshold
- Overall solid implementation, minor issues only

### Blocking Issues
**[Blocker]** app/services/api.ts:23

Using `any` type violates TypeScript strict mode.

- **Why:** Loses type safety, can hide runtime errors
- **Fix:**
  ```typescript
  // Before
  function processData(data: any) { ... }

  // After
  function processData(data: SearchResponse) { ... }
````

### Important Improvements

**[High]** app/pages/product-listing.tsx:1

Component JSX exceeds the 250-line Blocker threshold (see Clean Code Standards).

- **Why:** Hard to test and reason about; mixes data-fetch and presentation
- **Fix:** Extract the results grid and filters into presentational child components

### Verdict

**Not ready** — one Blocker (`any` type) must be fixed. After that, ready to merge.
```
