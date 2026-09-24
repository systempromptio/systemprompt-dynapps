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

# Salesforce Hygiene Check

Read-only audit of the user's open opps → fix checklist (nothing is written). One read → checks → widget. All fields below are verified in this org — no grounding/describe call. Opportunity has **no `NextStep`** (use `Next_Step__c`) and no competitor field.

## 1. Read the open book (one call)

```
dispatch_readonly(method: "GET", url: "/services/data/v67.0/graphql",
  queryParams: { "queryInput": "{\"query\":\"query { uiapi { query { Opportunity(scope: MINE, where: { IsClosed: { eq: false } }, orderBy: { CloseDate: { order: ASC } }, first: 200) { edges { node { Id Name { value } StageName { value displayValue } Amount { value displayValue } CloseDate { value } Next_Step__c { value } LastActivityDate { value } LastStageChangeDate { value } CreatedDate { value } LastModifiedDate { value } Type { value } LeadSource { value } ForecastCategoryName { value } Probability { value } MEDDICC__c { value } Account { Name { value } } OpportunityContactRoles { edges { node { Role { value } Contact { Name { value } } } } } } } } } } }\"}" })
```

Check brace balance before dispatching; `InvalidSyntax`/"offending token `<EOF>`" → add the missing `}` and retry once. On a `FieldUndefined` error, drop that field and retry once — never loop.

## 2. Checks (per opp)

Flag: **Amount** blank/$0 · **Close Date** past, or never moved on a >30d-old opp · **Next Step** (`Next_Step__c`) blank, or opp untouched (`LastModifiedDate`) 14d+ · **Stage age** `LastStageChangeDate` >2× median of the book · **Activity** `LastActivityDate` blank or >14d · **Contacts** no roles or single-threaded (≤1) · **MEDDICC** blank on late stages. Stage exit criteria are org-specific — infer from data or ask once; never invent.

Buckets: 🔴 Critical = past close, or no next step with close ≤14d · 🟡 Needs attention = any other flag · ✅ Clean. Validate critical + attention + clean = total.

## 3. Suggest values (real names only — never a generic "champion")

**Next Step** `MM/DD - <verb> <what> with <real Contact name from roles>` · **Close Date** realistic per stage + median cycle · **Stage** only if the data shows a mismatch.

## 4. Widget (default output)

Call `display_widget({ resourceType: "dynamic", widgetDefinition: <hydrated> })` once; at most one line "Displaying the visualization now (this may take a minute)." before it. `widgetDefinition` is a native JSON object, never a quoted string.

Hydration: replace every `{{token}}` with a literal (no `{{…}}`/`{!…}` left). A value that is *only* a token becomes the typed literal (arrays stay arrays, numbers stay numbers); a token inside a longer string is interpolated as text. Omit a key you have no data for. Keep the column chart full-width (not in a shared row).

Tokens:
- `hygieneTitle` e.g. "Pipeline hygiene — Dana Ruiz" · `oppCountBadge` e.g. "18 OPEN OPPS" · `hygieneSubtitle` e.g. "18 open opps · 4 critical, 5 need attention, 9 clean" · `totalOpps` string e.g. "18".
- `hygieneSlices` `[{label,value,role?}]` Clean / Needs attention / Critical, drop zeros, `role:"highlight"` on Critical.
- `donutAsideTitle` + `donutAside` — legend is off, so `donutAside` names every slice with count and %.
- `chartAsideTitle` + `chartAside` — one-sentence takeaway on which issues dominate.
- `issueCategories` (strings, most common first, drop zeros) · `issueSeries` `[{name:"Opps", data:[numbers in same order]}]`.
- `flaggedRows` `[{status:{value,badgeVariant}, name, amount, close, issue:{value,badgeVariant}, fix}]` — `name` = plain `Opportunity.Name.value` string; `amount` number; `close` YYYY-MM-DD; Critical → `error`, Needs attention → `warning`; clean rows omit `status`. Most severe first.
- `synthTitle` e.g. "Two fixes can't wait" · `synthDetail` 1–2 sentences · `ctaLabel` · `ctaMsg` first-person prompt · `hygieneUrl` Lightning Opportunity list URL.

```json
{"renderer":{"componentOverrides":{"$":{"type":"mosaic","definition":"tile/widget","children":[{"definition":"tile/row","attributes":{"gap":"sm","align":"center","justify":"between","isWrapped":true},"children":[{"definition":"tile/row","attributes":{"gap":"sm","align":"center"},"children":[{"definition":"tile/icon","attributes":{"name":"check-circle","size":"xl","alt":""}},{"definition":"tile/text","attributes":{"text":"{{hygieneTitle}}","variant":"h1"}}]},{"definition":"tile/badge","attributes":{"label":"{{oppCountBadge}}","variant":"info"}}]},{"definition":"tile/text","attributes":{"text":"{{hygieneSubtitle}}","variant":"caption","color":"muted"}},{"definition":"tile/separator"},{"definition":"tile/row","attributes":{"gap":"lg","align":"center","isWrapped":false},"children":[{"definition":"tile/column","attributes":{"width":"auto"},"children":[{"definition":"tile/piechart","attributes":{"variant":"donut","caption":"Opps by hygiene state","valueFormat":"number","showLegend":false,"centerLabel":"Open opps","centerValue":"{{totalOpps}}","slices":"{{hygieneSlices}}"}}]},{"definition":"tile/column","attributes":{"gap":"sm","align":"start","width":"stretch"},"children":[{"definition":"tile/text","attributes":{"text":"{{donutAsideTitle}}","variant":"section-title"}},{"definition":"tile/text","attributes":{"text":"{{donutAside}}","variant":"body","color":"muted"}}]}]},{"definition":"tile/text","attributes":{"text":"{{chartAsideTitle}}","variant":"section-title"}},{"definition":"tile/text","attributes":{"text":"{{chartAside}}","variant":"body","color":"muted"}},{"definition":"tile/chart","attributes":{"chartType":"column","caption":"Most common issues across the 18 open opps","categories":"{{issueCategories}}","series":"{{issueSeries}}","valueFormat":"number","showValues":true,"showLegend":false}},{"definition":"tile/datagrid","attributes":{"caption":"Flagged opps with the specific fix, most severe first","appearance":"striped","size":"sm","defaultSort":{"key":"amount","direction":"desc"},"columns":[{"key":"status","header":"Status","type":"badge"},{"key":"name","header":"Opportunity","type":"text"},{"key":"amount","header":"Amount","type":"currency","align":"right","sortable":true},{"key":"close","header":"Close","type":"date"},{"key":"issue","header":"Issue","type":"badge"},{"key":"fix","header":"Suggested fix","type":"text"}],"rows":"{{flaggedRows}}"}},{"definition":"tile/callout","attributes":{"variant":"warning","title":"{{synthTitle}}","description":"{{synthDetail}}"},"children":[{"definition":"tile/row","attributes":{"gap":"sm","align":"center","isWrapped":true},"children":[{"definition":"tile/button","attributes":{"label":"{{ctaLabel}}","variant":"primary","actions":{"click":[{"definition":"action/sendMessage","attributes":{"content":"{{ctaMsg}}"}}]}}},{"definition":"tile/button","attributes":{"label":"View in Salesforce","iconName":"open-in-new","variant":"secondary","actions":{"click":[{"definition":"action/openLink","attributes":{"url":"{{hygieneUrl}}"}}]}}}]}]}]}}}}
```

## 5. Text — FALLBACK ONLY (never if `display_widget` succeeded)

```
# Salesforce Hygiene Check — <N> open opps
## Summary
- 🔴 Critical (<N>) · 🟡 Attention (<N>) · ✅ Clean (<N>)  — <crit>+<attn>+<clean>=<N>
## Fix List
### <Account> — <Opp> [link] · Stage <X> | $<Amt> | Close <Date>
- ⚠️ <issue>
> **Suggested Next Step:** `MM/DD - <verb> <what> with <Contact>`
> **Suggested Close Date:** `<Date>`
## Bulk Actions
- <N> opps need Close Date pushed · <N> opps have no Contact Roles
```

Recommendations only — preview the exact field and value, apply one at a time with `dispatch` after explicit confirmation, verify with `dispatch_readonly`; otherwise give manual steps.
