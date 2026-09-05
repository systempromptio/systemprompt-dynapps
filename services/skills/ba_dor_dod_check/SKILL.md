# BA DoR/DoD Check

Validates one or many stories against the project's Definition of Ready before sprint planning, or Definition of Done before closure, and returns a per-story table with a pass or fail verdict and exactly which items are missing. The table is the artefact the planning meeting reads; nothing is marked Ready or Done on judgement alone.

## Use when

- Before sprint planning, to decide which stories are truly Ready.
- Before transitioning a story to Done.
- The user asks to batch-validate a sprint, an epic, or a list of keys.

## Inputs

| Input | Required | Notes |
|---|---|---|
| Story keys, a JQL, or pasted story text | yes | With a Jira tool in session, keys and JQL are fetched; otherwise ask for the story bodies. |
| Mode | yes | `DoR` or `DoD`. |
| Project DoR / DoD definitions | yes | From the knowledge bank or the user; if neither has them, stop and ask. |

## Workflow

1. **Ground in the knowledge bank.** Call `search_knowledge` with `{"query": "definition of ready checklist", "top_k": 3, "source_types": ["project_docs"]}` or `{"query": "definition of done checklist", "top_k": 3, "source_types": ["project_docs"]}` per mode, plus `{"query": "BA writing standards story structure", "top_k": 3, "source_types": ["project_docs"]}` and `{"query": "<epic or sprint> stories", "top_k": 10, "source_types": ["project_tickets"]}`. Cite the definition page you are validating against. If the definitions are not found anywhere, ask the user to supply them and stop; do not validate against an assumed list.
2. **Fetch each story.** If a Jira tool is available in this session, read every key and its fields (summary, description, parent, labels, status, linked issues, attachments); otherwise use the pasted bodies. Never validate a story you have not read in full.
3. **Apply the project definition item by item.** In addition, in DoR mode always check the organisational baseline: title matches `[Layer] | [Module] | [Action]`; description carries `As a … I want … so that …`; AC present, numbered, scenario names device and navigation path, no Gherkin, no vague terms (`all`, `relevant`, `appropriate`, `etc.`); Requirements end with the ADA link; Dependencies section present; Technical Notes blank; parent Epic key assigned; reviewed in chat by the BA. In DoD mode check: every AC scenario has recorded test evidence, linked bugs are closed or accepted, design sign-off recorded where UI changed, the traceability matrix carries the key, no open clarification remains.
4. **Score each story**: `n/m` items met, the missing items named exactly as they appear in the definition, and a verdict of `Ready` / `Not Ready` or `Done` / `Not Done`. A story with absent or vague AC is `Not Ready` regardless of the count.
5. **Present the table** `Jira ID | Title | Items Met | Items Missing | Verdict`, one row per story, then a one-line summary count.
6. **Record.** Offer to add the row as a Jira comment on each story for traceability; add comments or transition a story only after a yes in the current turn. A transition to Done additionally requires the DoD verdict `Done`.

## Result

One table with one row per story, `n/m` met, every missing item named, and a verdict, plus the definition page cited. Measure: every story requested has a row; every `Not Ready` or `Not Done` row lists at least one missing item; no story with missing or Gherkin AC shows `Ready`.

## Guardrails

- Never transition a story or post a comment without explicit approval in the current chat turn.
- Never mark Ready when AC are absent, contain Gherkin, or use vague terms.
- Never validate against an invented definition; missing DoR/DoD means ask and stop.
- Never invent a Jira key; a key that cannot be fetched is reported as `not found`.
- Labels, if suggested, go in the labels parameter, never in the description.
- Log any clarification the user gives about the definitions.
