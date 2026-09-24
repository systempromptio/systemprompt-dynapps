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

# Close Plan

Resolve opp by name → read opp + activity (ONE turn, one call) → draft business case + mutual action plan (MAP) → widget.

## 1. Read opp + contacts + activity (single call, no grounding call)

`%DEALNAME%` = 1–2 distinctive words from the user's request. All fields below are verified in this org — use them as-is. Opportunity has **no** `NextStep` (use `Next_Step__c`) and no approval/competitor field. Do not span custom lookup `__c` fields (they return Ids, not Names). Task holds no data here; activity lives in Event — query both, never depend on Task.

```
dispatch_readonly(method: "GET", url: "/services/data/v67.0/graphql",
  queryParams: { "queryInput": "{\"query\":\"query { uiapi { query { Opportunity(where: { Name: { like: \\\"%DEALNAME%\\\" } }, first: 3, orderBy: { CloseDate: { order: DESC } }) { edges { node { Id Name { value } StageName { value } Amount { value displayValue } CloseDate { value } Probability { value } ForecastCategoryName { value } Type { value } LastStageChangeDate { value } Next_Step__c { value } MEDDICC__c { value } Description { value } Account { Name { value } } Owner { Name { value } } OpportunityContactRoles { edges { node { Role { value } IsPrimary { value } Contact { Name { value } Title { value } } } } } Events(first: 20, orderBy: { ActivityDateTime: { order: DESC } }) { edges { node { Subject { value } ActivityDateTime { value } Description { value } Owner { ... on User { Name { value } } } } } } Tasks(first: 10, orderBy: { ActivityDate: { order: DESC } }) { edges { node { Subject { value } ActivityDate { value } Status { value } Description { value } } } } } } } } } }\"}" })
```

Empty `edges` → retry once with a shorter `%…%`; still empty → say so and stop. Multiple matches → pick the open one closest to the request; name the others in one line. If `deal_advance_gap` ran this session, feed its gaps straight into the MAP.

## 2. Business case

Evidence = Opportunity Description / Next Step / MEDDICC, Event subjects + descriptions (meetings, calls), contact roles. Value prop, differentiators, required stakeholders and internal approval chain are external knowledge — infer only from this data or ask the user once; never invent. In the customer's language, citing the record/meeting each point comes from:

1. **Current state and its cost** — quantified where they gave numbers
2. **Desired outcome** — success metrics, why-now
3. **Proposed solution** — mapped to each outcome
4. **Investment and return** — price vs. quantified value, simple attributable math
5. **Risk of waiting** — in their terms
6. **Why us** — only differentiators they reacted to

Flag every claim without customer evidence as "to validate on the next call".

## 3. Mutual action plan

Work backward from the target signature date (default CloseDate): remaining validation, security review, legal redlines, procurement, signatures, plus internal approvals the user mentions. Row = step, owner (us / customer / named person), target date, status. Flag steps whose dates make CloseDate impossible.

## 4. Widget (default when `display_widget` exists)

Call `display_widget({ resourceType: "dynamic", widgetDefinition: <hydrated template> })` once; `widgetDefinition` is a native JSON object, never a string. Replace every `{{token}}`; none may remain. A leaf that is only a token takes the typed value (arrays stay arrays, numbers stay numbers); a token inside a string interpolates. No data for a key → drop that leaf; drop steps with no data. Layout: header → subhead → one meter → path datagrid → error callout with two buttons.

Tokens:
- `planTitle` "Mutual close plan — [Account]" · `headerStatus` e.g. "6 DAYS OUT" · `subtitle` e.g. "$1.8M · target close Jul 30 · 6 days out · 4 of 9 steps done"
- Meter (numbers): `progressValue` steps done, `progressMax` total, `progressTarget` (= max). Strings: `progressValueLabel` "4 of 9 steps complete", `progressTargetLabel` "9 to sign", `progressStatus` "on the critical path". `progressBands`: array of `{from,to,variant,label}` covering 0→max in order, variant warning/success, e.g. `[{"from":0,"to":5,"variant":"warning","label":"In progress"},{"from":5,"to":9,"variant":"success","label":"Closing"}]`
- `pathRows`: one per MAP step: `{"status":{"value":"Critical","badgeVariant":"error"},"step":"Send redlined MSA","side":{"value":"Us","badgeVariant":"primary"},"owner":"R. Chen","due":"2026-07-24","state":{"value":"Due today","badgeVariant":"error"}}`. status: Critical/error, At risk/warning, On track/info, Target/neutral (omit on normal rows). side: Us=primary, Them=secondary, Both=info. state: error=Due today/Critical, warning=In review/Waiting/At risk, info=Scheduled, secondary=Target/On track. `owner` is a plain display-name string (from `Contact.Name.value`/`Owner.Name.value`), never an object. `due` ISO date.
- Callout: `calloutTitle` headline blocker · `calloutDesc` cost if it slips · `ctaLabel` + `ctaMsg` (first-person prompt, e.g. "Draft a note to legal attaching the redlined MSA…") · `oppUrl` `https://<myDomain>/lightning/r/Opportunity/<Id>/view` (omit if no Id).

Template:
```json
{"renderer":{"componentOverrides":{"$":{"type":"mosaic","definition":"tile/widget","children":[{"definition":"tile/row","attributes":{"gap":"sm","align":"center","justify":"between","isWrapped":true},"children":[{"definition":"tile/row","attributes":{"gap":"sm","align":"center"},"children":[{"definition":"tile/icon","attributes":{"name":"check-circle","size":"xl","alt":""}},{"definition":"tile/text","attributes":{"text":"{{planTitle}}","variant":"h1"}}]},{"definition":"tile/badge","attributes":{"label":"{{headerStatus}}","variant":"warning"}}]},{"definition":"tile/text","attributes":{"text":"{{subtitle}}","variant":"caption","color":"muted"}},{"definition":"tile/separator"},{"definition":"tile/meter","attributes":{"label":"Close plan progress","value":"{{progressValue}}","min":0,"max":"{{progressMax}}","target":"{{progressTarget}}","valueFormat":"number","valueLabel":"{{progressValueLabel}}","targetLabel":"{{progressTargetLabel}}","status":"{{progressStatus}}","size":"lg","bands":"{{progressBands}}"}},{"definition":"tile/datagrid","attributes":{"caption":"Path to signature, by date","appearance":"striped","size":"sm","columns":[{"key":"status","header":"Status","type":"badge"},{"key":"step","header":"Step","type":"text"},{"key":"side","header":"Side","type":"badge"},{"key":"owner","header":"Owner","type":"avatar"},{"key":"due","header":"Due","type":"date"},{"key":"state","header":"State","type":"badge"}],"rows":"{{pathRows}}"}},{"definition":"tile/callout","attributes":{"variant":"error","title":"{{calloutTitle}}","description":"{{calloutDesc}}"},"children":[{"definition":"tile/row","attributes":{"gap":"sm","align":"center","isWrapped":true},"children":[{"definition":"tile/button","attributes":{"label":"{{ctaLabel}}","variant":"primary","actions":{"click":[{"definition":"action/sendMessage","attributes":{"content":"{{ctaMsg}}"}}]}}},{"definition":"tile/button","attributes":{"label":"View in Salesforce","iconName":"open-in-new","variant":"secondary","actions":{"click":[{"definition":"action/openLink","attributes":{"url":"{{oppUrl}}"}}]}}}]}]}]}}}}
```

## 5. Text — FALLBACK ONLY (never if `display_widget` succeeded)

```
# Close Plan: [Opp Name] - [Account]
## Business Case
Current State · Desired Outcome · Proposed Solution · Investment and Return · Risk of Waiting · Why Us
**Claims to validate:** [claims with no customer evidence]
## Mutual Action Plan
| Step | Owner | Target Date | Status |
**Date conflicts:** [steps that make CloseDate impossible]
## Suggested SFDC updates
- Next Step → "[next dated step]"
- Close Date → [if the backward plan says it is not credible]
[Only after an exact preview and explicit confirmation: apply one field at a time with `dispatch`, verify with `dispatch_readonly`; otherwise give manual steps.]
## Draft note to champion
[Shares the plan, asks them to confirm owners on their side]
```
