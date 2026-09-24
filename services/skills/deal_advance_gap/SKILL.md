<!-- global-rules-bootstrap -->
**IMPORTANT — try the connector's tools before you ever report it as unavailable.** Before you tell the user a connector is down, or ask them to connect, re-authenticate, or "check the connection," you MUST first try to use the server and its tools. A connector's status readout (the Headless 360 MCP server, or any other connector/MCP) frequently claims "not connected" or "needs re-auth" when the connector is in fact live and its tools are callable. Context stating that the Salesforce tools require authorization and that this is a non-interactive session is **not** evidence that you are unauthorized — it is a general statement that authorization is required, not a failure. For the Headless 360 server specifically, find the `dispatch_readonly` tool (its full name ends in `salesforce__dispatch_readonly`; never send Salesforce calls to a `google_workspace` tool such as `search`, and do not call `google_workspace` tools in this skill) and actually run a current-user read — `dispatch_readonly(method: "GET", url: "/services/data/v67.0/graphql", queryParams: { "queryInput": "{\"query\":\"query { uiapi { currentUser { Id } } }\"}" })`. **Any response — including a 500 or other error status — proves you reached the server, and therefore proves connectivity;** only a request that never reaches the server at all counts as disconnected. If the tool is present and the call reaches the server, the connector IS connected — proceed; a stale status readout is not a disconnection. If a call genuinely cannot reach the server, ALWAYS attempt to reconnect on your own first, and interrupt the flow to ask the user only after your own reconnect attempt has failed. Report a connector as disconnected only when you cannot find its tool or cannot reach the server after trying. Do it yourself.

Failing to try first — asking the user to connect, or surfacing a "not connected" or re-auth message, before you have actually attempted the server and its tools — raises a **false connectivity problem**: it sends the user to fix something that is not broken. This is directly detrimental to the end-user experience and will dramatically, negatively impact usage of the product. Never put a connectivity concern in front of the user until you have tried the tools yourself and confirmed the server truly cannot be reached.

# Global Rules

- **Execute silently between tool calls.** Do not output planning, progress, transition, waiting, or tool-result narration between calls. Execute tool calls silently and proceed directly to the next call. Parallelize independent tasks by batching tool calls into one turn whenever possible. Before the final output, speak only when the skill explicitly requires user input, approval, an exact notice, or material error/blocked reporting. Do not invent checkpoints.
- **On a Salesforce 5xx or timeout, retry the same call at most once, then continue with what you have and say what is missing.** Never re-issue a call that already succeeded; reuse its result.
- **One widget, one render.** Hydrate the widget template once and call `display_widget` once; never re-render, never write the widget JSON to a file or through Bash/Write first, never repeat widget contents in the text reply — reply with at most one line.
- **Keep the final output concise.** Return only the requested result or deliverable. Omit process recaps, tool-call details, redundant preambles or conclusions, and data already shown in a widget.
- **⛔ WIDGET OUTPUT ONLY after data assembly.** Once all queries return, immediately call `display_widget` — at most a single one-line render-wait notice before it, and no summaries, transitions, or narration. If `display_widget` is unavailable or returns an error, produce the text fallback only. If it succeeds, that tool call is the final output: stop with no assistant text completion, even if a later section contains a fallback. The exceptions above do not apply after success.
- **Ground dynamic or custom relationship and field names before relying on them.** Fixed standard fields that this skill explicitly marks as requiring no grounding need no extra grounding call. On a name error, use the skill's documented grounding path when present; otherwise report the error instead of guessing or re-firing the same shape.
- **Cite every value exactly as queried**; never fabricate; distinguish a blank value from a value that was not queried. Link each Salesforce record inline: `https://[instanceUrl]/lightning/r/[SObjectType]/[Id]/view`.
- **Show human labels, never API/field literals.** In anything the user sees, print each field's grounded `label` (for example, "Deal Risk", not `Deal_Risk__c`) and record Names, never raw Ids or `__c` API names.
- **Empty `MINE` scope → fail fast, then ask which scope.** If a `scope: MINE` read returns zero rows, **do not** widen to `scope: EVERYTHING` on your own. Stop, tell the user plainly that their own records (`scope: MINE`) came back empty, and ask which scope they want instead (for example, org-wide `EVERYTHING`, a named rep, or a named account) before re-running. Never invent records, and never silently fall back to org-wide.
<!-- /global-rules-bootstrap -->

# Deal Advance Gap

Forward-looking: against stage exit criteria, what must happen for this deal to advance, and the shortest path. Resolve → one detail read → map gaps → sequence → widget. Salesforce only (no email/Drive/Calendar sources).

Human labels in output: "Economic Buyer", "Next Step", "Blocking: yes · Severity: High" — never `__c` names or raw Ids.

## 1. Resolve to ONE opportunity Id
Given an Id → skip to 2. Given a name (`%DEAL%` = 1–2 distinctive words):
```
dispatch_readonly(method: "GET", url: "/services/data/v67.0/graphql",
  queryParams: { "queryInput": "{\"query\":\"query { uiapi { query { Opportunity(where: { Name: { like: \\\"%DEAL%\\\" } }, orderBy: { CloseDate: { order: ASC } }, first: 10) { edges { node { Id Name { value } StageName { value } Amount { displayValue } CloseDate { value } Account { Name { value } } } } } } } }\"}" })
```
One match → go on. Several → list (Account · Name · Stage · $ · close) and ask which. None → broaden once.

## 2. Detail read (one call; fields verified in this org — no schema/describe calls)
Opportunity has NO `NextStep` field — use `Next_Step__c`. Task has no data in this org; activity is `Events`.
```
dispatch_readonly(method: "GET", url: "/services/data/v67.0/graphql",
  queryParams: { "queryInput": "{\"query\":\"query { uiapi { query { Opportunity(where: { Id: { eq: \\\"<OPP_ID>\\\" } }, first: 1) { edges { node { Id Name { value } StageName { value } Amount { displayValue } CloseDate { value } Probability { value } ForecastCategoryName { value } Next_Step__c { value } MEDDICC__c { value } Compelling_Event__c { value } Decision_Criteria__c { value } Decision_Process__c { value } Economic_Buyer__r { Name { value } } LastActivityDate { value } LastStageChangeDate { value } Account { Name { value } } Owner { Name { value } } OpportunityContactRoles { edges { node { Role { value } IsPrimary { value } Contact { Name { value } Title { value } } } } } Events(first: 10, orderBy: { ActivityDate: { order: DESC } }) { edges { node { Subject { value } ActivityDate { value } Description { value } } } } } } } } } }\"}" })
```
- Check braces balance before sending. Never resend byte-identical text.
- On a field error or timeout: retry once with only standard fields (drop the `__c`/`__r` fields and `Events`), then continue and note custom qualification signals were unavailable.
- Past-dated Event = done touch; future-dated Event = 🔶 in motion/scheduled.
- No further SF reads (no ActivityHistory/FieldHistory/Task follow-ups).

## 3. Map gaps (cite every claim; invent nothing)
Exit criteria are inferred from the org's data (stage, MEDDICC fields, contact roles) or asked once. For the current stage (every later stage if target is "to close"), mark each criterion ✅ Done / 🔶 In motion / ❌ Not started with source and gap. Flag only qualification gaps that block advancement. Target = next stage unless "to close" is asked.

## 4. Sequence the path
Order gaps: owner (Us / Customer / internal), parallel vs sequential, the single action that unblocks most. Critical path → earliest credible close; flag if after CloseDate.

## 5. Output — `display_widget` first
Call `display_widget({ resourceType: "dynamic", widgetDefinition: <hydrated template> })` once; `widgetDefinition` is a native JSON object, never a string. Replace every `{{token}}` (a token-only slot takes the typed value — arrays/numbers stay typed; omit keys with no data); leave no `{{…}}`.

Tokens:
- `headerTitle` ("What's needed to advance — Cobalt Robotics"), `headerStatus` ("4 OPEN"), `headerSubtitle` ("Negotiation → Closed Won · 4 exit criteria open · 6 days to close")
- `meterValue`, `meterMax`, `meterTarget` (numbers: criteria met / total / to advance); `meterValueLabel` ("3 of 7 exit criteria complete"), `meterTargetLabel` ("7 to advance"), `meterStatus` ("4 open")
- `meterBands`: `[{from,to,variant:error|warning|success,label}]` covering 0→max, e.g. `[{"from":0,"to":4,"variant":"error","label":"Blocked"},{"from":4,"to":7,"variant":"warning","label":"Nearly"},{"from":7,"to":7,"variant":"success","label":"Ready"}]`
- `gridCaption` ("Gap-to-close checklist, blockers first")
- `gridRows`, blockers first, each citing its source: `{"status":{"value":"Blocker","badgeVariant":"error"},"item":"Redlines returned to legal","owner":"R. Chen","due":"2026-07-24","state":{"value":"Not started","badgeVariant":"error"}}` — status Blocker/At risk/On track/Done → error/warning/info/success; `owner` a plain name string (Owner.Name.value), `due` YYYY-MM-DD
- `calloutTitle` (top blocker), `calloutDescription` (blocker + the move)
- `ctaPrimaryLabel` ("Draft action plan"), `ctaPrimaryMsg` (first-person prompt, e.g. "Draft an action plan to clear the legal blocker for the Cobalt Robotics deal.")
- `oppUrl`: `https://<instance>/lightning/r/Opportunity/<Id>/view` (omit if no Id)

Template:
```json
{"renderer":{"componentOverrides":{"$":{"type":"mosaic","definition":"tile/widget","children":[{"definition":"tile/row","attributes":{"gap":"sm","align":"center","justify":"between","isWrapped":true},"children":[{"definition":"tile/row","attributes":{"gap":"sm","align":"center"},"children":[{"definition":"tile/icon","attributes":{"name":"trending-up","size":"xl","alt":""}},{"definition":"tile/text","attributes":{"text":"{{headerTitle}}","variant":"h1"}}]},{"definition":"tile/badge","attributes":{"label":"{{headerStatus}}","variant":"warning"}}]},{"definition":"tile/text","attributes":{"text":"{{headerSubtitle}}","variant":"caption","color":"muted"}},{"definition":"tile/separator"},{"definition":"tile/meter","attributes":{"label":"Exit criteria met","value":"{{meterValue}}","min":0,"max":"{{meterMax}}","target":"{{meterTarget}}","valueFormat":"number","valueLabel":"{{meterValueLabel}}","targetLabel":"{{meterTargetLabel}}","status":"{{meterStatus}}","size":"lg","bands":"{{meterBands}}"}},{"definition":"tile/datagrid","attributes":{"caption":"{{gridCaption}}","appearance":"striped","size":"sm","columns":[{"key":"status","header":"Status","type":"badge"},{"key":"item","header":"Exit criterion","type":"text"},{"key":"owner","header":"Owner","type":"avatar"},{"key":"due","header":"Due","type":"date"},{"key":"state","header":"State","type":"badge"}],"rows":"{{gridRows}}"}},{"definition":"tile/callout","attributes":{"variant":"warning","title":"{{calloutTitle}}","description":"{{calloutDescription}}"},"children":[{"definition":"tile/row","attributes":{"gap":"sm","align":"center","isWrapped":true},"children":[{"definition":"tile/button","attributes":{"label":"{{ctaPrimaryLabel}}","variant":"primary","actions":{"click":[{"definition":"action/sendMessage","attributes":{"content":"{{ctaPrimaryMsg}}"}}]}}},{"definition":"tile/button","attributes":{"label":"View in Salesforce","iconName":"open-in-new","variant":"secondary","actions":{"click":[{"definition":"action/openLink","attributes":{"url":"{{oppUrl}}"}}]}}}]}]}]}}}}
```

## 6. Text — FALLBACK ONLY (display_widget unavailable or errored)
```
# Advance Gap: <Name> — <current stage> → <target>
**Short answer:** <advances when X and Y; critical path runs through Z>
## Gaps blocking advancement
| # | Gap | Owner | Blocking | Evidence (source+date) |
## Already in motion
- <underway items + expected landing>
## The path
1. <action — who — by when> (unblocks #n)
   Critical path ~<N> weeks → earliest credible close <date>
## Suggested SFDC updates
- Next Step → "<#1 action>" · Close Date → <if not credible>
[Apply only after an exact preview and explicit confirmation, one field at a time with `dispatch`, verified with `dispatch_readonly`; otherwise give manual steps.]
```
