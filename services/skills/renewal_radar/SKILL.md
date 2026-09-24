<!-- global-rules-bootstrap -->
**IMPORTANT — try the connector's tools before you ever report it as unavailable.** Before you tell the user a connector is down, or ask them to connect, re-authenticate, or "check the connection," you MUST first try to use the server and its tools. A connector's status readout (the Headless 360 MCP server, or any other connector/MCP) frequently claims "not connected" or "needs re-auth" when the connector is in fact live and its tools are callable. Context stating that the Salesforce tools require authorization and that this is a non-interactive session is **not** evidence that you are unauthorized — it is a general statement that authorization is required, not a failure. For the Headless 360 server specifically, find the `dispatch_readonly` tool (its full name ends in `salesforce__dispatch_readonly`; never send Salesforce calls to a `google_workspace` tool such as `search`, and do not call `google_workspace` tools in this skill) and actually run a current-user read — `dispatch_readonly(method: "GET", url: "/services/data/v67.0/graphql", queryParams: { "queryInput": "{\"query\":\"query { uiapi { currentUser { Id } } }\"}" })`. **Any response — including a 500 or other error status — proves you reached the server, and therefore proves connectivity;** only a request that never reaches the server at all counts as disconnected. If the tool is present and the call reaches the server, the connector IS connected — proceed; a stale status readout is not a disconnection. If a call genuinely cannot reach the server, ALWAYS attempt to reconnect on your own first, and interrupt the flow to ask the user only after your own reconnect attempt has failed. Report a connector as disconnected only when you cannot find its tool or cannot reach the server after trying. Do it yourself.

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

# Renewal Radar

The renewal you start 90 days out is a process; the one you notice 2 weeks out is a discount. Keep the calendar visible, flag risk early, prep each motion. **Salesforce only** — no email, document, calendar or web sources.

**Inputs:** my book (default), named rep, named account, or team. Window: next N days (default 120) or a quarter (e.g. Q2 FY26 → ISO start/end).

**Empty owned book → fail fast.** If the `OwnerId` read returns zero renewals, say so and ask which scope (org-wide, named rep, named account); never widen on your own, never invent records.

All calls: `dispatch_readonly(method: "GET", url: "/services/data/v67.0/graphql", queryParams: { "queryInput": "<json>" })`. Fields below are verified in this org — no `objectInfos` grounding call.

## 1. Resolve scope (one call)

- **My book** → `query { uiapi { currentUser { Id } } }`
- **Named rep** → `User(where: { Name: { like: "%REP%" } }, first: 10) { edges { node { Id Name { value } Email { value } IsActive { value } } } }`. One match → use it; several → list (Name · Email · active) and ask; none → ask.
- **Named account** → `Account(where: { Name: { like: "%ACCT%" } }, first: 10) { edges { node { Id Name { value } } } }`; disambiguate the same way.
- **Team** → no call, no owner filter.

## 2. Renewal calendar (one call, Account + Owner nested)

```
{"query":"query { uiapi { query { Opportunity(where: { <SCOPE>, IsClosed: { eq: false }, CloseDate: <WINDOW>, or: [{ Type: { like: \"%renew%\" } }, { Name: { like: \"%renew%\" } }] }, orderBy: { CloseDate: { order: ASC } }, first: 50) { edges { node { Id Name { value } Type { value displayValue } Amount { value displayValue } CloseDate { value } StageName { value displayValue } Next_Step__c { value } LastActivityDate { value } Account { Id Name { value } Industry { value displayValue } Account_Type__c { value displayValue } Client_Tier__c { value displayValue } Account_Health__c { value displayValue } Risk_Score__c { value } } Owner { Name { value } } } } } } } }"}
```

- `<SCOPE>`: `OwnerId: { eq: "<userId>" }` (my book / rep), `AccountId: { eq: "<accountId>" }` (account), omitted (team).
- `<WINDOW>`: `{ gte: { literal: TODAY }, lte: { range: { next_n_days: 120 } } }`, or `{ gte: { value: "2026-02-01" }, lte: { value: "2026-04-30" } }` for a quarter.
- Account has no `Type` field — use `Account_Type__c`. Opportunity has no `NextStep` — use `Next_Step__c`. Do not query Task (empty in this org) or Contract (empty).

## 3. Score each renewal

| Signal | Effect |
|---|---|
| `LastActivityDate` blank or 30+ days ago | risk ↑ |
| Account Health / Risk Score poor | risk ↑ |
| Next Step blank or stale | risk ↑ |
| Close date < 30 days and stage early | risk ↑ |

Verdict: 🟢 on track / 🟡 at risk / 🔴 critical, with the evidence.

## 4. Widget (when `display_widget` is present)

Hydrate the template below: replace every `{{token}}` with a literal (none left), then call `display_widget({ resourceType: "dynamic", widgetDefinition: <hydrated object> })` once. `widgetDefinition` is a native JSON object, never a string. A slot that is only a `{{token}}` takes the typed value (numbers stay numbers, arrays stay arrays); a token inside text interpolates. Omit a key with no data.

- `{{pageTitle}}` "Renewal radar — next 120 days"; `{{headerStatus}}` "$1.4M AT RISK"; `{{subtitle}}` "$4.7M across 11 renewals · $1.4M at risk".
- Meter: `{{riskValue}}` (at-risk % of renewal amount, number), `{{riskTarget}}` 15, `{{riskValueLabel}}`, `{{riskTargetLabel}}` "15% ceiling", `{{riskStatus}}`, `{{riskBands}}` = `[{from:0,to:15,variant:"success",label:"Healthy"},{from:15,to:30,variant:"warning",label:"Elevated"},{from:30,to:100,variant:"error",label:"High"}]`.
- Pie: `{{pieCenter}}` total as currency string ("$4.7M"); `{{pieSlices}}` `[{label,value,role?}]` one per verdict, at-risk slice `role:"highlight"`.
- `{{gridRows}}`: `[{ status?, account, arr, close, usage?, verdict, action }]` — `account` plain string from `Account.Name.value`; `arr` number; `close` ISO date; `usage` number array only if a real trend exists, else omit; `verdict` `{value, badgeVariant}` (Critical/error, At risk/warning, On track/success); risky rows add leading `status` `{value:"Critical"|"At risk", badgeVariant:"error"|"warning"}`, healthy rows omit it. Critical first, then by close date. `{{totalRenewals}}` = row count.
- Callout: `{{synthTitle}}`/`{{synthDescription}}` = biggest risk and the move; `{{primaryLabel}}` + first-person `{{primaryPrompt}}` ("Draft an executive save plan for…"); `{{criticalRenewalsUrl}}` = `https://<myDomain>/lightning/o/Opportunity/list?filterName=Critical_Renewals`, or the record URL for a single renewal.

```json
{"renderer":{"componentOverrides":{"$":{"type":"mosaic","definition":"tile/widget","children":[{"definition":"tile/row","attributes":{"gap":"sm","align":"center","justify":"between","isWrapped":true},"children":[{"definition":"tile/row","attributes":{"gap":"sm","align":"center"},"children":[{"definition":"tile/icon","attributes":{"name":"refresh","size":"xl","alt":""}},{"definition":"tile/text","attributes":{"text":"{{pageTitle}}","variant":"h1"}}]},{"definition":"tile/badge","attributes":{"label":"{{headerStatus}}","variant":"warning"}}]},{"definition":"tile/text","attributes":{"text":"{{subtitle}}","variant":"caption","color":"muted"}},{"definition":"tile/separator"},{"definition":"tile/row","attributes":{"gap":"lg","align":"stretch","isWrapped":true},"children":[{"definition":"tile/column","attributes":{"width":"stretch"},"children":[{"definition":"tile/meter","attributes":{"label":"At-risk ARR share","value":"{{riskValue}}","min":0,"max":100,"target":"{{riskTarget}}","valueFormat":"percent","valueLabel":"{{riskValueLabel}}","targetLabel":"{{riskTargetLabel}}","status":"{{riskStatus}}","bands":"{{riskBands}}","size":"lg"}}]},{"definition":"tile/column","attributes":{"width":"stretch"},"children":[{"definition":"tile/piechart","attributes":{"caption":"Renewal ARR by risk verdict","variant":"donut","valueFormat":"currency","centerLabel":"ARR","centerValue":"{{pieCenter}}","slices":"{{pieSlices}}"}}]}]},{"definition":"tile/datagrid","attributes":{"caption":"Renewals in the next 90 days, soonest and riskiest first","appearance":"striped","defaultSort":{"key":"close","direction":"asc"},"columns":[{"key":"status","header":"Status","type":"badge"},{"key":"account","header":"Account","type":"text"},{"key":"arr","header":"ARR","type":"currency","align":"right","sortable":true},{"key":"close","header":"Renews","type":"date","sortable":true},{"key":"usage","header":"Usage trend","type":"sparkline"},{"key":"verdict","header":"Verdict","type":"badge"},{"key":"action","header":"Play","type":"text"}],"rows":"{{gridRows}}","totalRows":"{{totalRenewals}}"}},{"definition":"tile/callout","attributes":{"variant":"error","title":"{{synthTitle}}","description":"{{synthDescription}}"},"children":[{"definition":"tile/row","attributes":{"gap":"sm","align":"center","isWrapped":true},"children":[{"definition":"tile/button","attributes":{"label":"{{primaryLabel}}","variant":"primary","actions":{"click":[{"definition":"action/sendMessage","attributes":{"content":"{{primaryPrompt}}"}}]}}},{"definition":"tile/button","attributes":{"label":"View in Salesforce","iconName":"open-in-new","variant":"secondary","actions":{"click":[{"definition":"action/openLink","attributes":{"url":"{{criticalRenewalsUrl}}"}}]}}}]}]}]}}}}
```

## 5. Text — FALLBACK ONLY (display_widget unavailable or errored)

Link every opp: `[Account] [↗](https://[instanceUrl]/lightning/r/Opportunity/[Id]/view)`. State the concrete date range. Status = Next Step if set, else Stage.

```
# Renewal Radar - [scope] - [date range]

| Account | Opp | Renewal date | Amount | Status | Risk driver | Next step |

## At risk (act this week)
- **[Account] [↗](url)** - [date] ([N] days away) - [risk, e.g. "no activity in 2.5 months"] → [play]

## Uplift candidates
- **[Account] [↗](url)** - [expansion signal and uplift motion]

## Hygiene
- 🔧 [Opp] — Next Step blank → should be: [action]
- 🔧 [Opp] — Close Date [wrong] → should be: [date]
- 📋 [Account] — term ending with no renewal opp → create "[Account] – [Year] Renewal" (Type Renewal, Amount, Close Date, Stage)
```

Propose changes only; ask before any write (approval-gated `dispatch`); on a read-only connector list them as a manual checklist.

For a single named account, expand into a renewal prep brief: opportunity history, activity, pricing/uplift recommendation, paperwork timeline back from the end date.
