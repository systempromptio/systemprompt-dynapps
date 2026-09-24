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
# Goal
Summarize the closed activity (Events and Tasks) on ONE `Account`, `Contact`, `Lead` or `Opportunity`. Fast results, minimal thinking.

# Rules
- Only those four parents expose `ActivityHistories`; any other object → stop and say so.
- `ActivityHistory` is a child subquery only (not in UI-API GraphQL) → SOQL at `v67.0`. All fields below are standard; no grounding.
- In this org activity lives mostly in Events; Tasks may be empty. `ActivityHistories` returns both — never depend on Task alone.
- Show `ActivitySubtype` (Call/Email/Task/Event), not `ActivityType` (often blank).
- Empty history is a valid "no activity" result. Never fabricate or mock activities.

## 1. Resolve the record
Id with the right prefix (`001`/`003`/`00Q`/`006`) → use it. Else:
```
dispatch_readonly(method: "GET", url: "/services/data/v67.0/query",
  queryParams: { "q": "SELECT Id, Name FROM [objectName] WHERE Name LIKE '%[match]%' LIMIT 5" })
```
One match → use it. Several → list Name · Id and ask. None → say so; never invent.

## 2. Query activity history
```
dispatch_readonly(method: "GET", url: "/services/data/v67.0/query",
  queryParams: { "q": "SELECT Name, (SELECT ActivityDate, ActivitySubtype, Description, Subject, Owner.Name, Who.Name, StartDateTime, LastModifiedDate FROM ActivityHistories ORDER BY ActivityDate DESC NULLS LAST, LastModifiedDate DESC LIMIT 500) FROM [objectName] WHERE Id = '[recordId]'" })
```
On `"implementation restriction on ActivityHistories"` → retry with a smaller subquery `LIMIT 200`. Null/empty collection = no activity.

## 3. Render
`TotalCount = 0` → skip the widget; output only `No activity history found for [RecordName].`
Otherwise call `display_widget({ resourceType: "dynamic", widgetDefinition: <hydrated template> })` once, `widgetDefinition` as a native JSON object (never a quoted string). One optional notice before it: "Displaying the visualization now (this may take a minute)."

Tokens (a slot that is only `{{token}}` takes the typed value; omit keys you lack; leave no `{{…}}`):
- `pageTitle` "Activity history — <RecordName>"; `pageSubtitle` "window · N touches · last contact …"
- `heatmapCaption`; `heatmapDomain` number `[min,max]` e.g. `[0,4]`; `heatmapDays` `[{date:"YYYY-MM-DD", value:n}]` per day over ~12 weeks, from `ActivityDate`
- `calloutVariant` ("info"), `calloutTitle`, `calloutDescription` — the engagement read and the gap
- `primaryButtonLabel`, `primaryButtonMsg` — first-person next-step prompt
- `viewRecordUrl` `https://<myDomain>/lightning/r/<ObjectName>/<recordId>/view`
- `datagridCaption`; `datagridTotalRows` number = `ActivityHistories.totalSize`; `datagridRows` most recent ~5 `{date, type:{value: ActivitySubtype, badgeVariant}, who: Who.Name or "—", note: Subject}`

```json
{"renderer":{"componentOverrides":{"$":{"type":"mosaic","definition":"tile/widget","children":[{"definition":"tile/row","attributes":{"gap":"sm","align":"center","isWrapped":false},"children":[{"definition":"tile/icon","attributes":{"name":"activity","size":"xl","alt":""}},{"definition":"tile/text","attributes":{"text":"{{pageTitle}}","variant":"h1"}}]},{"definition":"tile/text","attributes":{"text":"{{pageSubtitle}}","variant":"caption","color":"muted"}},{"definition":"tile/separator"},{"definition":"tile/row","attributes":{"gap":"lg","align":"start","isWrapped":false},"children":[{"definition":"tile/column","attributes":{"width":"md"},"children":[{"definition":"tile/heatmap","attributes":{"layout":"calendar","caption":"{{heatmapCaption}}","encode":"color","scale":"sequential","valueFormat":"number","domain":"{{heatmapDomain}}","days":"{{heatmapDays}}"}}]},{"definition":"tile/column","attributes":{"width":"stretch"},"children":[{"definition":"tile/callout","attributes":{"variant":"{{calloutVariant}}","title":"{{calloutTitle}}","description":"{{calloutDescription}}"},"children":[{"definition":"tile/row","attributes":{"gap":"sm","align":"center","isWrapped":true},"children":[{"definition":"tile/button","attributes":{"label":"{{primaryButtonLabel}}","variant":"primary","actions":{"click":[{"definition":"action/sendMessage","attributes":{"content":"{{primaryButtonMsg}}"}}]}}},{"definition":"tile/button","attributes":{"label":"View in Salesforce","iconName":"open-in-new","variant":"secondary","actions":{"click":[{"definition":"action/openLink","attributes":{"url":"{{viewRecordUrl}}"}}]}}}]}]}]}]},{"definition":"tile/datagrid","attributes":{"caption":"{{datagridCaption}}","appearance":"striped","totalRows":"{{datagridTotalRows}}","columns":[{"key":"date","header":"Date","type":"date"},{"key":"type","header":"Type","type":"badge"},{"key":"who","header":"With","type":"text"},{"key":"note","header":"Summary","type":"text"}],"rows":"{{datagridRows}}"}}]}}}}
```

### Fallback (only if display_widget is unavailable or errors)
```markdown
# Activity History: [RecordName] ([ObjectName])
**[TotalCount] activities** | **Last:** [LastActivityDate]

| Date | Type | Subject | Owner | With |
|---|---|---|---|---|
```
Newest first, max 20 (then `…and [N] older activities not shown`); undated → "Undated"; blank `Who.Name` → "—"; non-empty `Description` as a sub-bullet truncated to 200 chars.
