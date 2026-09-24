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

# Weekly Wrap

Resolve scope → ONE combined Salesforce read → assemble → widget. No grounding call: every field below is verified for this org (`NextStep` does not exist — use `Next_Step__c`; loss reason is `ReasonLost__c`). Salesforce only: meetings come from `Event` (and `Task`, which may be empty in this org); no email or document sources.

## 1. Scope

- **"my week"** (default) → `scope: MINE` on every root, no lookup.
- **"team"** → one roster read, then replace every `scope: MINE` with `OwnerId: { in: [\"<ID>\", …] }`:
  ```
  dispatch_readonly(method: "GET", url: "/services/data/v67.0/graphql",
    queryParams: { "queryInput": "{\"query\":\"query { uiapi { query { User(where: { ManagerId: { eq: \\\"<CURRENT_USER_ID>\\\" }, IsActive: { eq: true } }, first: 50) { edges { node { Id Name { value } } } } } } }\"}" })
  ```
- **Named account/deal** → add `Name: { like: \"%<name>%\" }` (Opportunity) or `Account: { Name: { like: … } }` to the Opportunity roots.

## 2. Read — one dispatch

```
dispatch_readonly(method: "GET", url: "/services/data/v67.0/graphql",
  queryParams: { "queryInput": "{\"query\":\"query { uiapi { query { closedThisWeek: Opportunity(scope: MINE, where: { IsClosed: { eq: true }, CloseDate: { eq: { literal: THIS_WEEK } } }, first: 50) { edges { node { Id Name { value } StageName { value } IsWon { value } Amount { value } CloseDate { value } Next_Step__c { value } ReasonLost__c { value } Account { Name { value } } } } } newThisWeek: Opportunity(scope: MINE, where: { CreatedDate: { eq: { literal: THIS_WEEK } } }, first: 50) { edges { node { Id Name { value } StageName { value } Amount { value } Account { Name { value } } } } } touchedThisWeek: Opportunity(scope: MINE, where: { IsClosed: { eq: false }, LastModifiedDate: { eq: { literal: THIS_WEEK } } }, first: 50) { edges { node { Id Name { value } StageName { value } Amount { value } CloseDate { value } LastStageChangeDate { value } Next_Step__c { value } Account { Name { value } } } } } closingNextWeek: Opportunity(scope: MINE, where: { IsClosed: { eq: false }, CloseDate: { eq: { literal: NEXT_WEEK } } }, first: 50) { edges { node { Id Name { value } StageName { value } Amount { value } CloseDate { value } Account { Name { value } } } } } meetingsThisWeek: Event(scope: MINE, where: { ActivityDate: { eq: { literal: THIS_WEEK } } }, first: 100) { edges { node { Id Subject { value } StartDateTime { value } WhatId { value } } } } meetingsNextWeek: Event(scope: MINE, where: { ActivityDate: { eq: { literal: NEXT_WEEK } } }, first: 50) { edges { node { Id Subject { value } StartDateTime { value } Location { value } WhatId { value } } } } tasksThisWeek: Task(scope: MINE, where: { ActivityDate: { eq: { literal: THIS_WEEK } } }, first: 100) { edges { node { Id Subject { value } ActivityDate { value } TaskSubtype { value } } } } } } }\"}" })
```

On a `FieldUndefined` error, drop only that field and re-run once; never guess a substitute.

## 3. Assemble (record only; invent nothing)

- **Closed** — won (`IsWon`) / lost with Reason Lost. Net = sum of won; lost is reported, not subtracted.
- **Moved** — open opps modified this week, grouped by current stage. Mark "stage changed" only if Last Stage Change Date is this week; never draw a from→to arrow.
- **New** — account, amount, starting stage.
- **Activity** — Event count per weekday (skip internal-looking subjects such as standups), plus Task counts split by subtype (Call / Email / other) when Task returns rows. Email counts come only from logged Tasks — say "no logged emails" rather than zero emails.
- **Monday** — next-week meetings, opps closing next week, and any Next Step text naming a date next week.

## 4. Output — widget first

Call `display_widget` as soon as data is assembled (at most one line "Displaying the visualization now (this may take a minute)." before it). `widgetDefinition` must be a native JSON object, not a string. Replace every `{{token}}`: a value that is only a token becomes a typed literal (numbers stay numbers, arrays stay arrays); a token inside a string is interpolated. No `{{…}}` may remain.

```json
{"renderer":{"componentOverrides":{"$":{"type":"mosaic","definition":"tile/widget","children":[{"definition":"tile/row","attributes":{"gap":"sm","align":"center","isWrapped":false},"children":[{"definition":"tile/icon","attributes":{"name":"calendar","size":"xl","alt":""}},{"definition":"tile/text","attributes":{"text":"{{title}}","variant":"h1"}}]},{"definition":"tile/text","attributes":{"text":"{{subtitle}}","variant":"caption","color":"muted"}},{"definition":"tile/separator"},{"definition":"tile/column","attributes":{"width":"md"},"children":[{"definition":"tile/chart","attributes":{"chartType":"column","caption":"{{activityCaption}}","categories":"{{activityCategories}}","series":"{{activitySeries}}","valueFormat":"number","showLegend":true}},{"definition":"tile/waterfall","attributes":{"caption":"{{waterfallCaption}}","valueFormat":"currency","size":"lg","start":"{{waterfallStart}}","steps":"{{waterfallSteps}}","end":"{{waterfallEnd}}"}}]},{"definition":"tile/datagrid","attributes":{"caption":"{{datagridCaption}}","appearance":"striped","size":"sm","columns":[{"key":"status","header":"Status","type":"badge"},{"key":"deal","header":"Deal","type":"text"},{"key":"amount","header":"Amount","type":"currency","align":"right"},{"key":"event","header":"What happened","type":"badge"},{"key":"note","header":"Note","type":"text"}],"rows":"{{dealRows}}"}},{"definition":"tile/callout","attributes":{"variant":"success","title":"{{calloutTitle}}","description":"{{calloutDescription}}"},"children":[{"definition":"tile/row","attributes":{"gap":"sm","align":"center","isWrapped":true},"children":[{"definition":"tile/button","attributes":{"label":"{{button1Label}}","variant":"primary","actions":{"click":[{"definition":"action/sendMessage","attributes":{"content":"{{button1Content}}"}}]}}},{"definition":"tile/button","attributes":{"label":"{{button2Label}}","variant":"secondary","actions":{"click":[{"definition":"action/sendMessage","attributes":{"content":"{{button2Content}}"}}]}}}]}]}]}}}}
```

Tokens:
- `title` ("Weekly wrap — week of Sep 21"), `subtitle` (closed $, won/lost counts, pipeline added, activity count).
- `activityCaption`, `activityCategories` (Mon–Fri), `activitySeries` = `[{name, data:[numbers]}]` — series Meetings, plus Calls / Emails only if Task returned rows.
- `waterfallCaption`, `waterfallStart` `{label, value}`, `waterfallSteps` `[{label, delta}]` (Won, New pipe, Slipped, Lost), `waterfallEnd` `{label}`.
- `datagridCaption`, `dealRows` = `[{status:{value,badgeVariant}, deal (plain string from Account.Name.value or opp name), amount (number), event:{value,badgeVariant}, note}]`; badges Won/success, Lost/error, Slipped/warning; order Won, Lost, Slipped.
- `calloutTitle`, `calloutDescription`, `button1Label`/`button1Content`, `button2Label`/`button2Content` (first-person follow-up prompts).

Omit rows you have no data for; show blanks as blank.

```
display_widget({ resourceType: "dynamic", widgetDefinition: <hydrated object> })
```

## 5. Text — FALLBACK ONLY (if `display_widget` is unavailable or errors)

```
# Week of [Mon date] - [Scope]
## Closed
- ✅ **[Account]** $[X] WON
- ❌ **[Account]** $[X] lost - [Reason Lost]
**Net: $[won total]**
## Moved
- **[Account]** now at [Stage] ($[X])
## New
- **[Account]** $[X] created at [Stage]
## Activity
[N] customer meetings, [N] logged calls/emails
## Monday
- [Time] [Meeting] - [Account]
- Closing next week: [Account] $[X]
- Next steps due: [list]
```
