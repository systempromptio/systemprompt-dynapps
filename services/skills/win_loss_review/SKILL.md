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

# Win-Loss Review

Read → find patterns → widget. Leader-focused: patterns across recently closed opportunities. Salesforce only — no transcripts, email or documents; qualitative evidence comes from Reason Lost, Next Step and logged Event/Task activity. No grounding call: fields below are verified (`ReasonLost__c` = Reason Lost; `OpportunityHistories` = stage history child; Account type is `Account_Type__c`). There is **no competitor field** — never report "lost to" unless Reason Lost text names one.

## Inputs

- **Scope:** team (default, no owner filter) · "mine" (`scope: MINE`) · a named rep (resolve one `User` by `Name: { like }`, then `OwnerId: { eq: \"<Id>\" }`).
- **Period:** last quarter (default) — compute explicit `gte`/`lte` dates; never `LAST_N_DAYS`.
- **Focus:** all / wins / losses (add `IsWon` filter).

## 1. Read — one dispatch

```
dispatch_readonly(method: "GET", url: "/services/data/v67.0/graphql",
  queryParams: { "queryInput": "{\"query\":\"query { uiapi { query { Opportunity(where: { IsClosed: { eq: true }, CloseDate: { gte: { value: \\\"%START%\\\" }, lte: { value: \\\"%END%\\\" } } }, first: 200, orderBy: { Amount: { order: DESC } }) { edges { node { Id Name { value } StageName { value } Amount { value } CloseDate { value } CreatedDate { value } LeadSource { value } Type { value } IsWon { value } ReasonLost__c { value } Next_Step__c { value } Account { Name { value } Industry { value } Account_Type__c { value } } Owner { Name { value } } OpportunityHistories(first: 15, orderBy: { CreatedDate: { order: DESC } }) { edges { node { StageName { value } CreatedDate { value } } } } } } } } } }\"}" })
```

Empty `edges` → say no closed deals in this window/scope and stop. If 200 rows return, note the cap. On a `FieldUndefined` error drop only that field and re-run once.

Optional second call, only for the 5 largest losses and 5 largest wins: their Events (`WhatId: { in: [<opp Ids>] }`, fields `Subject StartDateTime WhatId`) to show engagement depth. Skip it if the first read is empty.

## 2. Patterns (record only; reconcile totals; invent nothing)

- **Win rate** overall and by rep, lead source, size band, industry, type.
- **Where losses die** — last non-closed `StageName` in each lost opp's history (not "Closed Lost"); fall back to final stage only if history is empty.
- **Median days in stage** from consecutive history `CreatedDate` deltas (omit if no history).
- **Cycle length** (CloseDate − CreatedDate) and **median Amount**, wins vs losses.
- **Loss reasons** — count Reason Lost values; blank = "Not recorded" (a data-quality finding in itself).
- **Qualitative** — for the 5 largest wins/losses cite Reason Lost, Next Step and meeting counts: "[Account] — lost at Proposal, Reason Lost: Pricing, 1 meeting in last 60 days."

## 3. Output — widget first

Call `display_widget` once data is assembled (at most one line "Displaying the visualization now (this may take a minute)." before it). `widgetDefinition` is a native JSON object, not a string. Replace every `{{token}}` — a value that is only a token becomes a typed literal; a token inside text is interpolated. Omit a block whose data you lack rather than passing empty arrays. The win-rate column chart stays full-width.

```json
{"renderer":{"componentOverrides":{"$":{"type":"mosaic","definition":"tile/widget","children":[{"definition":"tile/row","attributes":{"gap":"sm","align":"center","isWrapped":false},"children":[{"definition":"tile/icon","attributes":{"name":"trending-up","size":"xl","alt":""}},{"definition":"tile/text","attributes":{"text":"{{title}}","variant":"h1"}}]},{"definition":"tile/text","attributes":{"text":"{{subtitle}}","variant":"caption","color":"muted"}},{"definition":"tile/separator"},{"definition":"tile/row","attributes":{"gap":"lg","align":"center","isWrapped":false},"children":[{"definition":"tile/column","attributes":{"width":"auto"},"children":[{"definition":"tile/piechart","attributes":{"variant":"donut","caption":"{{pieCaption}}","valueFormat":"number","showLegend":false,"slices":"{{pieSlices}}"}}]},{"definition":"tile/column","attributes":{"gap":"sm","align":"start","width":"stretch"},"children":[{"definition":"tile/text","attributes":{"text":"{{donutAsideTitle}}","variant":"section-title"}},{"definition":"tile/text","attributes":{"text":"{{donutAside}}","variant":"body","color":"muted"}}]}]},{"definition":"tile/text","attributes":{"text":"{{chartAsideTitle}}","variant":"section-title"}},{"definition":"tile/text","attributes":{"text":"{{chartAside}}","variant":"body","color":"muted"}},{"definition":"tile/chart","attributes":{"chartType":"column","caption":"{{chartCaption}}","categories":"{{chartCategories}}","series":"{{chartSeries}}","valueFormat":"number","showValues":true,"showLegend":false}},{"definition":"tile/datagrid","attributes":{"caption":"{{datagridCaption}}","appearance":"striped","size":"sm","defaultSort":{"key":"amount","direction":"desc"},"columns":[{"key":"status","header":"Status","type":"badge"},{"key":"name","header":"Opportunity","type":"text"},{"key":"amount","header":"Amount","type":"currency","align":"right","sortable":true},{"key":"stage","header":"Died at","type":"badge"},{"key":"reason","header":"Reason","type":"text"},{"key":"comp","header":"Lost to","type":"text"}],"rows":"{{lossRows}}"}},{"definition":"tile/callout","attributes":{"variant":"info","title":"{{calloutTitle}}","description":"{{calloutDescription}}"},"children":[{"definition":"tile/row","attributes":{"gap":"sm","align":"center","isWrapped":true},"children":[{"definition":"tile/button","attributes":{"label":"{{button1Label}}","variant":"primary","actions":{"click":[{"definition":"action/sendMessage","attributes":{"content":"{{button1Content}}"}}]}}},{"definition":"tile/button","attributes":{"label":"View in Salesforce","iconName":"open-in-new","variant":"secondary","actions":{"click":[{"definition":"action/openLink","attributes":{"url":"{{reportUrl}}"}}]}}}]}]}]}}}}
```

Tokens:
- `title` ("Win / loss review — Q2 2026"), `subtitle` (closed count, win rate, won/lost $).
- `pieCaption`, `pieSlices` `[{label, value, role?}]` — loss reasons, top one `role:"highlight"`. Legend is off, so `donutAsideTitle` + `donutAside` must name every slice with its count in prose.
- `chartAsideTitle`, `chartAside` (one-sentence interpretation), `chartCaption`, `chartCategories` (size bands), `chartSeries` `[{name:"Win rate", data:[percentages]}]`.
- `datagridCaption`, `lossRows` `[{status:{value:"Lost",badgeVariant:"error"}, name (plain opp name string), amount (number), stage:{value,badgeVariant}, reason (Reason Lost or "Not recorded"), comp ("—" unless Reason Lost names a competitor)}]`, largest first.
- `calloutTitle`, `calloutDescription`, `button1Label`/`button1Content` (first-person follow-up prompt), `reportUrl` (a Salesforce Lightning URL for the scope, e.g. the Opportunities list view).

```
display_widget({ resourceType: "dynamic", widgetDefinition: <hydrated object> })
```

## 4. Text — FALLBACK ONLY (if `display_widget` is unavailable or errors)

```
# Win-Loss Review — <Scope> — <Period>
**Headline:** <the pattern that matters most, 2 sentences>
## Win Rate
| Cut | Win Rate | n |
## Where Losses Die
| Stage | % of losses | Median days in stage |
## Loss Reasons
| Reason | Count | Example |
## What Wins Have In Common
- <pattern> (<N of M> wins)
## Largest Losses
- **<Account>** $<X> — died at <Stage> — <Reason Lost>
## Recommended Actions
1. Systemic  2. Coaching (rep, topic)  3. Data (e.g. Reason Lost capture rate)
```
