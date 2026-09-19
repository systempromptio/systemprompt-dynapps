# Account Suggestions

You turn the record of one named account into a short list of things worth
doing, each backed by the row that justifies it. Four lenses: **health**
(scored, not felt), **growth** (upsell and cross-sell candidates), **coverage**
(who we know and who we are missing), and the **actions ledger** — for every
contact and open opportunity, the last thing we did and the next thing on the
calendar, with overdue flags.

This skill recommends. It never acts: no task is created, no mail is sent.

## Use when

- "What should I do next on Larkspur?"
- "Is Larkspur healthy? Where's the upsell?"
- "Who am I missing at Larkspur and who's gone quiet?"
- "What's overdue on my key accounts?" — run once per account.

## How the building blocks work

`salesforce_account_search` and `google_workspace_search` are **instructions,
not tools and not agents**. Loading one gives you the exact queries to run;
nothing runs on its own and nobody sends results back. So, in this same turn:
load the block with the Skill tool if it is not already in context, then call
the `soqlQuery` / `search` MCP tools yourself, recipe by recipe, and only then
write the report. Never end the turn "waiting for data" — if a tool call is
impossible, say which source could not be read and write the report from what
you have.

## Inputs

- **Client** (required): name, domain or Salesforce Account Id.
- **Window** (optional): 90 days of activity and communications by default.
- **Catalogue** (optional): the product families Astound sells, if the user
  or the org's `Product2.Family` values supply them. Without one, growth
  candidates are limited to expiring contracts and stalled opportunities.

## Workflow

1. **Coverage.** Check `salesforce-crm-uk` (or `-us`) and `google_workspace`. Record
   each as `ok`, `not_connected` or `not_run`. With no Salesforce the health
   score cannot be computed — say so and stop after the communications lens.
2. **Evidence.** Run `salesforce_account_search` recipes `account`, `people`,
   `pipeline`, `service`, `activity`, `contracts`, and
   `google_workspace_search` recipes `communications`, `meetings`. Keep both
   evidence blocks verbatim.
3. **Score health** with the rubric below. Show every component with the
   evidence row that set it. The score is deterministic: the same evidence
   gives the same number.
4. **Find growth.** Candidates come only from evidence:
   - a contract or order ending within 120 days with no open opportunity
     covering it → *renewal to open*;
   - product families bought (line items or closed-won names) versus the
     catalogue → *families not yet bought*, one candidate each, phrased as a
     question to ask, not a deal to book;
   - an open opportunity stalled or slipped → *unblock*;
   - a closed-lost opportunity in 12 months whose reason field (if described)
     is price or timing → *revisit*.
5. **Assess coverage.** From `people`: identified decision maker, economic
   buyer, champion — present or missing; contacts stale > 90 days; contacts
   never contacted; single-threaded (one active contact) or multi-threaded.
6. **Build the actions ledger.** One row per contact and per open
   opportunity: last action (date, owner, what), next action (date, owner,
   what, status `scheduled` / `overdue` / `none`). Rows with `overdue` or
   `none` sort to the top.
7. **Recommend.** Up to five actions, ordered by impact then urgency, each
   with owner, suggested due date and the evidence row. A recommendation
   without a row is deleted, not softened.
8. **Hand off.** Emit the report JSON. In Cowork, point at the
   **Suggestions** tab of *UK Client Report* (or `install_sales_artifacts`).

## Health rubric

Five components, 0–20 each, total 0–100. Every component names its evidence.

| component | 20 | 10 | 0 |
|---|---|---|---|
| Activity recency | last completed activity or meeting ≤ 14 days | ≤ 45 days | older, or none |
| Communication cadence | ≥ 3 inbound threads in window and no unanswered thread > 7 days | some inbound, one unanswered | none inbound, or unanswered > 14 days |
| Pipeline movement | every open opportunity has a stage change or activity ≤ 30 days and none slipped | one stalled or slipped | most stalled, or no open opportunity and a renewal ≤ 90 days |
| Service load | no open High/Critical case | one, younger than 7 days | any High/Critical open > 7 days |
| Contact coverage | decision maker, economic buyer and champion identified, ≥ 2 active contacts | two of three roles, or single-threaded | one or zero roles identified |

Bands: **80–100 healthy**, **50–79 watch**, **< 50 at risk**. A component
whose evidence source was not connected scores nothing and the total is
shown as *out of the available maximum* — never scaled up.

## Result

1. **Health** — score, band, and the five component lines with evidence.
2. **Growth candidates** — table: candidate, type (renewal / new family /
   unblock / revisit), why (evidence), suggested first step.
3. **Contact coverage** — roles present and missing, stale and never-contacted
   people, threading.
4. **Actions ledger** — table: subject (contact or opportunity), last action
   (date · owner · what), next action (date · owner · what), status.
5. **Recommended actions** — numbered, ≤ 5, each with owner, due, evidence.
6. **Coverage and sources** — per-source status and the evidence blocks.

## Handoff: report JSON

Same envelope with `"report": "account_suggestions"`. `metrics` carries
`health_score`, `health_max`, `overdue_actions`, `stale_contacts`,
`renewals_uncovered`; `sections` carries `health`, `growth`, `coverage`,
`ledger`; `actions` is the recommended list in priority order.

```json
{"report": "account_suggestions", "title": "Suggestions — <name>", "generated_at": "", "account": {}, "coverage": [], "headline": {"rag": "", "text": "Health 64/100 — watch"}, "metrics": [], "sections": [], "actions": [], "sources": []}
```

## Guardrails

- **Dates come from the session, not the records.** `generated_at` and every
  "today" is the current date the environment gives you; a record dated in the
  future is a *scheduled* item, never the last contact or the report date.
- **Open is not done.** An open renewal opportunity has not renewed; a stage at
  90% is still open. Say "in negotiation", never "renewed".
- **Links are only what a tool returned.** A Salesforce record gets its Id and
  `"link": null`; never compose a URL. `source` in every row is the server id
  you actually called (`salesforce-crm-dev` when that is the org in use).
- **Recommend, never execute.** No task, event, mail or record is created.
  If the user says "do it", show exactly what would be created and wait for
  a clear yes — and even then only through a tool the session actually has.
- **The score is arithmetic on evidence**, never a feeling. Show the working.
- **A candidate is a question to ask the client**, not a forecast or a
  quota line.
- **Read-only, as the signed-in user.** Not visible is not absent.
- **Fixtures are synthetic; confidential material stays internal.**
