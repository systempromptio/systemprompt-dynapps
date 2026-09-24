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
# Account Tiering

Read (one call) → activity (one call) → score → tier → widget. Scope: "my accounts", a named list, or a list view (ask for names instead). Default tiers A/B/C + Deprioritize. Resolve an ambiguous name by asking the user — don't probe or guess.

## 1. Read accounts (no grounding call — fields below are verified for this org)

`%SCOPE%` = `scope: MINE` for "my accounts"; `where: { or: [{ Name: { like: \\\"%Acme%\\\" } }, ...] }` for a named list.

```
dispatch_readonly(method: "GET", url: "/services/data/v67.0/graphql",
  queryParams: { "queryInput": "{\"query\":\"query { uiapi { query { Account(%SCOPE%, first: 200) { edges { node { Id Name { value } Industry { value } NumberOfEmployees { value } AnnualRevenue { value displayValue } Account_Type__c { value } Client_Tier__c { value } Segment__c { value } Account_Health__c { value } Account_Status__c { value } Sum_of_Closed_Won_Opportunities__c { value } Sum_of_All_Open_Opportunities__c { value } LastActivityDate { value } Owner { Name { value } } Opportunities { edges { node { Id StageName { value } IsClosed { value } Amount { value } CloseDate { value } } } } Contacts { edges { node { Id Title { value } } } } } } } } } }\"}" })
```

Never add `Type` (it does not exist on Account; use `Account_Type__c`). `first: 200` caps the book — if larger, say so and ask the user to narrow it.

## 2. Activity (Salesforce Event + Task, one turn, after step 1)

Task is usually empty in this org; Event holds the activity. Fire both in parallel with the Account Ids from step 1:
```
dispatch_readonly(method: "GET", url: "/services/data/v67.0/query",
  queryParams: { "q": "SELECT AccountId, COUNT(Id) n, MAX(ActivityDate) lastDate FROM Event WHERE AccountId IN ('<ids>') AND ActivityDate = LAST_N_DAYS:90 GROUP BY AccountId" })
```
Same shape with `FROM Task`. No rows → signal = 0; never block on it.

## 3. Score ICP fit (0-10)

ICP (industries, size, titles, disqualifiers) is inferred from the org's won accounts or asked once — never invented.

| Signal | Scoring |
|---|---|
| Industry match | exact=3, adjacent=1, off=0 |
| Size (employees/revenue) in range | in=3, ±50%=1, out=0 |
| Target persona in Contact titles | yes=2, maybe=1, none=0 |
| No disqualifiers | clean=2, soft=1, hard=0 |

## 4. Score engagement (0-10)

| Signal | Scoring |
|---|---|
| Open opp (`IsClosed=false`) | yes=3, no=0 |
| LastActivityDate | <30d=3, 30-90d=2, 90-180d=1, older=0 |
| Meetings last 90d (step 2) | 3+=2, 1-2=1, 0=0 |
| Contacts on account | 3+=2, 2=1, ≤1=0 |

## 5. Tier

- **A** high fit, high engagement: progress the open opp, multi-thread.
- **B** high fit, low engagement: activation — outbound, find a trigger.
- **C** low fit, high engagement: one discovery call to confirm or DQ.
- **Deprioritize** low/low: revisit quarterly.

## 6. Output — widget first

Call `display_widget({ resourceType: "dynamic", widgetDefinition: <hydrated template> })` once; `widgetDefinition` is a native JSON object, never a string. Replace each `{{token}}` with a typed value (a slot that is only a token becomes that value — arrays stay arrays); omit keys you have no data for; leave no `{{…}}`.

Tokens:
- `title` ("Account tiering — book of 40"), `subtitle` (scoring basis + tier counts)
- `heatmapCaption`; `heatmapXLabels` engagement bands low→high, ≤12 chars (["Low engage","Mid engage","High engage"]); `heatmapYLabels` fit bands high→low, ≤16 chars; `heatmapDomain` [min,max] count; `heatmapCells` [{x,y,value,valueLabel?}] one per occupied quadrant, e.g. {x:"High engage",y:"High fit",value:6,valueLabel:"6 · Tier A"}
- `piechartCaption`, `piechartCenterLabel` ("Accounts"), `piechartCenterValue` (string total), `piechartSlices` [{label,value,role?}] with role "highlight" on Tier A
- `datagridCaption`; `datagridRows` one per Tier A account: {account, fit: 1-5, engage: 0-100, arr: Sum_of_Closed_Won value, white: Sum_of_All_Open value, motion:{value:"Expand"|"Deepen"|…, badgeVariant: success|info|warning|error}}
- `calloutTitle`, `calloutDescription` (biggest coverage gap + move), `primaryButtonLabel`, `primaryButtonMsg` (first-person follow-up prompt), `salesforceUrl` (Account list view Lightning URL)

```json
{"renderer":{"componentOverrides":{"$":{"type":"mosaic","definition":"tile/widget","children":[{"definition":"tile/row","attributes":{"gap":"sm","align":"center","isWrapped":false},"children":[{"definition":"tile/icon","attributes":{"name":"layers","size":"xl","alt":""}},{"definition":"tile/text","attributes":{"text":"{{title}}","variant":"h1"}}]},{"definition":"tile/text","attributes":{"text":"{{subtitle}}","variant":"caption","color":"muted"}},{"definition":"tile/separator"},{"definition":"tile/row","attributes":{"gap":"lg","align":"stretch","isWrapped":false},"children":[{"definition":"tile/column","attributes":{"width":"stretch"},"children":[{"definition":"tile/heatmap","attributes":{"layout":"matrix","caption":"{{heatmapCaption}}","xLabels":"{{heatmapXLabels}}","yLabels":"{{heatmapYLabels}}","encode":"color","scale":"sequential","valueFormat":"number","domain":"{{heatmapDomain}}","cells":"{{heatmapCells}}"}}]},{"definition":"tile/column","attributes":{"width":"stretch"},"children":[{"definition":"tile/piechart","attributes":{"caption":"{{piechartCaption}}","variant":"donut","valueFormat":"number","centerLabel":"{{piechartCenterLabel}}","centerValue":"{{piechartCenterValue}}","slices":"{{piechartSlices}}"}}]}]},{"definition":"tile/datagrid","attributes":{"caption":"{{datagridCaption}}","appearance":"striped","size":"sm","defaultSort":{"key":"fit","direction":"desc"},"columns":[{"key":"account","header":"Account","type":"text"},{"key":"fit","header":"ICP fit","type":"number","align":"right","sortable":true},{"key":"engage","header":"Engagement","type":"databar","target":100},{"key":"arr","header":"ARR","type":"currency","align":"right","sortable":true},{"key":"white","header":"Whitespace","type":"currency","align":"right"},{"key":"motion","header":"Motion","type":"badge"}],"rows":"{{datagridRows}}"}},{"definition":"tile/callout","attributes":{"variant":"recommended","title":"{{calloutTitle}}","description":"{{calloutDescription}}"},"children":[{"definition":"tile/row","attributes":{"gap":"sm","align":"center","isWrapped":true},"children":[{"definition":"tile/button","attributes":{"label":"{{primaryButtonLabel}}","variant":"primary","actions":{"click":[{"definition":"action/sendMessage","attributes":{"content":"{{primaryButtonMsg}}"}}]}}},{"definition":"tile/button","attributes":{"label":"View in Salesforce","iconName":"open-in-new","variant":"secondary","actions":{"click":[{"definition":"action/openLink","attributes":{"url":"{{salesforceUrl}}"}}]}}}]}]}]}}}}
```

## 7. Text — FALLBACK ONLY, if `display_widget` failed

```
# Account Tiering - [N] accounts
## Tier A - Active Pursuit ([N], $[pipeline])
| Account | Fit | Eng | Open Opp | Last Touch | Next Action |
## Tier B - Activate ([N])
## Tier C - Qualify or DQ ([N])
## Deprioritized ([N]) — names only
## Coverage Gaps
- [N] Tier A/B accounts with no activity in 30+ days
- [N] accounts missing Industry or NumberOfEmployees (can't score)
```
