<!-- global-rules-bootstrap -->
**Try the connector before reporting it unavailable.** A status readout saying "not connected"/"needs re-auth" is often stale, and a note that Salesforce needs authorization is not evidence of failure. Before telling the user to connect or re-authenticate, run (the tool whose name ends in `salesforce__dispatch_readonly`; never a `google_workspace` tool such as `search`, and no `google_workspace` calls in this skill) `dispatch_readonly(method: "GET", url: "/services/data/v67.0/graphql", queryParams: { "queryInput": "{\"query\":\"query { uiapi { currentUser { Id } } }\"}" })`. Any response, even a 500, proves connectivity — proceed. Only if the call cannot reach the server, try to reconnect yourself first, then ask the user. Raising a false connectivity problem is seriously harmful to the user experience.

# Global Rules

- **Execute silently between tool calls.** Do not output planning, progress, transition, waiting, or tool-result narration between calls. Execute tool calls silently and proceed directly to the next call. Parallelize independent tasks by batching tool calls into one turn whenever possible. Before the final output, speak only when the skill explicitly requires user input, approval, an exact notice, or material error/blocked reporting. Do not invent checkpoints.
- **Keep the final output concise.** Return only the requested result or deliverable. Omit process recaps, tool-call details, redundant preambles or conclusions, and data already shown in a widget.
- **⛔ WIDGET OUTPUT ONLY after data assembly.** Once all queries return, immediately call `display_widget` — at most a single one-line render-wait notice before it, and no summaries, transitions, or narration. If `display_widget` is unavailable or returns an error, produce the text fallback only. If it succeeds, that tool call is the final output: stop with no assistant text completion, even if a later section contains a fallback. The exceptions above do not apply after success.
- **Ground dynamic or custom relationship and field names before relying on them.** Fixed standard fields that this skill explicitly marks as requiring no grounding need no extra grounding call. On a name error, use the skill's documented grounding path when present; otherwise report the error instead of guessing or re-firing the same shape.
- **Cite every value exactly as queried**; never fabricate; distinguish a blank value from a value that was not queried. Link each Salesforce record inline: `https://[instanceUrl]/lightning/r/[SObjectType]/[Id]/view`.
- **Show human labels, never API/field literals.** In anything the user sees, print each field's grounded `label` (for example, "Deal Risk", not `Deal_Risk__c`) and record Names, never raw Ids or `__c` API names.
- **Empty `MINE` scope → fail fast, then ask which scope.** If a `scope: MINE` read returns zero rows, **do not** widen to `scope: EVERYTHING` on your own. Stop, tell the user plainly that their own records (`scope: MINE`) came back empty, and ask which scope they want instead (for example, org-wide `EVERYTHING`, a named rep, or a named account) before re-running. Never invent records, and never silently fall back to org-wide.
<!-- /global-rules-bootstrap -->

# Forecast Narrative

Aggregate the number + sample the deals (ONE turn) → narrative. Scope: my forecast (default), or a named rep/team. Period: current quarter (default).

## 1. Fields (verified — no grounding/describe call)

Opportunity: `Name` StageName `ForecastCategory` Amount CloseDate Probability `Next_Step__c` (label "Next Step"; there is NO `NextStep`) LastActivityDate IsClosed IsWon `MEDDICC__c`; Account Name; Owner Name. Buckets: **Closed** (Closed Won), **Commit**, **Best Case**, **Pipeline**; **Omitted** = drop. API v67.0 only.

## 2. Aggregate + sample — ISSUE BOTH IN ONE TURN

Never sum a `first: N` page for the number — use the server-side aggregate. `%QSTART%`/`%QEND%` = explicit `YYYY-MM-DD` quarter bounds (never `THIS_QUARTER`); bound both ends of CloseDate. `scope: MINE` = running user's book. Named rep/team: replace `scope: MINE` with `where: { OwnerId: { eq: \"<UserId>\" } ...}` (resolve the User by `Name like`), and keep `Owner { Name }` in the sample.

**2a. The number** (count + $ per ForecastCategory):
```
dispatch_readonly(method: "GET", url: "/services/data/v67.0/graphql",
  queryParams: { "queryInput": "{\"query\":\"query { uiapi { aggregate { Opportunity(scope: MINE, where: { CloseDate: { gte: { value: \\\"%QSTART%\\\" }, lte: { value: \\\"%QEND%\\\" } } }, groupBy: { ForecastCategory: { group: true } }) { edges { node { aggregate { ForecastCategory { value } Id { count { value } } Amount { sum { value } } } } } totalCount } } } }\"}" })
```
If the aggregate errors, don't re-fire it; use SOQL scoped by owner (`%OWNERID%` from the bootstrap `currentUser` read, or the resolved rep):
`GET /services/data/v67.0/query?q=SELECT ForecastCategory, COUNT(Id) cnt, SUM(Amount) amt FROM Opportunity WHERE OwnerId = '%OWNERID%' AND CloseDate >= %QSTART% AND CloseDate <= %QEND% GROUP BY ForecastCategory`

**2b. Deal sample** (top 50 open by Amount):
```
dispatch_readonly(method: "GET", url: "/services/data/v67.0/graphql",
  queryParams: { "queryInput": "{\"query\":\"query { uiapi { query { Opportunity(scope: MINE, where: { IsClosed: { eq: false }, CloseDate: { gte: { value: \\\"%QSTART%\\\" }, lte: { value: \\\"%QEND%\\\" } } }, orderBy: { Amount: { order: DESC } }, first: 50) { edges { node { Id Name { value } StageName { value displayValue } ForecastCategory { value displayValue } Amount { value displayValue } CloseDate { value } Probability { value } Next_Step__c { value } LastActivityDate { value } Account { Name { value } } Owner { Name { value } } } } } } } }\"}" })
```
`InvalidSyntax`/`<EOF>` → a `}` is missing; fix once, never re-send identical text.

**2c. Changes** (same turn; SOQL only): `GET /services/data/v67.0/query?q=SELECT OpportunityId, Field, OldValue, NewValue, CreatedDate FROM OpportunityFieldHistory WHERE CreatedDate >= %QSTART% AND Field IN ('StageName','Amount','CloseDate','ForecastCategoryName') ORDER BY CreatedDate DESC`. A pasted prior snapshot takes precedence — diff it instead. Empty → skip the delta section.

## 3. Build the narrative (no extra reads)

- The Number = 2a totals (whole book). Per-deal lines = top Commit/Best Case deals from 2b; say they are the top sample.
- Per Commit/Best Case deal: one line — status, what's needed, risk (cite Next Step, Last Activity). At risk = Best Case, or Commit with blank Next Step or Last Activity 7d+ old. Team rollup: max 5 deals per rep.
- One deal ≥~40% of Commit → headline the concentration risk.
- Link each deal.

## 4. Widget (default when `display_widget` exists)

Call `display_widget({ resourceType: "dynamic", widgetDefinition: <hydrated template> })` once; `widgetDefinition` is a native JSON object, never a string. Replace every `{{token}}` (a slot that is only a token takes the typed value; omit a token you lack data for); leave no `{{…}}`.

Tokens:
- strings: `title` ("Q3 forecast — Dana Ruiz"), `headerStatus` ("88% TO QUOTA"), `subtitle`, `meterLabel`, `meterValueLabel`, `meterTargetLabel`, `meterStatus`, `waterfallCaption`, `datagridCaption`, `calloutTitle` / `calloutDescription` (key risk or move), `primaryButtonLabel`/`primaryButtonContent`, `secondaryButtonLabel`/`secondaryButtonContent` (content = first-person follow-up prompt, e.g. "Draft a forecast email for Q3 covering…").
- numbers: `meterValue`, `meterMax`, `meterTarget`.
- `meterBands`: `[{from,to,variant:"error"|"warning"|"success",label}]` covering 0→max.
- `waterfallStart`: `{label:"Closed",value}`; `waterfallSteps`: `[{label:"+ Commit",delta},{label:"+ Best case",delta},{label:"+ Pipeline",delta}]`; `waterfallEnd`: `{label:"Total open"}`.
- `datagridRows`: `[{status?:{value,badgeVariant},name (plain string from Name.value),amount,prob,close:"YYYY-MM-DD",call:{value:"Commit"|"Best case"|"Pipeline",badgeVariant}}]`; set `status:{value:"At risk",badgeVariant:"warning"}` only on at-risk rows. Empty array drops the grid.

Template:
```json
{"renderer":{"componentOverrides":{"$":{"type":"mosaic","definition":"tile/widget","children":[{"definition":"tile/row","attributes":{"gap":"sm","align":"center","justify":"between","isWrapped":true},"children":[{"definition":"tile/row","attributes":{"gap":"sm","align":"center"},"children":[{"definition":"tile/icon","attributes":{"name":"trending-up","size":"xl","alt":""}},{"definition":"tile/text","attributes":{"text":"{{title}}","variant":"h1"}}]},{"definition":"tile/badge","attributes":{"label":"{{headerStatus}}","variant":"warning"}}]},{"definition":"tile/text","attributes":{"text":"{{subtitle}}","variant":"caption","color":"muted"}},{"definition":"tile/separator"},{"definition":"tile/meter","attributes":{"label":"{{meterLabel}}","value":"{{meterValue}}","min":0,"max":"{{meterMax}}","target":"{{meterTarget}}","valueFormat":"currency","valueLabel":"{{meterValueLabel}}","targetLabel":"{{meterTargetLabel}}","status":"{{meterStatus}}","size":"lg","bands":"{{meterBands}}"}},{"definition":"tile/waterfall","attributes":{"caption":"{{waterfallCaption}}","valueFormat":"currency","showValues":true,"size":"lg","start":"{{waterfallStart}}","steps":"{{waterfallSteps}}","end":"{{waterfallEnd}}"}},{"definition":"tile/datagrid","attributes":{"caption":"{{datagridCaption}}","appearance":"striped","defaultSort":{"key":"amount","direction":"desc"},"columns":[{"key":"status","header":"Status","type":"badge"},{"key":"name","header":"Deal","type":"text"},{"key":"amount","header":"Amount","type":"currency","align":"right","sortable":true},{"key":"prob","header":"Prob.","type":"number","align":"right"},{"key":"close","header":"Close","type":"date"},{"key":"call","header":"My call","type":"badge"}],"rows":"{{datagridRows}}"}},{"definition":"tile/callout","attributes":{"variant":"warning","title":"{{calloutTitle}}","description":"{{calloutDescription}}"},"children":[{"definition":"tile/row","attributes":{"gap":"sm","align":"center","isWrapped":true},"children":[{"definition":"tile/button","attributes":{"label":"{{primaryButtonLabel}}","variant":"primary","actions":{"click":[{"definition":"action/sendMessage","attributes":{"content":"{{primaryButtonContent}}"}}]}}},{"definition":"tile/button","attributes":{"label":"{{secondaryButtonLabel}}","variant":"secondary","actions":{"click":[{"definition":"action/sendMessage","attributes":{"content":"{{secondaryButtonContent}}"}}]}}}]}]}]}}}}
```

## 5. Text — FALLBACK ONLY — DO NOT USE IF `display_widget` SUCCEEDED

```
# Forecast — <Scope> — <Period>
## The Number
| | $ | # |
|---|---|---|
| Closed Won | $<X> | <N> |
| Commit | $<X> | <N> |
| Best Case | $<X> | <N> |
| Pipeline | $<X> | <N> |
| **Commit Total** | **$<Closed+Commit>** | |

## Changes Since Last Week
- ⬆/⬇/✅/❌ <Deal> … (or "No prior snapshot — skipping delta")

## Commit Deals
- **<Account>** $<X> closing <Date> — <status, what's needed, risk>

## Best Case Deals
- **<Account>** $<X> — <what gets it to commit>

## Risk to Commit
- <Deal + specific risk + mitigation>

## On the Radar
- <Pipeline/Best-Case deals that could pull INTO the number if accelerated, or slip OUT — the upside/downside not yet in Commit>

## Asks
- <exec help / resourcing / unblocks>
```
**Concentration callout:** if a single deal is a large share (~40%+) of the Commit number, call it out explicitly — a single-deal quarter is a risk headline, not a footnote.

ForecastCategory should change? Preview the one-field change and use `dispatch` only after explicit confirmation, then verify it with `dispatch_readonly`. "What if <deal> slips?" → `deal-slip-scenario`. Submission stays manual.
