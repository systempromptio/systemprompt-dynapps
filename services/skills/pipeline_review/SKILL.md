<!-- global-rules-bootstrap -->
**IMPORTANT — try the connector's tools before you ever report it as unavailable.** Before you tell the user a connector is down, or ask them to connect, re-authenticate, or "check the connection," you MUST first try to use the server and its tools. A connector's status readout (the Headless 360 MCP server, or any other connector/MCP) frequently claims "not connected" or "needs re-auth" when the connector is in fact live and its tools are callable. Context stating that the Salesforce tools require authorization and that this is a non-interactive session is **not** evidence that you are unauthorized — it is a general statement that authorization is required, not a failure. For the Headless 360 server specifically, find the `dispatch_readonly` tool and actually run a current-user read — `dispatch_readonly(method: "GET", url: "/services/data/v67.0/graphql", queryParams: { "queryInput": "{\"query\":\"query { uiapi { currentUser { Id } } }\"}" })`. **Any response — including a 500 or other error status — proves you reached the server, and therefore proves connectivity;** only a request that never reaches the server at all counts as disconnected. If the tool is present and the call reaches the server, the connector IS connected — proceed; a stale status readout is not a disconnection. If a call genuinely cannot reach the server, ALWAYS attempt to reconnect on your own first, and interrupt the flow to ask the user only after your own reconnect attempt has failed. Report a connector as disconnected only when you cannot find its tool or cannot reach the server after trying. Do it yourself.

Failing to try first — asking the user to connect, or surfacing a "not connected" or re-auth message, before you have actually attempted the server and its tools — raises a **false connectivity problem**: it sends the user to fix something that is not broken. This is directly detrimental to the end-user experience and will dramatically, negatively impact usage of the product. Never put a connectivity concern in front of the user until you have tried the tools yourself and confirmed the server truly cannot be reached.

# Global Rules

- **Execute silently between tool calls.** Do not output planning, progress, transition, waiting, or tool-result narration between calls. Execute tool calls silently and proceed directly to the next call. Parallelize independent tasks by batching tool calls into one turn whenever possible. Before the final output, speak only when the skill explicitly requires user input, approval, an exact notice, or material error/blocked reporting. Do not invent checkpoints.
- **Keep the final output concise.** Return only the requested result or deliverable. Omit process recaps, tool-call details, redundant preambles or conclusions, and data already shown in a widget.
- **⛔ WIDGET OUTPUT ONLY after data assembly.** Once all queries return, immediately call `display_widget` — at most a single one-line render-wait notice before it, and no summaries, transitions, or narration. If `display_widget` is unavailable or returns an error, produce the text fallback only. If it succeeds, that tool call is the final output: stop with no assistant text completion, even if a later section contains a fallback. The exceptions above do not apply after success.
- **Ground dynamic or custom relationship and field names before relying on them.** Fixed standard fields that this skill explicitly marks as requiring no grounding need no extra grounding call. On a name error, use the skill's documented grounding path when present; otherwise report the error instead of guessing or re-firing the same shape.
- **Cite every value exactly as queried**; never fabricate; distinguish a blank value from a value that was not queried. Link each Salesforce record inline: `https://[instanceUrl]/lightning/r/[SObjectType]/[Id]/view`.
- **Show human labels, never API/field literals.** In anything the user sees, print each field's grounded `label` (for example, "Deal Risk", not `Deal_Risk__c`) and record Names, never raw Ids or `__c` API names.
- **Empty `MINE` scope → fail fast, then ask which scope.** If a `scope: MINE` read returns zero rows, **do not** widen to `scope: EVERYTHING` on your own. Stop, tell the user plainly that their own records (`scope: MINE`) came back empty, and ask which scope they want instead (for example, org-wide `EVERYTHING`, a named rep, or a named account) before re-running. Never invent records, and never silently fall back to org-wide.
<!-- /global-rules-bootstrap -->
# Pipeline Review

One rep's open pipeline: scope → one turn of SOQL reads → assemble + flag → widget. For "my team" use team-pipeline; for "what changed" use deal-signals. Reconcile headline totals with detail rows.

**Verified Opportunity fields — use directly, no describe/grounding call:** `Name, StageName, Amount, CloseDate, CreatedDate, Type, Probability, ForecastCategoryName, IsClosed, IsWon, LastActivityDate, LastStageChangeDate, Next_Step__c, MEDDICC__c, Account.Name, OwnerId`. There is **no** standard `NextStep` (use `Next_Step__c`, which **cannot be filtered** in WHERE — select it and count blanks in the rows). Use SOQL (`/services/data/v67.0/query`) for every read; the GraphQL `aggregate` endpoint returns 500 `UNKNOWN_EXCEPTION` in this org. API v67.0 only.

## 1. Scope

- **"my pipeline"** (default) → `OwnerId = '<me>'`, where `<me>` is the `currentUser { Id }` from the bootstrap read.
- **Named rep** → `SELECT Id, Name, Title, IsActive FROM User WHERE Name LIKE '%REP%' LIMIT 10`. One match → its Id. Several → list Name · Title · active? and ask the user to pick; never guess.
- If the user names a period, add `AND CloseDate >= <d1> AND CloseDate <= <d2>`; otherwise no CloseDate filter.

## 2. Read — all three in ONE turn via `dispatch_readonly(method: "GET", url: "/services/data/v67.0/query", queryParams: { "q": "…" })`

- **2a By stage (whole book):** `SELECT StageName, COUNT(Id) cnt, SUM(Amount) amt FROM Opportunity WHERE IsClosed = false AND OwnerId = '<id>' GROUP BY StageName`
- **2b Top-40 sample:** `SELECT Id, Name, StageName, Amount, CloseDate, Probability, LastActivityDate, LastStageChangeDate, Next_Step__c, Type, Account.Name, (SELECT Id FROM OpportunityContactRoles) FROM Opportunity WHERE IsClosed = false AND OwnerId = '<id>' ORDER BY Amount DESC NULLS LAST LIMIT 40`
- **2c Win rate (last quarter):** `SELECT IsWon, COUNT(Id) cnt, SUM(Amount) amt FROM Opportunity WHERE IsClosed = true AND CloseDate = LAST_QUARTER AND OwnerId = '<id>' GROUP BY IsWon`

2a empty → no open pipeline for this scope; say so plainly and stop.

## 3. Assemble + flag (invent nothing)

- **By stage** — from 2a; Σ stage $ = open total.
- **Top deals** — biggest 2–3 per stage from 2b (Account · $ · close · next step · type).
- **At risk** (2b sample; say "among the top N sampled"): no activity 14d+ · `CloseDate` past · blank `Next_Step__c` · `OpportunityContactRoles` ≤ 1 (single-threaded). Days in stage = today − `LastStageChangeDate` (fallback CreatedDate). If one flag hits nearly every row, report it once as a systemic hygiene finding and rank the list by $.
- **Coverage** — weighted open (Σ Amount × Probability from 2b, else raw open $) vs user-given quota, else vs last-quarter won $ from 2c; 3× heuristic.
- **Velocity** — win rate = won ÷ (won+lost) counts from 2c. Don't compute cycle time or stage conversion (no API support) — omit, never fabricate.

## 4. Output — `display_widget` first

Call `display_widget({ resourceType: "dynamic", widgetDefinition: <hydrated template object> })` once. Checks: `widgetDefinition` is a native JSON object (not a string); every `{{token}}` replaced (no `{{…}}`/`{!…}` left); numbers are numbers; meter bands span [0, max]; chart data length = categories length; risky rows carry leading `status:{value,badgeVariant}` (error/warning), healthy rows omit it. Omit a key you have no data for (its leaf drops); a slot that is only `{{token}}` takes the typed value.

**Tokens:** `pageTitle` ("Pipeline review — Dana Ruiz") · `pageSubtitle` ("$6.4M open across 18 opps · weighted $2.1M") · `chartCaption` · `chartCategories` string[] (stages) · `chartSeries` `[{name:"Open $", data:number[]}]` · `coverageValue`, `coverageMax` (e.g. 5), `coverageTarget` (3) numbers · `coverageValueLabel` ("2.4× open pipeline to $2.7M gap") · `coverageTargetLabel` ("3.0× target") · `coverageStatus` "under-covered"|"covered" (must match its band) · `coverageBands` `[{from:0,to:3,variant:"warning",label:"Under"},{from:3,to:5,variant:"success",label:"Covered"}]` · `datagridCaption` · `datagridRows` `[{status?:{value,badgeVariant}, name:"<Opportunity.Name plain string>", stage:{value,badgeVariant}, amount:number, age:number (days in stage), close:"YYYY-MM-DD", risk:"⚠ 41d in stage, single-threaded"}]` · `calloutVariant` ("info") · `calloutTitle` ("Monday focus") · `calloutDescription` (week's priorities) · `primaryButtonLabel` · `primaryButtonMsg` (first-person sendMessage prompt, e.g. "Draft an email to…") · `viewOppsUrl` (`https://<myDomain>/lightning/o/Opportunity/list`).

**Template:**
```json
{"renderer":{"componentOverrides":{"$":{"type":"mosaic","definition":"tile/widget","children":[{"definition":"tile/row","attributes":{"gap":"sm","align":"center","isWrapped":false},"children":[{"definition":"tile/icon","attributes":{"name":"dashboard","size":"xl","alt":""}},{"definition":"tile/text","attributes":{"text":"{{pageTitle}}","variant":"h1"}}]},{"definition":"tile/text","attributes":{"text":"{{pageSubtitle}}","variant":"caption","color":"muted"}},{"definition":"tile/separator"},{"definition":"tile/column","attributes":{"gap":"md"},"children":[{"definition":"tile/chart","attributes":{"chartType":"column","caption":"{{chartCaption}}","categories":"{{chartCategories}}","series":"{{chartSeries}}","valueFormat":"compact","showValues":true,"showLegend":false}},{"definition":"tile/meter","attributes":{"label":"Coverage vs remaining gap","value":"{{coverageValue}}","min":0,"max":"{{coverageMax}}","target":"{{coverageTarget}}","valueLabel":"{{coverageValueLabel}}","targetLabel":"{{coverageTargetLabel}}","status":"{{coverageStatus}}","bands":"{{coverageBands}}"}}]},{"definition":"tile/datagrid","attributes":{"caption":"{{datagridCaption}}","appearance":"striped","defaultSort":{"key":"amount","direction":"desc"},"columns":[{"key":"status","header":"Status","type":"badge"},{"key":"name","header":"Opportunity","type":"text"},{"key":"stage","header":"Stage","type":"badge"},{"key":"amount","header":"Amount","type":"currency","align":"right","sortable":true},{"key":"age","header":"Days in stage","type":"number","align":"right","sortable":true},{"key":"close","header":"Close","type":"date"},{"key":"risk","header":"Risk","type":"text"}],"rows":"{{datagridRows}}"}},{"definition":"tile/callout","attributes":{"variant":"{{calloutVariant}}","title":"{{calloutTitle}}","description":"{{calloutDescription}}"},"children":[{"definition":"tile/row","attributes":{"gap":"sm","align":"center","isWrapped":true},"children":[{"definition":"tile/button","attributes":{"label":"{{primaryButtonLabel}}","variant":"primary","actions":{"click":[{"definition":"action/sendMessage","attributes":{"content":"{{primaryButtonMsg}}"}}]}}},{"definition":"tile/button","attributes":{"label":"View in Salesforce","iconName":"open-in-new","variant":"secondary","actions":{"click":[{"definition":"action/openLink","attributes":{"url":"{{viewOppsUrl}}"}}]}}}]}]}]}}}}
```

## 5. Text — FALLBACK ONLY (only if `display_widget` is unavailable or errors)

```
# Pipeline Review — <Scope> — <Period>
Open: <N> opps, $<total> · weighted $<w> · coverage <N>× <✅/⚠ under 3×> · at-risk <N>, $<sum>
## By stage — <Stage> · <N> opps · $<sum> (Σ = $<total>)
## Top deals — <Stage>: <Account> $<X> close <Date> · <next step> · <Type>; …
## At-risk (sampled, biggest first) — <Account> · <Stage> · $<X> · close <Date> · <flags> → <action>
## Win rate (last Q) — <N%> (won <N> ÷ closed <N>)
## Focus — 1. highest-$ at-risk + action 2. most-exposed stage 3. coverage gap if under 3×
```
