# Client 360 Report

You produce the whole-relationship view of one named client: who they are to
us, who we know there, what money is in play, how delivery is going, what we
have said to each other lately, and what that adds up to. It is grounded in
three sources — Salesforce, Google Workspace and Atlassian — each of which may
be missing, and it says which ones it saw.

## Use when

- "Give me the full picture on Larkspur."
- "I'm taking over the Larkspur account — what do I need to know?"
- "Build the 360 for the QBR."
- A leader asks for "everything we have" on a key account.

For a meeting in the next few hours use `uk_sales_client_prep`; for money and
recent contact only, `account_status_report`; for what to do next,
`account_suggestions`.

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

- **Client** (required): name, domain or Salesforce Account Id. Ask when more
  than one Account matches.
- **Window** (optional): 90 days for activity and communications, 12 months
  for closed business. Say the window in the output.
- **Audience** (optional): internal by default. A client-facing version drops
  internal notes, health scores and commercial margin — ask before producing
  one.

## Workflow

1. **Coverage first.** Check which of `salesforce-crm-uk` (or `-us`),
   `google_workspace` and `atlassian` this session can reach. Record each as
   `ok`, `not_connected` or `not_run`. A missing source is reported in the
   Coverage line and its sections are marked *not available to this caller* —
   never dropped, never filled from memory.
2. **CRM.** Run `salesforce_account_search` recipes `account`, `people`,
   `pipeline`, `service`, `activity`, `contracts` — in that order, resolving
   the Account once. Keep its evidence block verbatim.
3. **Workspace.** Run `google_workspace_search` recipes `documents`,
   `communications`, `meetings` with the client name, domain and any CRM
   alias. Keep its evidence block verbatim.
4. **Delivery.** If Atlassian is connected: resolve the site with
   `getAccessibleAtlassianResources`, find the client's Jira project(s) and
   Confluence space by a *verified* association (project key or space name
   recorded on the Account, or an explicit user statement — never a name
   match alone), then read open issues newest first, issues closed in the
   window, and the newest status report or decision log. Record the key and
   updated date of everything used.
5. **Reconcile.** Where sources disagree — CRM says the renewal is in
   negotiation, mail says the client paused it — show both with dates. Do
   not merge them into one story.
6. **Assess.** Write the relationship headline and the risks and
   opportunities *from the evidence rows only*. Each risk or opportunity
   points at the row that justifies it.
7. **Hand off to the artifact.** Emit the report JSON (below). In Cowork,
   tell the user to open **UK Client Report**, pick the account and choose the
   *360* tab for the live view; if the artifact is not in their library, point
   them at `install_sales_artifacts`.

## Result

A Markdown report in this order, every figure with currency and date, every
claim with its source row:

1. **Relationship headline** — two sentences: what this client is to Astound
   and where the relationship stands, with a RAG and the one fact that sets it.
2. **Account facts** — type, industry, owner, parent, since when, last CRM
   activity, tier or health fields if the org records them.
3. **People** — table: name, title, inferred role (decision maker, economic
   buyer, champion, influencer, user, unknown), last contact date, next action.
   Then the gaps: no identified economic buyer, champion stale > 90 days,
   single-threaded.
4. **Commercial** — open pipeline by stage (per currency), closed won and lost
   in 12 months, products bought, contracts and renewal dates, days in stage
   and slipped or stalled flags.
5. **Delivery** — open issues by priority, blocked items, what closed in the
   window, the latest status report line, each with key and date. Or *not
   available to this caller*.
6. **Communications** — last inbound, last outbound, unanswered threads,
   last meeting, next meeting, documents on file (newest SOW, proposal,
   account plan).
7. **Timeline** — the last 90 days merged across all sources, newest first,
   one line per event with its source.
8. **Risks and opportunities** — each one sentence plus the evidence row.
9. **Coverage and sources** — one line per source with status and query count,
   then the evidence blocks from the building-block skills, verbatim.

Measure of done: a colleague who has never heard of the client could run the
next meeting from this page, and every number on it can be clicked back to a
record.

## Handoff: report JSON

Emit this object after the Markdown (in a fenced `json` block). The UK Client
Report artifact renders exactly this shape; keys are fixed, arrays may be
empty, nothing else is added.

```json
{
  "report": "client_360",
  "title": "Client 360 — <account name>",
  "generated_at": "<RFC 3339>",
  "account": {"id": "", "name": "", "website": "", "owner": "", "org": "salesforce-crm-uk", "currency": "GBP"},
  "coverage": [{"source": "salesforce-crm-uk", "status": "ok|partial|not_connected|not_run", "detail": ""}],
  "headline": {"rag": "green|amber|red", "text": ""},
  "metrics": [{"key": "open_pipeline", "label": "Open pipeline", "value": 0, "currency": "GBP", "as_of": "<date>"}],
  "sections": [{"id": "people", "title": "People", "columns": ["Name", "Title", "Role", "Last contact", "Next action"], "rows": [{}], "notes": [""]}],
  "actions": [{"priority": 1, "action": "", "owner": "", "due": "<date|null>", "evidence": "<source row>"}],
  "sources": [{"source": "salesforce-crm-uk|google_workspace|atlassian", "title": "", "date": "<date>", "link": "", "id": ""}]
}
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
- **Read-only across all three systems.** Never log, edit, create, transition
  or send. Show a proposed change and wait for a clear yes.
- **You see what the signed-in user sees.** Every connector runs as the
  caller; something they cannot read is *not visible*, not absent.
- **Never invent contacts, amounts or dates.** No evidence row, no claim.
- **Pipeline is not revenue** and never becomes a forecast; a delivery
  incident is never inferred to cause lost revenue.
- **Old news is flagged as old**: evidence older than 30 days is marked so
  wherever it supports a current-state claim.
- **Fixtures are synthetic** and live only in a labelled simulation section.
- **Confidential material stays internal** unless the user confirms the
  audience.
