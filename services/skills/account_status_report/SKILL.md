# Account Status Report

You answer one question about one named account: **where does it stand
today?** Money first — what is open, what closed, what renews, what is
overdue — then the last thirty days of contact, what we have promised, and
when we next speak. It is narrower and faster than `client_360_report` and is
the page an account owner checks on a Monday.

## Use when

- "Status on Larkspur?"
- "What's the financial position with Larkspur and when did we last talk?"
- "Anything outstanding on the Larkspur account before I call them?"
- A weekly one-to-one needs a per-account line.

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
- **Window** (optional): 30 days of communications, 12 months of closed
  business, 90 days of cases. State it.

## Workflow

1. **Coverage.** Check `salesforce-crm-uk` (or `-us`) and `google_workspace`; Atlassian
   is not consulted here. Record each as `ok`, `not_connected` or `not_run`.
2. **Money.** Run `salesforce_account_search` recipes `account`, `pipeline`,
   `contracts`, `service`. From the evidence derive, per currency:
   - open pipeline by stage, with the soonest close date and any slipped or
     stalled flags;
   - closed won and closed lost in the last 12 months, count and value;
   - contracts and orders with end dates, renewals inside 90 days;
   - open cases by priority, and any High or Critical case older than seven
     days as a service risk.
3. **Contact.** Run `salesforce_account_search` recipe `activity` and
   `google_workspace_search` recipes `communications` and `meetings` for the
   window. Derive: last inbound (date, who), last outbound (date, who),
   threads where the last word was theirs and unanswered, last meeting, next
   meeting or scheduled activity, overdue tasks.
4. **Commitments.** From task subjects, meeting notes and mail, list what we
   said we would do, who owns it and by when, with the source row. A
   commitment with no evidence of completion after its date is **open and
   late**.
5. **Set the RAG.** Green: no overdue commitment, no service risk, contact
   inside 14 days, nothing slipped. Amber: any one of those. Red: two or
   more, or an unanswered client thread older than seven days, or a renewal
   inside 60 days with no open opportunity. State which condition set it.
6. **Hand off.** Emit the report JSON (below). In Cowork, point the user at
   the **Status** tab of *UK Client Report*; if it is not installed, at
   `install_sales_artifacts`.

## Result

1. **Status headline** — RAG, one sentence, the condition that set it.
2. **Financial position** — three short tables: open pipeline (stage, name,
   amount + currency, close date, days in stage, flag); closed 12 months
   (won/lost, count, value per currency); contracts and renewals (number,
   end date, days to renewal, open opportunity covering it: yes/no).
   Service risk line beneath: open cases by priority, oldest High/Critical.
3. **Recent communications** — last inbound, last outbound, unanswered
   threads (subject, from, days waiting), last meeting, documents touched in
   the window.
4. **Open commitments** — table: commitment, owner, due, status
   (open/late/done), evidence.
5. **Next scheduled touch** — the soonest future event or task with owner,
   or *none scheduled* as a finding.
6. **Coverage and sources** — per-source status and the evidence blocks,
   verbatim.

Measure of done: every amount has a currency and a date, every "late" has a
due date and a source, and the RAG can be defended from the page alone.

## Handoff: report JSON

Same envelope as `client_360_report` with `"report": "account_status"`.
`metrics` carries at least `open_pipeline`, `closed_won_12m`,
`closed_lost_12m`, `renewals_90d`, `open_cases`, `overdue_tasks`,
`days_since_last_contact` (each with `currency` where money, `as_of` always).
`sections` carries `pipeline`, `closed`, `renewals`, `communications`,
`commitments`; `actions` carries the late commitments and the next touch.

```json
{"report": "account_status", "title": "Account status — <name>", "generated_at": "", "account": {}, "coverage": [], "headline": {"rag": "", "text": ""}, "metrics": [], "sections": [], "actions": [], "sources": []}
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
- **No forecasting.** Pipeline totals are pipeline; they are never weighted,
  projected or called revenue. Closed won is the only revenue-shaped number
  and it is labelled as CRM-recorded, not invoiced.
- **Currencies are never mixed.** Totals are per ISO code.
- **Read-only**, as the signed-in user, on both systems.
- **Unanswered means unanswered by us.** Only a thread whose latest message is
  the client's counts; do not infer from a gap in dates.
- **Absence is reported**: no meeting scheduled, no open opportunity against
  a renewal, no contact in 30 days — each is a finding line, never silence.
- **Fixtures are synthetic**; confidential material stays internal.
