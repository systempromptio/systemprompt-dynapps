# BA Impact Analysis

Answers "what changes if we do X" with evidence: an impacted-items table across epics, stories, the requirement traceability matrix, and the descoped list, a scope verdict (in scope, out of scope, or Change Request), a risk assessment, one recommendation, and at most five open questions. It gives the Product Owner a decision to make rather than a discussion to have.

## Use when

- A Product Owner or stakeholder proposes a new feature, a change to an approved one, or an architectural shift.
- The user asks which stories or epics a change touches, or whether something is a Change Request.

## Inputs

| Input | Required | Notes |
|---|---|---|
| The proposed change, in the proposer's words | yes | Quote it verbatim at the top of the output. |
| Project key | yes | Never assume it. |
| Who proposed it | no | Checked against the stakeholder list for decision authority. |
| Affected module(s) if known | no | Narrows the searches. |

## Workflow

1. **Ground in the knowledge bank.** Call `search_knowledge` with `{"query": "<change> scope FSD", "top_k": 5, "source_types": ["project_docs"]}`, `{"query": "<change> architecture integration", "top_k": 5, "source_types": ["project_docs"]}`, `{"query": "<change> <module>", "top_k": 10, "source_types": ["project_tickets"]}`, and `{"query": "<client> stakeholders decision authority change request process", "top_k": 3, "source_types": ["client_accounts"]}`. Cite every hit that lands in the table. If unavailable, say so and ask the user for the scope document and the list of stories in the module.
2. **Scope verdict.** Compare the change against the FSD and scope hits: in scope (covered by an existing FSD ID), out of scope (contradicts the scope), or Change Request (absent from the scope). Check the descoped list with `{"query": "descoped items <change>", "top_k": 5, "source_types": ["project_docs"]}`; a descoped match makes the verdict Change Request with the conflict named.
3. **Enumerate impacted items.** For each epic, story, RTM row, and descoped item touched, record: ID (Jira key or FSD ID, never invented), title, item type, impact type (`rewrite AC`, `new dependency`, `descope`, `re-estimate`, `blocked`, `no change but verify`), and the evidence citation. If a Jira tool is available in this session, confirm each key exists and its current status; if a tracker tool exists, read the RTM and Risk Register for existing entries.
4. **Assess risk.** For each new risk: description, likelihood (High / Medium / Low), impact (High / Medium / Low), mitigation, owner if known. Log risks to the Risk Register when a tracker tool exists; otherwise include them in a `Risks to log` block so they are not lost in chat.
5. **Recommend one path**: proceed as-is, raise a Change Request (assign the next `CR-nnn` only after the user agrees it is a CR), or descope the conflicting item. State the sprint and estimate consequence in one sentence each.
6. **Open questions.** List at most five, each addressed to a named role, each blocking a specific row of the table.
7. **Present** the verdict, the table, the risks, the recommendation, and the questions. Ask the user which path to take before any tracker, Jira, or Confluence write.

## Result

An output with the change quoted, a one-line scope verdict, an impacted-items table `ID | Title | Type | Impact | Evidence`, a risk table, a single recommendation, and up to five questions. Measure: every ID in the table resolves to a real ticket or FSD entry (or is marked `unverified` with the reason), no row lacks an evidence citation, and the questions count is five or fewer.

## Guardrails

- Never invent Jira keys, FSD IDs, or stakeholder names; unverifiable rows are labelled `unverified`.
- Never write to Jira, Confluence, or a tracker without explicit approval in the current chat turn; never overwrite a live tracker.
- A change that is clearly a Change Request stops here with the recommendation; the CR itself is raised only after the user's decision.
- Risks identified are recorded, not merely mentioned.
- Five open questions maximum; group related ones.
- No Gherkin in any rewritten AC you propose.
- Log every clarification the user provides.
