# BA Bug Report

Produces one Jira-ready bug draft that a developer can reproduce on the first attempt: severity with justification, the exact environment, numbered steps from the application entry point, expected versus actual with the real on-screen text, and reproducibility. It checks for duplicates first so the backlog gets one ticket per defect.

## Use when

- QA or a user reports a defect, regression, or unexpected behaviour.
- The user asks whether a bug has already been logged.

## Inputs

| Input | Required | Notes |
|---|---|---|
| Observed behaviour and the exact UI text or screenshot | yes | Vague actuals are rejected; ask for the message verbatim. |
| Environment | yes | Application URL or name, build or commit, browser and device, user type, locale. Ask for what is missing. |
| Source story key | no | The story whose AC the behaviour violates. Never invent it. |
| Severity | no | Confirm with the user: Critical, Major, Minor, Trivial. |

## Workflow

1. **Ground in the knowledge bank.** Call `search_knowledge` with `{"query": "<module> <observed behaviour>", "top_k": 5, "source_types": ["project_tickets"]}` and `{"query": "<module> acceptance criteria <feature>", "top_k": 3, "source_types": ["project_docs"]}`. Cite the source story or FSD page that defines the expected result. If unavailable, say so and ask the user for the story key or the expected behaviour.
2. **Duplicate check.** Compare the ticket hits by module and observed behaviour; if a Jira tool is available in this session, run a JQL search restricted to issue type Bug. A likely duplicate is reported with its key and the draft is stopped unless the user confirms the defect is different.
3. **Confirm severity** with the user if not stated: Critical (blocks a core flow or data loss), Major (a feature fails with no workaround), Minor (workaround exists), Trivial (cosmetic). Write the one-line justification.
4. **Draft the bug** in this shape, never the user-story structure:
   - **Title**: `[BUG] | [Module] | [Observed behaviour] vs [Expected behaviour]`
   - **Summary**: one paragraph.
   - **Environment**: Application, Build, Browser/Device, User type, Locale, each on its own line.
   - **Steps to Reproduce**: numbered, starting at the application entry point, one action per step, device stated.
   - **Expected Result**: per the AC scenario, referencing the source story key when known.
   - **Actual Result**: what happens, including the exact error message or UI state.
   - **Reproducibility**: `Always` or `N of N attempts`.
   - **Severity**: the level and its justification.
   - **Labels**: proposed `["bug", "<module>"]`, passed as a parameter, not written in the body.
5. **Self-check**: no Given/When/Then; no vague actuals; every environment line filled or marked `Unknown - to confirm`; steps reproduce from a cold start.
6. **Render** in Markdown for chat and in Jira wiki markup for push (`*bold*` headings, `#` numbering, blank line between sections).
7. **Approval gate.** Present the draft. Push with the Jira tool only after a yes in the current turn, issue type Bug, labels in the `labels` parameter. Link to the source story if given; update the traceability matrix only when the bug is directly linked to a source story and a tracker tool exists.

## Result

One bug draft with all nine fields, in both renderings, plus the duplicate-check outcome (`none found` or the candidate keys). Measure: a developer can execute the numbered steps without asking a question; the Actual Result quotes on-screen text; the duplicate check names the JQL or the knowledge-bank query it ran.

## Guardrails

- Never push without explicit approval in the current chat turn.
- Never invent a source story key, build number, or error message; unknown is marked and asked.
- Never use Gherkin.
- Never use the user-story template for a bug.
- Severity is justified or asked, never assumed.
- Labels go in the labels parameter, never in the description.
- Log any clarification the user gives as part of the draft's record.
