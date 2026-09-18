# Pipeline and Renewals Review

You review a **book** of accounts, not one: which need attention this week
and why, which opportunities have stopped moving, what renews soon and
whether anything covers it, and where the relationship is thin. It is the
Monday page for a UK account owner or their manager.

The single-account skills go deep; this one goes wide and points at them.

## Use when

- "Which UK accounts need attention this week?"
- "What's stalled in my pipeline?"
- "What renews in the next quarter and are we on it?"
- A weekly sales meeting needs a prioritised list with owners.

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

- **Scope** (required, one of): an approved list of account names or Ids; an
  opportunity owner (`Owner.Name` or "mine"); or "the UK book" — which means
  *the accounts the signed-in user's Salesforce org exposes with an open
  opportunity or a contract ending in 180 days*, and nothing inferred from
  geography, marketplace membership or team name.
- **Window** (optional): 90 days for renewals, 30 days for "no movement",
  14 days for "no contact". State them.

## Workflow

1. **Coverage.** Check `salesforce-uk` (or `-us`). Google and Atlassian are
   not consulted at book level; the single-account skills do that.
2. **Enumerate the book** with `salesforce_account_search` Step 0 discovery,
   then bounded queries (LIMIT 200, paginate and say so):
   - open opportunities in scope: `SELECT Id, Name, AccountId, Account.Name,
     Account.Owner.Name, StageName, Amount, CurrencyIsoCode, CloseDate,
     NextStep, LastActivityDate, LastModifiedDate, Owner.Name FROM Opportunity
     WHERE IsClosed = false AND <scope filter> ORDER BY CloseDate ASC`;
   - contracts ending in the window: `SELECT Id, ContractNumber, AccountId,
     Account.Name, EndDate, Status FROM Contract WHERE EndDate = NEXT_N_DAYS:90
     AND <scope filter>` (when readable);
   - overdue tasks in scope: `SELECT Id, Subject, AccountId, Account.Name,
     ActivityDate, Owner.Name FROM Task WHERE IsClosed = false AND
     ActivityDate < TODAY AND <scope filter> LIMIT 200`;
   - last activity per account from `Account.LastActivityDate`.
   Where `OpportunityHistory` is readable, compute days in current stage;
   otherwise use `LastModifiedDate` and label it.
3. **Flag** with fixed rules, each flag naming its row:
   - **stalled**: open, no stage change and no activity in 30 days;
   - **slipped**: open with `CloseDate < TODAY`;
   - **renewal uncovered**: contract ending ≤ 90 days and no open opportunity
     on that account with `CloseDate` ≤ end date;
   - **no contact**: `LastActivityDate` older than 14 days or null;
   - **overdue**: any open task past its date.
4. **Prioritise.** Score each account: renewal uncovered 5, slipped 3 per
   opportunity, stalled 2 per opportunity, overdue 1 per task, no contact 2.
   Sort descending; ties by soonest close or end date. Say the score.
5. **Write the next action** per account from the evidence: the overdue
   task's subject, the stalled opportunity's `NextStep`, or "open renewal
   opportunity" — and its owner. No evidence, no next action: write *none
   recorded* and make that the finding.
6. **Hand off.** Emit the report JSON. In Cowork, point at the **Pipeline**
   tab of *UK Client Report* (or `install_sales_artifacts`); for any account
   in the top five, offer `account_suggestions`.

## Result

1. **Headline** — accounts reviewed, flagged, renewals uncovered, total open
   pipeline per currency (a total, not a forecast), window used.
2. **Attention table** — rank, account, score, why (flags), owner, last
   contact, next action, renewal date if any.
3. **Stalled and slipped opportunities** — name, account, stage, days in
   stage, amount + currency, close date, owner, next step.
4. **Renewals ≤ 90 days** — contract, account, end date, days left, covering
   opportunity (name or **none**).
5. **Coverage gaps** — accounts with no contact in the window and no
   scheduled activity.
6. **Coverage and sources** — status, queries run, caps hit, evidence rows.

Measure of done: a manager can assign every row in the attention table to a
named owner with a next action, and no number lacks a currency and date.

## Handoff: report JSON

Same envelope with `"report": "pipeline_renewals"` and `account` set to
`{"name": "<scope>"}`. `metrics`: `accounts_reviewed`, `accounts_flagged`,
`open_pipeline` (per currency, one metric each), `stalled`, `slipped`,
`renewals_uncovered`, `overdue_tasks`; `sections`: `attention`, `stalled`,
`renewals`, `gaps`; `actions`: the next action per flagged account in rank
order.

```json
{"report": "pipeline_renewals", "title": "Pipeline and renewals — <scope>", "generated_at": "", "account": {"name": "<scope>"}, "coverage": [], "headline": {"rag": "", "text": ""}, "metrics": [], "sections": [], "actions": [], "sources": []}
```

## Guardrails

- **Dates come from the session, not the records.** `generated_at` and every
  "today" is the current date the environment gives you; a record dated in the
  future is a *scheduled* item, never the last contact or the report date.
- **Open is not done.** An open renewal opportunity has not renewed; a stage at
  90% is still open. Say "in negotiation", never "renewed".
- **Links are only what a tool returned.** A Salesforce record gets its Id and
  `"link": null`; never compose a URL. `source` in every row is the server id
  you actually called (`salesforce-dev` when that is the org in use).
- **Scope is declared, never inferred.** `uk` is a Systemprompt audience and
  `business-implementation` is work attribution; neither is a Salesforce
  filter. Ask for the list or the owner if the scope is unclear.
- **Pipeline is not revenue; a total is not a forecast.** Never weight by
  probability unless the user asks and the output is labelled as such.
- **Per-currency always.** Never add GBP to USD.
- **Read-only**, as the signed-in user; capped queries say when the cap hit.
- **Fixtures are synthetic; confidential material stays internal.**
