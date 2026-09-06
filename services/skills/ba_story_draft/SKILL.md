# BA Story Draft

Drafts one front-end or back-end user story in the organisational writing standard, exhaustive enough that what is written is exactly what gets built. It produces two renderings of the same story: Markdown for review in chat, and Jira wiki markup ready for the push tool, with the Technical Notes section left blank for the developers.

## Use when

- The user asks to draft, write, split, or refine a user story, an FE story, a BE story, or an FE+BE pair.
- The user asks whether an existing draft meets the writing standard.

## Inputs

| Input | Required | Notes |
|---|---|---|
| Feature description or FSD reference | yes | Ask for the FSD ID if the scope document has one. |
| Layer | yes | FE or BE. Visual or interaction change → FE; logic or data change → BE; both → two stories, BE first. |
| Module / Epic shorthand | yes | Used in the title. Confirm the parent Epic key before any push. |
| Project key | yes | Never assume it. |
| Design links, ADA link, Field Validations page link | no | Placeholders are flagged as open questions, never invented. |

## Workflow

1. **Ground in the knowledge bank.** Call `search_knowledge` with `{"query": "BA writing standards user story structure acceptance criteria", "top_k": 3, "source_types": ["project_docs"]}`, `{"query": "<feature> FSD scope architecture", "top_k": 5, "source_types": ["project_docs"]}`, `{"query": "<feature> story", "top_k": 5, "source_types": ["project_tickets"]}`, and `{"query": "<client> project context stakeholders", "top_k": 3, "source_types": ["client_accounts"]}`. Cite the FSD page and any similar stories by `uri`. If unavailable, say so and ask the user for the FSD text and a similar story.
2. **Pre-flight: descoped check.** Search `{"query": "descoped items excluded scope <feature>", "top_k": 5, "source_types": ["project_docs"]}`. If the feature or a dependency it needs is descoped, reply `This feature is explicitly descoped in the project rules.` and stop.
3. **Pre-flight: similar stories.** Use the ticket hits from step 1; if a Jira tool is available in this session, also run a JQL search on the feature keywords. Reuse their wording for shared components so the backlog stays consistent, and reference them by Jira key.
4. **Clarify.** Ask at most five grouped questions for anything the FSD, the architecture notes, and prior clarifications do not answer: persona, device parity, error copy, third-party behaviour, locale. Log every answer received as a clarification (Questions tracker if a tracker tool exists, otherwise a `Clarifications` block in the reply). Do not proceed on a guess.
5. **Draft the story** in this exact order:
   - **Title**: `[Layer] | [Module/Epic] | [Action Summary]`, summary 15 to 20 words maximum.
   - **Description**: `As a [User Role], I want [action] so that [business value].` Internal persona for analytics or infra stories.
   - **Design Links**: the links supplied; `To be added` if none.
   - **Requirements**: multi-level numbered list grouped as UI/UX Rendering, Interactions, API Integration, Fail-safes. Desktop and mobile separated when behaviour differs. Fail-safes name the fallback and the exact error copy; unknown API failures are logged for later review. Form fields point at the Field Validations page with the row reference; no inline validation rules. The last numbered point is `ADA requirements are listed here: [Link]`.
   - **Acceptance Criteria**: one or more `Scenario n - Assuming a customer visits the website via a [desktop/mobile] device and navigates to [Page] to [Action].` in bold, unnumbered; under each, numbered steps starting `Customer does [X]`, then they/them. Every error state, edit flow, and field has its own explicit steps. The last step of every scenario is `ADA compliance to be maintained.` Elements that touch undeveloped features keep full AC plus the note `NOTE: The redirection/functionality for [Feature] is not yet developed. Integration is out of scope for this story and will be added later as part of a separate story.` and are logged as a pending task.
   - **Dependencies**: cross-story (by Jira key) or third-party; `None` if absent.
   - **Notes**: only for BE content-slot stories, asking the tech lead to add slot IDs once developed.
   - **Technical Notes**: heading present, body empty.
6. **Self-check** against the standard: no Given/When/Then anywhere; no "all", "relevant", "appropriate", "etc." in AC; no HTTP codes, endpoint names, payload keys, HTML attributes, or CSS classes; device and navigation path in every scenario; ADA link last in Requirements; Technical Notes blank.
7. **Render twice.** Markdown for chat (`1.`, `a.`, `**bold**`). Jira wiki markup for push (`*bold*` headings, `#`/`##`/`###` numbering, `*`/`**` bullets, an empty line between every section and before and after each scenario line, no `h2.` headings).
8. **Approval gate.** Present both renderings and the clarifications log. Ask for the parent Epic key and the label values. Push with the Jira tool only after the user says yes in the current turn, passing labels in the `labels` parameter; then record the returned key in the traceability matrix if a tracker tool exists.

## Result

One story with all eight sections in the mandated order, in both renderings, plus a clarifications log and the list of open questions. Measure: `grep -ci "given\|when\|then"` over the AC returns 0 for Gherkin keywords used as step openers, every scenario line names a device and a page, the Requirements list ends with the ADA link, and the Technical Notes body is empty.

## Guardrails

- Never use Gherkin. Given/When/Then in AC is a failed draft.
- Never push to Jira without explicit approval in the current chat turn, and never without a confirmed parent Epic key.
- Never invent Jira keys, FSD IDs, stakeholder names, vendor names, endpoints, or error copy that the FSD provides; unknown means ask.
- Never write in Technical Notes.
- Never pass labels inside the description; use the labels parameter.
- Never draft a descoped feature or one whose dependency is descoped.
- Log every clarification received; a decision that lives only in chat is lost.
- BE stories describe functional outcomes, never response shapes or status codes.
