<!-- global-rules-bootstrap -->
**Try the connector before ever reporting it unavailable.** Status readouts (Headless 360 or any MCP) often claim "not connected"/"needs re-auth" while tools are live; a note that tools "require authorization" in a non-interactive session is not a failure. First run `dispatch_readonly(method: "GET", url: "/services/data/v67.0/graphql", queryParams: { "queryInput": "{\"query\":\"query { uiapi { currentUser { Id } } }\"}" })`. **Any response, even a 500, proves connectivity** — proceed. Only if a call never reaches the server, attempt to reconnect yourself; ask the user only after that fails. Raising a false connectivity problem sends the user to fix something that isn't broken and badly hurts the product.

# Global Rules

- **Execute silently between tool calls**; batch independent calls in one turn. Speak before the final output only for required input/approval or a material error.
- **Concise final output** — the deliverable only; no process recap or repeating widget data.
- **⛔ Widget output only after data assembly.** Call `display_widget` immediately (at most one one-line render-wait notice). If it succeeds, that call is the final output — no further text. If unavailable/errors, give the text fallback only.
- **Field names:** use only the fields in this skill (verified on this org). On a name error, report it; don't guess or re-fire the same shape.
- **Cite every value as queried**; never fabricate; distinguish blank from not-queried. Link records: `https://[instanceUrl]/lightning/r/[SObjectType]/[Id]/view`.
- **Human labels only** ("Account Health", not `Account_Health__c`); show record Names, never raw Ids.
- **Empty `MINE` scope → stop and ask** which scope (org-wide, a rep, an account); never widen silently or invent records.
<!-- /global-rules-bootstrap -->
# Customer Health

Two jobs, one data pass: the ongoing "are we okay here?" check, and QBR prep that proves value and sets up the next phase. Read (one call) → score → output. Resolve to ONE account; if the name is ambiguous, ask the user to pick.

**Inputs:** account (name or Id) · mode: health check (default) or QBR prep · period: last quarter (default).

## 1. Read — one call

No grounding call: every field below is verified. `%ACCOUNT%` = the user's match (1–2 words); for an Id use `Id: { eq: \"...\" }`. Treat the template as literal — replace only `%ACCOUNT%`/the Id filter; never reshape or drop closing delimiters. Activity lives in **Event** (Task has no records in this org — read it, never depend on it).

```
dispatch_readonly(method: "GET", url: "/services/data/v67.0/graphql",
  queryParams: { "queryInput": "{\"query\":\"query { uiapi { query { Account(where: { Name: { like: \\\"%ACCOUNT%\\\" } }, first: 1) { edges { node { Id Name { value } Account_Type__c { value displayValue } Client_Tier__c { value displayValue } Segment__c { value displayValue } Account_Status__c { value displayValue } Account_Health__c { value displayValue } Account_Health_Indicator__c { value displayValue } Relationship_Health_Reason__c { value } Financial_Health__c { value displayValue } Risk_Score__c { value displayValue } CSAT_Score_open_text_field__c { value } Last_Business_Review_open_text_field__c { value } Last_C_Level_Meeting_open_text_field__c { value } Industry { value displayValue } AnnualRevenue { value displayValue } LastActivityDate { value } CreatedDate { value } Owner { Name { value } } Opportunities(first: 20, orderBy: { CloseDate: { order: DESC } }) { edges { node { Id Name { value } Type { value displayValue } StageName { value displayValue } Amount { value displayValue } CloseDate { value } IsClosed { value } IsWon { value } Next_Step__c { value } } } } Cases(first: 10, where: { CreatedDate: { gte: { literal: LAST_90_DAYS } } }, orderBy: { CreatedDate: { order: DESC } }) { edges { node { Id CaseNumber { value } Subject { value } Status { value } Priority { value } IsClosed { value } CreatedDate { value } } } } Events(first: 50, where: { ActivityDate: { gte: { literal: LAST_90_DAYS } } }, orderBy: { ActivityDate: { order: DESC } }) { edges { node { Subject { value } ActivityDate { value } Type { value displayValue } Who { Name { value } } } } } Tasks(first: 20, where: { ActivityDate: { gte: { literal: LAST_90_DAYS } } }, orderBy: { ActivityDate: { order: DESC } }) { edges { node { Subject { value } ActivityDate { value } } } } } } } } } }\"}" })
```

Empty `Account` edges → broaden `%ACCOUNT%` once or ask. Empty `Cases`/`Events`/`Tasks` = none in the window, not an error — don't retry.

## 2. Score (cite every value; invent nothing)

| Dimension | Signal | Status |
|---|---|---|
| Relationship | Event attendees (`Who`), breadth of contacts met, Last C-Level Meeting, Relationship Health Reason | 🟢🟡🔴 |
| Engagement trend | Event (+Task) cadence last 90d, bucketed by week; LastActivityDate | |
| Commercial | renewal proximity (open Renewal-type opp CloseDate), open expansion, Financial Health / Risk Score | |
| Support | open Cases, aging/priority | |
| Value delivery | Last Business Review, CSAT, Account Health fields | |

Overall verdict with the one or two dimensions driving it.

## 3. Output — widget first

Call `display_widget({ resourceType: "dynamic", widgetDefinition: <hydrated> })` once. `widgetDefinition` must be a native JSON object, never a quoted string. Substitute each `{{token}}`: a slot that is only a token takes the typed value (arrays stay arrays, numbers stay numbers); a token inside a string interpolates as text; omit a key you have no data for so that leaf drops. Leave no `{{…}}`.

Tokens:
- `headerTitle` str "Account health — <Account>"; `headerStatus` str label ("AT RISK"/"HEALTHY"/"NEEDS ATTENTION"; renderer maps color); `headerSubtitle` str "ARR · days to renewal · health + trend".
- `chartCaption` str; `chartCategories` string[6] week labels; `chartSeries` [{name, data: number[6]}] — weekly Event cadence; omit the chart if no cadence data.
- `meterValue` 0–100 composite score; `meterTarget` num (e.g. 75); `meterValueLabel` "62 / 100"; `meterTargetLabel` "75 healthy"; `meterStatus` short read; `meterBands` [{from,to,variant: error|warning|success,label}] covering 0→100.
- `gridCaption` str; `gridRows` one per dimension, worst first: {status?: {value, badgeVariant: neutral|primary|secondary|outline|success|info|warning|error}, signal, now (formatted string, e.g. "3 / 90d"), trend: number[6], note}. Empty → grid omitted.
- `calloutTitle` the single most important read; `calloutDescription` the read + recommended move.
- `ctaPrimaryLabel` button label; `ctaPrimaryMsg` first-person prompt it sends; `accountUrl` Account Lightning URL.

```json
{"renderer":{"componentOverrides":{"$":{"type":"mosaic","definition":"tile/widget","children":[{"definition":"tile/row","attributes":{"gap":"sm","align":"center","justify":"between","isWrapped":true},"children":[{"definition":"tile/row","attributes":{"gap":"sm","align":"center"},"children":[{"definition":"tile/icon","attributes":{"name":"activity","size":"xl","alt":""}},{"definition":"tile/text","attributes":{"text":"{{headerTitle}}","variant":"h1"}}]},{"definition":"tile/badge","attributes":{"label":"{{headerStatus}}","variant":"error"}}]},{"definition":"tile/text","attributes":{"text":"{{headerSubtitle}}","variant":"caption","color":"muted"}},{"definition":"tile/separator"},{"definition":"tile/column","attributes":{"gap":"md"},"children":[{"definition":"tile/chart","attributes":{"chartType":"line","caption":"{{chartCaption}}","categories":"{{chartCategories}}","series":"{{chartSeries}}","valueFormat":"number","showValues":false,"showLegend":false}},{"definition":"tile/meter","attributes":{"label":"Composite health","value":"{{meterValue}}","min":0,"max":100,"target":"{{meterTarget}}","valueFormat":"number","valueLabel":"{{meterValueLabel}}","targetLabel":"{{meterTargetLabel}}","status":"{{meterStatus}}","size":"lg","bands":"{{meterBands}}"}}]},{"definition":"tile/datagrid","attributes":{"caption":"{{gridCaption}}","appearance":"striped","columns":[{"key":"status","header":"Status","type":"badge"},{"key":"signal","header":"Signal","type":"text"},{"key":"now","header":"Current","type":"text","align":"right"},{"key":"trend","header":"6-wk trend","type":"sparkline"},{"key":"note","header":"Read","type":"text"}],"rows":"{{gridRows}}"}},{"definition":"tile/callout","attributes":{"variant":"error","title":"{{calloutTitle}}","description":"{{calloutDescription}}"},"children":[{"definition":"tile/row","attributes":{"gap":"sm","align":"center","isWrapped":true},"children":[{"definition":"tile/button","attributes":{"label":"{{ctaPrimaryLabel}}","variant":"primary","actions":{"click":[{"definition":"action/sendMessage","attributes":{"content":"{{ctaPrimaryMsg}}"}}]}}},{"definition":"tile/button","attributes":{"label":"View in Salesforce","iconName":"open-in-new","variant":"secondary","actions":{"click":[{"definition":"action/openLink","attributes":{"url":"{{accountUrl}}"}}]}}}]}]}]}}}}
```

## 4. Text — FALLBACK ONLY (never if `display_widget` succeeded)

```
# Health: <Account> — 🟢/🟡/🔴
<verdict sentence>
<dimension table with evidence>
## Watch items
- <signal, why it matters, suggested action>
## Suggested SFDC updates
- <Account health/status fields: before → after; confirm before writing, or a checklist on a read-only connector>
```

**QBR prep mode** adds: 1. **Value delivered** — outcomes since last review (CRM-visible, cited) · 2. **Adoption story** — working vs underused (CRM facts only) · 3. **Open items** — Cases resolved/open, prior commitments · 4. **Next phase** — expansion plays (`expansion-whitespace`) and renewal framing (`renewal-radar`) · 5. **Agenda + attendees** — who should be in the room (from Event attendees/contacts) and the asks. Offer the agenda as text; never create or send meetings.
