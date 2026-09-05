# SF Validate Rules

Runs the sixteen Salesforce Core governance checks over a given set of Apex, LWC, and Flow files and returns one violations table plus a pass/fail verdict. It mirrors the deterministic checks in the team's `sf-validate_rules` MCP tool so a developer gets the same answer locally before review, with a line number and a fix for every finding.

## Use when

- Before a PR, before a sandbox deploy, or after `sf_apex_build` / `sf_declarative_build` produce files.
- The user asks "does this pass our rules", "lint this Apex", "check this flow".
- If the `sf-core` MCP server is present in the session, call its `sf-validate_rules` tool with the same files and reconcile; otherwise apply the checks below by hand.

## Inputs

| Input | Required | Notes |
|---|---|---|
| File paths or pasted code | yes | `.cls`, `.trigger`, LWC `.js`/`.html`, `.flow-meta.xml` or flow JSON. |
| Story ID | no | Enables the process-evidence rows (branch name, story folder, schema freshness). |
| Language per file | no | Inferred from extension when absent. |

## Workflow

1. **Ground in the knowledge bank.** Call `search_knowledge` with `{"query": "Salesforce Core always-active rules Apex LWC Flow standards", "top_k": 5, "source_types": ["project_docs"]}` and, when a story ID is given, `{"query": "<STORY-ID>", "top_k": 3, "source_types": ["project_tickets"]}`. Cite hits in the report header as the rule source. If unavailable, state it and use the check table below as the sole source.
2. **Read every file in full.** Never grade from a diff.
3. **Apex — six checks**, in this order:

| # | Rule | Severity | Detect | Fix |
|---|---|---|---|---|
| A1 | `no-hardcoded-ids` | error | quoted 15 or 18 char alphanumeric literal | CMT record, custom label, or query by DeveloperName |
| A2 | `bulkification-required` | error | `insert/update/delete/upsert/undelete` or `[SELECT` inside `for`/`while` | collect in a list, one DML/SOQL outside the loop |
| A3 | `test-class-required` | review | no `@isTest` class accompanies the change | add `<Class>Test` targeting ≥90% |
| A4 | `naming-convention` | warning | class name not `^[A-Z][a-zA-Z0-9]*$` | PascalCase |
| A5 | `exception-handling` | warning | `try` without `catch` | catch, log, and surface via `addError` or rethrow a typed exception |
| A6 | `soql-injection-risk` | error | `String query = ... + var` | bind variables `:var` or `String.escapeSingleQuotes` |

   Also flag `System.assert(true)` (review, `meaningful-assertions`), a class missing `with sharing` (review, `security-sharing`), and a static Boolean flag declared without `= false` (review, `static-flag-hygiene`).

4. **LWC — five checks**:

| # | Rule | Severity | Detect | Fix |
|---|---|---|---|---|
| L1 | `no-direct-dom` | error | `document.`, `getElementById`, `querySelector` on `document`, `innerHTML =` | `this.template.querySelector`, reactive properties |
| L2 | `use-wire-adapter` | warning | data retrieval without `@wire` | `@wire(getRecord)` or LDS adapters; imperative Apex only with a reason |
| L3 | `use-slds` | warning | no SLDS classes or base components | `lightning-*` base components, SLDS utility classes |
| L4 | `accessibility-missing` | info | interactive element without `aria-*`, `role`, or label | add label, role, keyboard handler |
| L5 | `missing-lifecycle` | info | no `connectedCallback` / `renderedCallback` / `disconnectedCallback` | add the hook that owns setup and teardown, or state none is needed |

5. **Flow — five checks**:

| # | Rule | Severity | Detect | Fix |
|---|---|---|---|---|
| F1 | `invalid-json` / unparseable XML | error | parser fails | repair the definition |
| F2 | `missing-start` / `missing-flow-name` / `empty-flow` | error | no Start element, no name, zero elements | add the Start element and at least one element |
| F3 | `flow-too-complex` | warning | more than 50 elements | split into subflows |
| F4 | `async-timeout` | warning | async action or callout with no timeout policy | set a timeout, add a fault path |
| F5 | `variable-naming` | info | variable not camelCase `^[a-z][a-zA-Z0-9]*$` | rename |

   Also flag a failable element with no `faultConnector` (warning, `missing-fault-path`) and a validation rule mixing several boolean operators (warning, `single-purpose-rule`).

6. **Process evidence** when a story ID is given (severity `review`): branch is `feature/<STORY-ID>` or `bug/<ID>`; `stories/<STORY-ID>/` holds all four files; `docs/PROJECT_SCHEMA.md` changed if any field or object was added; no production deploy without a release tag.
7. **Verdict.** `FAIL` if any `error` exists; otherwise `PASS` with the counts of `warning`, `info`, and `review`. Sort rows error → warning → info → review.
8. **Report** as one table `Severity | Rule | File:line | Finding | Fix`, then the verdict line, then the rule-source citations.

## Result

One violations table with a `file:line` and a concrete fix on every row, a `PASS` or `FAIL` line, and the counts per severity. Measure: rerunning the skill after the listed fixes yields zero `error` rows; the row count equals the number of distinct findings, with none omitted for brevity.

## Guardrails

- Every row has a `file:line`. A finding without a location is not reported.
- Severity is calibrated: an `error` blocks; do not inflate warnings to errors or bury an error as info.
- Do not rewrite the files. Report and propose; edits belong to `sf_apex_build` or `sf_declarative_build` after the user asks.
- Do not invent rules beyond the sixteen plus the listed extras. If a knowledge-bank hit adds a project rule, cite it on the row.
- If a file could not be read, list it under `Not checked` rather than passing it silently.
