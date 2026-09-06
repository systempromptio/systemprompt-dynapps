# BA Epic Breakdown

Drafts one epic in the organisational protocol and decomposes it into a dependency-ordered list of front-end and back-end stories. It produces the epic body with persona narrative, measurable KPIs, in-scope and descoped items, and a story list a sprint can be planned from, all presented in chat for approval before anything touches Jira.

## Use when

- The user asks to create an epic, define a module, or start a new feature group from a scope document.
- The user asks to break an approved epic into stories.

## Inputs

| Input | Required | Notes |
|---|---|---|
| Module or feature-group name | yes | |
| Scope or FSD reference | yes | Confluence page, FSD IDs, or pasted text. |
| Project key | yes | Never assume it; confirm with a Jira tool if present. |
| Industry context | no | Used for the research step and KPI baselines. |

## Workflow

1. **Ground in the knowledge bank.** Call `search_knowledge` with `{"query": "<module> scope FSD in-scope features", "top_k": 5, "source_types": ["project_docs"]}`, `{"query": "<module> architecture integrations", "top_k": 5, "source_types": ["project_docs"]}`, `{"query": "<module> epic", "top_k": 5, "source_types": ["project_tickets"]}`, and `{"query": "<client> business model KPIs industry", "top_k": 3, "source_types": ["client_accounts"]}`. Cite the scope page and any existing epic by `uri`. If unavailable, say so and ask for the scope text.
2. **Pre-flight: descoped check.** Search `{"query": "descoped items excluded scope <module>", "top_k": 5, "source_types": ["project_docs"]}`. A descoped module is refused with `This feature is explicitly descoped in the project rules.` Descoped sub-items are listed under Descoped Items, not silently dropped.
3. **Pre-flight: duplicate epics.** Use the ticket hits; if a Jira tool is available in this session, run a JQL search for epics in the same module. An existing epic means propose an update, not a duplicate.
4. **Research.** Where the scope leaves a practice undefined, use web search for the industry norm and say which points came from research versus the scope document. Never let research add scope.
5. **Draft the epic** in exactly this shape:
   - **Description** — `1. As a [persona], I want to [action] so that [value].`
   - **Success Metrics (KPIs)** — at least two measurable, time-bound goals with a baseline where the knowledge bank supplies one.
   - **In-Scope Features** — numbered, each traceable to an FSD ID or the Temp FSD ID you propose when the scope has no IDs (format `<ABBR>-001`).
   - **Descoped Items** — only when relevant, cross-referenced from the descoped list.
6. **Decompose into stories.** Propose five to seven stories: separate FE and BE where parallel work is possible; BE and infrastructure stories are blockers and go first; title each `[Layer] | [Module] | [Action Summary]`; give one line of scope per story and its dependency (`depends on: <title or Jira key>`); no story bleeds into another's boundary.
7. **Clarify.** Ask at most five grouped questions for anything the scope does not settle (persona, KPI targets, third-party ownership). Log answers as clarifications.
8. **Approval gate.** Present the epic and the story list in chat. Ask for label values. Push the epic with the Jira tool only after a yes in the current turn, labels in the `labels` parameter. Stories are then drafted one by one with `ba_story_draft`, each needing the returned epic key as parent.

## Result

An epic body with the three mandatory sections (four with Descoped Items), every in-scope feature carrying an FSD or Temp FSD ID, at least two KPIs with numbers and timeframes, and a dependency-ordered story table `# | Title | Layer | Scope | Depends on`. Measure: each story title parses as `[FE|BE] | <Module> | <summary>`, no story lacks a dependency entry (`None` counts), and no in-scope feature lacks an ID.

## Guardrails

- Never push before approval in the current chat turn; never assume the project key.
- Never invent FSD IDs from thin air: propose Temp FSD IDs explicitly labelled as proposals.
- Never draft an epic, or a story within it, for a descoped feature.
- Never write labels in the description text.
- Never use Gherkin anywhere, including the story scope lines.
- Research informs KPIs and wording; it never adds features the scope does not contain. A feature outside scope is a Change Request question, asked and not assumed.
- Log every clarification received.
