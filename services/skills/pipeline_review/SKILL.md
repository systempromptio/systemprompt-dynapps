# Pipeline Review

You produce a review of open deals: every open opportunity by stage and owner with
its amount and close date, deals flagged as stale when nothing has happened on
them for more than 30 days, and totals whose scope is written out. When Salesforce
is linked the numbers come from the CRM; when it is not, the review is built from
the open opportunities recorded in the knowledge bank and says so plainly.

## Use when

- "How is the pipeline looking this quarter?"
- "Which deals have gone quiet?"
- "Show me my open deals by stage."
- "What is closing this month and who owns it?"
- A weekly sales stand-up or a forecast conversation with leadership.

## Inputs

- **Scope** (optional): a single owner, the whole team, or one client. If the
  user does not say, ask once, then default to "all open opportunities the user
  can see" and state that in the output.
- **Period** (optional): closing this month, this quarter, or any close date.
  Default is any close date.
- **Stale threshold** (optional). Default is 30 days with no activity.

## Workflow

1. **Search the knowledge bank first.** Call the `knowledge-bank` MCP tool
   `search_knowledge` with `{query, top_k, source_types}` and keep each hit's
   `uri` and `date`:
   - `{"query": "open opportunities proposals in flight pending decisions", "top_k": 15, "source_types": ["client_accounts"]}`
   - If the scope is one client: `{"query": "<client> opportunity proposal renewal", "top_k": 8, "source_types": ["client_accounts"]}`
   - For context on why a deal may be stuck: `{"query": "<client> blocked waiting on client decision", "top_k": 5, "source_types": ["project_docs", "project_tickets"]}`
   `client_accounts` is where each client brief lists its open opportunities and
   their amounts; `project_docs` and `project_tickets` explain delivery blockers
   that stall a deal. Note each hit's `date`: an opportunity in a brief dated
   three months ago may have closed since.
2. **Check whether Salesforce is linked.** If the `salesforce` server is
   available, continue to step 3. If not, skip to step 5 and build the review from
   the knowledge-bank opportunities. Head the review with: "Built from the
   knowledge bank; Salesforce is not linked, so amounts and stages are as recorded
   in the client briefs on their stated dates."
3. **Pull open deals.** With the Salesforce query tool:
   `SELECT Name, Account.Name, StageName, Amount, Probability, CloseDate, Owner.Name, LastActivityDate, LastModifiedDate FROM Opportunity WHERE IsClosed = false [AND Owner.Name = :owner] [AND CloseDate = THIS_QUARTER] ORDER BY StageName, CloseDate`
   Then the rollup, where aggregate queries are permitted:
   `SELECT StageName, COUNT(Id), SUM(Amount) FROM Opportunity WHERE IsClosed = false GROUP BY StageName`
   and the same grouped by `Owner.Name`. If aggregates are not permitted, fetch
   the rows with a sensible `LIMIT`, total them yourself, and say the result was
   capped.
4. **Flag stale and slipping deals.** Stale: `LastActivityDate` (or
   `LastModifiedDate` if activity is null) older than the threshold. Slipping:
   `CloseDate < TODAY` while still open. Both are called out by name, never
   buried in a total.
5. **Reconcile the two sources.** A deal that appears in a client brief but not in
   the CRM, or the other way round, is listed under "Not in both sources" with the
   date of each. Never merge amounts from the two sources into one total.
6. **Total with scope.** Every total is written as "N open deals worth $X USD,
   <scope>, close date <period>, from <system> on <date>". Weighted value, if
   asked for, is the sum of amount × probability ÷ 100, and says so.
7. **Return the table and summary** in chat. If a working folder is connected,
   also save it as `briefs/pipeline-<YYYY-MM-DD>.md` and report the path.

## Result

1. **Scope line** — whose deals, which period, which system, read when.
2. **Table** — one row per open deal: deal, client, stage, amount, close date,
   owner, last activity, flags (stale / slipping / not in both sources).
3. **By stage** — count and total per stage, in stage order.
4. **By owner** — count and total per owner.
5. **Stale deals** — named, with days since last activity and the most recent
   evidence from the knowledge bank on why.
6. **Summary** — three to five sentences: total, what is closing soon, what is
   at risk, and the one thing to act on this week.
7. **Sources** — knowledge-bank hits (`uri`, date) and CRM queries run, or the
   "built from the knowledge bank" note if Salesforce was not linked.

Measure of done: every amount in the table has a stage, owner, and close date,
the totals add up to the table, and the scope line leaves nothing ambiguous.

## Guardrails

- **Read-only on the CRM.** Stage changes, close dates, and amounts are never
  edited here. If the user asks for a change, show the record and the old → new
  value and wait for a clear yes.
- **Never invent amounts or close dates.** A deal with no amount is listed as
  "amount not recorded", not estimated.
- **State the scope and date of every number.** A total with no scope is
  misleading in a forecast conversation, so it does not appear.
- **Stages and probabilities are per org.** Read the real values; if the org uses
  custom forecast fields, prefer those and say so.
- **Currency is stated on first use** ("$250,000 USD") and never mixed across
  currencies in one total.
