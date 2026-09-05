# SF Story Intake

Turns a pasted Jira story into a governed, schema-checked implementation brief for a Salesforce Core project. It produces the four story-folder documents that every downstream persona, deploy, and QA pass reads from, with every object and field named exactly as the project schema names it and every assumption written down before a line of code exists.

## Use when

- The user pastes a Jira story, user story, bug, or feature requirement for a Salesforce Core (Sales / Service Cloud) project.
- A story folder `stories/<STORY-ID>/` does not yet exist, or exists but is stale against the ticket.
- Someone asks "which persona should build this" or "what does this story touch".

## Inputs

| Input | Required | Notes |
|---|---|---|
| Story text or Jira key | yes | Verbatim text preferred. With only a key and a Jira tool in session, fetch it. |
| Story labels | yes | Needed for the no-AI gate. If labels are not visible, ask before continuing. |
| Repo root | yes | Must contain `sfdx-project.json`; the story folder is written beside `force-app/`. |
| Target sandbox alias | no | Goes into `deployment.md`; leave a `<sandbox-alias>` placeholder if unknown. |

## Workflow

1. **Ground in the knowledge bank.** Call the `knowledge-bank` MCP tool `search_knowledge` three times: `{"query": "<project> Salesforce Core project schema objects fields", "top_k": 5, "source_types": ["project_docs"]}`, `{"query": "<story title or key words>", "top_k": 5, "source_types": ["project_tickets"]}`, and `{"query": "Salesforce Core always-active rules branch deploy practice", "top_k": 3, "source_types": ["project_docs"]}`. Cite every hit you rely on by its `uri` in `design-note.md`. If the tool is unavailable or returns nothing, write `Knowledge bank: unavailable` in the design note and fall back to `docs/PROJECT_SCHEMA.md` in the repo.
2. **No-AI gate.** Inspect the labels. If any of `no-ai`, `no_ai`, `noai`, `ai-exempt`, `human-only` is present, reply exactly `REFUSE: story <ID> is labelled <label>; human-only work.` and stop. Do not summarise, parse, or evaluate the story.
3. **Parse.** Extract into a table: What (feature / automation / UI / integration / fix), Object(s), Field(s) with API names, each Acceptance Criterion as `AC-n`, Constraints (performance, security, licensing, blackout windows).
4. **Schema check.** Compare every object and field against the schema hits from step 1 (or `docs/PROJECT_SCHEMA.md`). Mark each as `exists` or `to be added`. Never invent an API name. An unmatched name with no "new field" wording in the story is an open question, not a guess.
5. **Route.** Apply the decision tree in order and record the answer: Apex / trigger / batch / callout / >1000 records → `Apex_Architect`; else record logic doable by Flow, validation rule, or CMT → `Declarative_Lead`; else UI / LWC / page → `UI_UX_Specialist`; else `Feature_Lead` (design note only). Always append `QA_Engineer`. A story may need several personas in sequence; list them in order.
6. **State assumptions.** Before writing files, list: persona(s) assumed, objects and fields impacted with their exists / to-be-added status, whether Flow / validation rule / CMT / configuration was considered before Apex and why it was or was not sufficient, open questions. Ask at most five questions per turn and at most two rounds; if still unresolved, stop and report.
7. **Branch.** Instruct `git switch -c feature/<STORY-ID> develop` (or `bug/<TICKET-ID>`; `main` where no `develop` exists). Do not run it without confirmation.
8. **Write the four files** under `stories/<STORY-ID>/`:
   - `requirement.md` — the story text verbatim, labels, source link, capture date.
   - `design-note.md` — schema impact table, impacted components and metadata, persona(s) and rationale, declarative-first evaluation, deployment ordering constraints, rollback strategy, knowledge-bank citations, `Last updated: <date>`.
   - `deployment.md` — deploy order, `sf project deploy start --manifest manifest/<STORY-ID>-package.xml --target-org <sandbox-alias> --test-level NoTestRun --wait 15` for sandbox with a stated reason, post-deploy steps (page-layout assignment, permission set assignment, flow activation).
   - `qa-test-request.md` — one `MT-nn` manual case per AC with steps and expected result, the `sf apex run test --class-names <TestClass> --code-coverage --result-format human --target-org <sandbox-alias> --wait 10` command, and a **Video Capture Requirement** section requiring final manual UI testing recorded on video with test case IDs identifiable.
9. **Hand off.** Name the next skill: `sf_apex_build` for `Apex_Architect`, `sf_declarative_build` for `Declarative_Lead`. Present the file list and the assumptions block as the final answer.

## Result

Four files exist at `stories/<STORY-ID>/{requirement,design-note,deployment,qa-test-request}.md`. The design note contains a schema-impact table where every row is `exists` or `to be added` and no row is a guess, a named persona sequence, and at least one knowledge-bank citation or the explicit `unavailable` marker. The final message lists the files, the persona route, and the open questions. Measure: `ls stories/<STORY-ID>/ | wc -l` returns 4 and `grep -c "to be added\|exists" stories/<STORY-ID>/design-note.md` equals the number of fields parsed.

## Guardrails

- A no-AI label is a hard halt. Refuse and stop; never read the story "just to summarise it".
- Never invent object, field, profile, or record type API names. Unknown means ask.
- Never write code in this skill. Intake ends at the four documents and the hand-off.
- Never commit, push, deploy, or create a PR. Every write beyond the story folder needs an explicit human YES in the current turn.
- Two clarification rounds maximum, then halt and report what is blocked.
- Apex is the last resort. The design note must show the declarative evaluation even when Apex wins.
- If the story adds custom objects or fields, the design note must state that `docs/PROJECT_SCHEMA.md` and the page layouts change in the same PR.
