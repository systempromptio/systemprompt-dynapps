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
# Research Prospect

Salesforce only. **Web search and external research are out of scope** (blocked for this model) — do not call WebSearch/WebFetch or any Google Workspace tool. Say once in the output that company facts, news and funding signals were not researched externally.

Resolve Account by name → ONE combined read → score fit → widget. All fields below are verified for this org; no grounding/describe call.

## 1. Inputs

- **Company name or domain** (required); **contact name or title** (optional).
- ICP, value prop and competitors are not in Salesforce — infer only from org data or ask the user once. Never invent.

## 2. Read (one call; Website fallback only if Name finds nothing)

Match on `Name` first. Never combine Name and Website with `or` (500s). Account has **no `Type`** — use `Account_Type__c`.

```
dispatch_readonly(method: "GET", url: "/services/data/v67.0/graphql",
  queryParams: { "queryInput": "{\"query\":\"query { uiapi { query { Account(where: { Name: { like: \\\"%COMPANY%\\\" } }, first: 5) { edges { node { Id Name { value } Website { value } Industry { value displayValue } NumberOfEmployees { value } AnnualRevenue { value displayValue } Account_Type__c { value displayValue } Client_Tier__c { value displayValue } Segment__c { value displayValue } Market_segment__c { value displayValue } Verticle__c { value displayValue } Employee_Range__c { value displayValue } Territory__c { value displayValue } Account_Status__c { value displayValue } Account_Health__c { value displayValue } Current_Platform__c { value displayValue } Owner { Name { value } } Opportunities(orderBy: { CloseDate: { order: DESC } }, first: 10) { edges { node { Id Name { value } StageName { value displayValue } Amount { value displayValue } CloseDate { value } IsClosed { value } IsWon { value } Next_Step__c { value } } } } Contacts(first: 10) { edges { node { Id Name { value } Title { value } Email { value } } } } } } } lead: Lead(where: { Company: { like: \\\"%COMPANY%\\\" }, IsConverted: { eq: false } }, first: 5) { edges { node { Id Name { value } Title { value } Company { value } Status { value displayValue } Industry { value } NumberOfEmployees { value } Owner { ... on User { Name { value } } } } } } events: Event(where: { Account: { Name: { like: \\\"%COMPANY%\\\" } } }, orderBy: { ActivityDate: { order: DESC } }, first: 10) { edges { node { Id Subject { value } ActivityDate { value } Type { value } } } } } } }\"}" })
```

- **Account found** → output must say prominently "already in CRM, owned by [Name]" (plus open opps, past wins/losses, recent Events) so the user doesn't step on a colleague.
- **No Account** → rerun the Account root only with `Website: { like: \"%DOMAIN%\" }`. Still nothing → "net new — no Account"; report any open Lead rows (owner, status). Broaden `%…%` once at most; never re-fire the same shape.
- If `lead` or `events` errors, drop that root and retry once without it rather than abandoning the read.

## 3. ICP fit (from Salesforce values only)

Score Industry/Vertical, Size (employees/revenue/range), Buyer persona (contact titles), Relationship (tier, status, health, past wins), Timing (open opps, recent Events). Each ✅/⚠️/❌ with the queried evidence; a dimension with no data is "unknown", not ❌. Overall **Strong / Moderate / Poor** (0-100 score) + one-sentence rationale.

## 4. Hooks

2-3 specific hooks grounded in CRM facts (current platform, prior opp outcomes, recent meetings, contact roles). No generic claims, no external facts.

## 5. Widget

When data is assembled, call `display_widget` (one optional notice line: "Displaying the visualization now (this may take a minute)."). `widgetDefinition` is a native JSON object, not a string. Hydrate this template, replacing every `{{token}}`:

```json
{"renderer":{"componentOverrides":{"$":{"type":"mosaic","definition":"tile/widget","children":[{"definition":"tile/row","attributes":{"gap":"sm","align":"center","justify":"between","isWrapped":true},"children":[{"definition":"tile/row","attributes":{"gap":"sm","align":"center","isWrapped":false},"children":[{"definition":"tile/icon","attributes":{"name":"search","size":"xl","alt":""}},{"definition":"tile/text","attributes":{"text":"{{prospectTitle}}","variant":"h1"}}]},{"definition":"tile/badge","attributes":{"label":"{{fitStatus}}","variant":"success"}}]},{"definition":"tile/text","attributes":{"text":"{{prospectSubtitle}}","variant":"caption","color":"muted"}},{"definition":"tile/separator"},{"definition":"tile/meter","attributes":{"label":"ICP fit","value":"{{icpValue}}","min":0,"max":100,"target":"{{icpTarget}}","valueFormat":"number","valueLabel":"{{icpValueLabel}}","targetLabel":"{{icpTargetLabel}}","status":"{{icpStatus}}","bands":"{{icpBands}}"}},{"definition":"tile/datagrid","attributes":{"caption":"What we know — signals and openings","appearance":"striped","size":"sm","columns":[{"key":"status","header":"Status","type":"badge"},{"key":"topic","header":"Signal","type":"text"},{"key":"tag","header":"Read","type":"badge"},{"key":"note","header":"Detail","type":"text"}],"rows":"{{signalRows}}"}},{"definition":"tile/callout","attributes":{"variant":"recommended","title":"{{synthTitle}}","description":"{{synthDetail}}"},"children":[{"definition":"tile/row","attributes":{"gap":"sm","align":"center","isWrapped":true},"children":[{"definition":"tile/button","attributes":{"label":"{{ctaLabel}}","variant":"primary","actions":{"click":[{"definition":"action/sendMessage","attributes":{"content":"{{ctaMsg}}"}}]}}},{"definition":"tile/button","attributes":{"label":"View in Salesforce","iconName":"open-in-new","variant":"secondary","actions":{"click":[{"definition":"action/openLink","attributes":{"url":"{{prospectUrl}}"}}]}}}]}]}]}}}}
```

Tokens:
- `prospectTitle` ("Prospect brief — [Company]"), `fitStatus` ("STRONG FIT"…), `prospectSubtitle` (industry · employees · revenue · fit).
- Meter: `icpValue`, `icpTarget` (e.g. 70) — numbers 0-100; `icpValueLabel` ("88 / 100"), `icpTargetLabel` ("70 = qualified"), `icpStatus`; `icpBands` = `[{from:0,to:50,variant:"error",label:"Weak"},{from:50,to:70,variant:"warning",label:"Fair"},{from:70,to:100,variant:"success",label:"Strong"}]`.
- `signalRows`: array of `{status:{value:"Buy|Fit|Watch",badgeVariant:"success|info|warning"}, topic, tag:{value:"Opportunity|Signal|Obstacle",badgeVariant:"success|info|warning"}, note}` — one per CRM signal; empty → omit.
- Callout: `synthTitle` (single most important read), `synthDetail` (approach + timing); `ctaLabel`/`ctaMsg` (first-person prompt, e.g. "Draft outreach to …"); `prospectUrl` = `https://[instanceUrl]/lightning/r/Account/[Id]/view` — omit that button if no Account Id.

A slot that is only a `{{token}}` takes the typed value (numbers stay numbers, arrays stay arrays); a token inside a string interpolates as text. No `{{…}}`/`{!…}` may remain; quote blanks as blank, drop signals with no data.

```
display_widget({ resourceType: "dynamic", widgetDefinition: <hydrated> })
```

## 6. Text — fallback only if `display_widget` is unavailable or errors

```
# Prospect Research: [Company]
## CRM Status
[Already in Salesforce — owned by [Name], [N] open opps | Net new — no Account (Lead: …)]
## Snapshot (Salesforce)
Industry/vertical, employees, revenue, tier, segment, platform, territory
## Relationship
Opps (open/won/lost), contacts (name, title), recent Events
## ICP Fit: [Strong/Moderate/Poor]
[table] + rationale
## Hooks
1. …
## Suggested Next Step
[e.g. "Owned by [Name] — coordinate first" | "Run draft-outreach to [Contact]" | "Poor fit — disqualify"]
_External research (news, funding, leadership) not performed — out of scope._
