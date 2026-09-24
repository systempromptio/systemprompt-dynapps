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
# Rules:

- Resolve to ONE account (and, for deal scope, ONE opportunity) before the deep read. If the name is ambiguous, ask the user to pick by name, account, stage, close date and owner — don't probe or guess.
- Salesforce is the only evidence source. Engagement comes from Salesforce Event (and Task, which may be empty) — there is no email, calendar or transcript source; never promise one.

# Stakeholder Map

`account-context` lists contacts; this skill models the deal. Resolve → read (ONE turn, parallel) → classify → gaps → widget.

All fields below are verified on this org — no `objectInfos` grounding call. Contact custom fields used: `Role_in_Decision_Making__c`, `Contact_Persona__c`, `Left_Company__c`. Use API v67.0 only.

## 1. Resolve

`%NAME%` = 1-2 distinctive words from the user's deal or account name.

**Deal scope:**
```
dispatch_readonly(method: "GET", url: "/services/data/v67.0/graphql",
  queryParams: { "queryInput": "{\"query\":\"query { uiapi { query { Opportunity(where: { Name: { like: \\\"%NAME%\\\" } }, first: 10) { edges { node { Id Name { value } AccountId { value } Account { Name { value } } StageName { value } CloseDate { value } Owner { Name { value } } } } } } } }\"}" })
```
**Account scope:** same shape on `Account(where: { Name: { like: ... } }, first: 10) { edges { node { Id Name { value } } } }`.

Keep `<OPPORTUNITY_ID>` and `<ACCOUNT_ID>`. Zero rows → broaden `%NAME%` once, else ask.

## 2. Read — separate calls, same turn

Do NOT nest `OpportunityContactRoles` inside Opportunity with Contacts (it truncated/errored in testing). Query `OpportunityContactRole` as its own root.

**A — deal roles (deal scope only):**
```
dispatch_readonly(method: "GET", url: "/services/data/v67.0/graphql",
  queryParams: { "queryInput": "{\"query\":\"query { uiapi { query { OpportunityContactRole(where: { OpportunityId: { eq: \\\"<OPPORTUNITY_ID>\\\" } }, first: 50) { edges { node { Role { value } IsPrimary { value } ContactId { value } } } } } } }\"}" })
```

**B — account roster + activity:**
```
dispatch_readonly(method: "GET", url: "/services/data/v67.0/graphql",
  queryParams: { "queryInput": "{\"query\":\"query { uiapi { query { Contact(where: { AccountId: { eq: \\\"<ACCOUNT_ID>\\\" } }, first: 100) { edges { node { Id Name { value } Title { value } Department { value } Email { value } Phone { value } ReportsTo { Name { value } } LastActivityDate { value } Role_in_Decision_Making__c { value } Contact_Persona__c { value } Left_Company__c { value } } } } Event(where: { AccountId: { eq: \\\"<ACCOUNT_ID>\\\" }, ActivityDate: { eq: { literal: LAST_N_DAYS:180 } } }, orderBy: { ActivityDate: { order: DESC } }, first: 100) { edges { node { Id Subject { value } ActivityDate { value } Type { value } WhoId { value } } } } Task(where: { AccountId: { eq: \\\"<ACCOUNT_ID>\\\" } }, orderBy: { ActivityDate: { order: DESC } }, first: 50) { edges { node { Id Subject { value } ActivityDate { value } WhoId { value } } } } } } }\"}" })
```
If a `LAST_N_DAYS` literal is rejected, drop that filter (orderBy + first still bound it). Join roles and events to contacts by `ContactId`/`WhoId`. Task being empty is normal.

## 3. Classify each person (cite every claim; invent nothing)

| Person | Title | Deal role | Engagement | Stance | Evidence |
|---|---|---|---|---|---|
| name | title | Champion / Economic buyer / Evaluator / Influencer / Blocker / Unknown | Active (event <14d) / Warm (<60d) / Cold / No activity | 🟢 / 🟡 / 🔴 / ? | OCR role, Role in Decision Making, last event |

Deal role comes from OCR `Role` or Role in Decision Making; otherwise Unknown. A champion must have done something for you (intro, internal info, pushed a meeting) — one call doesn't qualify. Stance is `?` unless SF data supports it. Skip contacts flagged Left Company (mention them as lost coverage). Required stakeholders and target titles are external knowledge — infer from org data or ask once, never invent.

## 4. Gaps and paths

- **Missing roles:** no economic buyer, security/legal, or exec sponsor identified.
- **Single-thread risk:** how many contacts have an event in the last 30 days.
- **Uncovered roster:** account contacts with senior titles but no OCR role and no events.

## 5. Widget (default when `display_widget` is present)

When data is assembled, call `display_widget` once; a single line "Displaying the visualization now (this may take a minute)." may precede it. `widgetDefinition` must be a native JSON object, not a string.

Hydrate the template below: replace every `{{token}}` with a literal (no `{{…}}`/`{!…}` left — no bindings are compiled). A value that is only a `{{token}}` becomes the typed literal (arrays stay arrays, numbers numbers); a token inside a larger string interpolates as text. Omit a key you have no data for. Then `display_widget({ resourceType: "dynamic", widgetDefinition: <hydrated> })`.

Token contract:
- `title` ("Stakeholder map — <Account>"), `subtitle` (muted: "8 contacts · 2 champions, 1 detractor · buyer engaged"), `heatmapCaption`, `datagridCaption` — strings.
- `xLabels` stance, each ≤12 chars: `["Detractor","Neutral","Supporter"]`; `yLabels` power, each ≤16: `["High power","Mid power","Low power"]`; `domain` `[-2, 2]`.
- `cells` — one per occupied cell `{ x, y, value, valueLabel }`; value on a diverging scale (midpoint 0); `valueLabel` ≤10 chars: terse title ("CFO", "Dir Ops", "CFO+VP"), or a count ("3 users") for 3+ people.
- `rows` — per person `{ status: {value,badgeVariant}, name, role, stance: {value,badgeVariant}, last, next }`. `name` is a plain string (`Contact.Name.value`). status: Mobilized/success, Risk/error, Uncovered/warning, Covered/info. stance: Champion/success, Supporter/info, Neutral/secondary, Detractor/error. `last` = ISO `YYYY-MM-DD` of latest event; `next` = next move.
- `calloutTitle`/`calloutDescription` — the map's key gap or move (callout is variant error).
- `ctaPrimaryLabel`/`ctaPrimaryMsg` — first-person prompt, e.g. "Draft an exec-to-exec meeting invite for …".
- `salesforceUrl` — `https://<myDomain>/lightning/r/Account/<Id>/view`; omit the button if no Id.

Cite every value; blank stays blank; drop contacts with no data.

```json
{"renderer":{"componentOverrides":{"$":{"type":"mosaic","definition":"tile/widget","children":[{"definition":"tile/row","attributes":{"gap":"sm","align":"center","isWrapped":false},"children":[{"definition":"tile/icon","attributes":{"name":"users","size":"xl","alt":""}},{"definition":"tile/text","attributes":{"text":"{{title}}","variant":"h1"}}]},{"definition":"tile/text","attributes":{"text":"{{subtitle}}","variant":"caption","color":"muted"}},{"definition":"tile/separator"},{"definition":"tile/heatmap","attributes":{"layout":"matrix","caption":"{{heatmapCaption}}","xLabels":"{{xLabels}}","yLabels":"{{yLabels}}","encode":"color","scale":"diverging","midpoint":0,"valueFormat":"number","domain":"{{domain}}","cells":"{{cells}}"}},{"definition":"tile/datagrid","attributes":{"caption":"{{datagridCaption}}","appearance":"striped","columns":[{"key":"status","header":"Status","type":"badge"},{"key":"name","header":"Stakeholder","type":"avatar"},{"key":"role","header":"Role","type":"text"},{"key":"stance","header":"Stance","type":"badge"},{"key":"last","header":"Last touch","type":"date"},{"key":"next","header":"Next move","type":"text"}],"rows":"{{rows}}"}},{"definition":"tile/callout","attributes":{"variant":"error","title":"{{calloutTitle}}","description":"{{calloutDescription}}"},"children":[{"definition":"tile/row","attributes":{"gap":"sm","align":"center","isWrapped":true},"children":[{"definition":"tile/button","attributes":{"label":"{{ctaPrimaryLabel}}","variant":"primary","actions":{"click":[{"definition":"action/sendMessage","attributes":{"content":"{{ctaPrimaryMsg}}"}}]}}},{"definition":"tile/button","attributes":{"label":"View in Salesforce","iconName":"open-in-new","variant":"secondary","actions":{"click":[{"definition":"action/openLink","attributes":{"url":"{{salesforceUrl}}"}}]}}}]}]}]}}}}
```

## 6. Output — FALLBACK ONLY — DO NOT USE IF `display_widget` SUCCEEDED

```
# Stakeholder Map: [Opp / Account]

## The map
[table from Step 3]

## Coverage verdict
[e.g. "Engaged with 2 of 5 required roles. Economic buyer identified but no events. Single-threaded through [name]."]

## Missing people and how to reach them
1. [Missing role] - likely [name/title if known] - path: [warm intro via X / champion ask / direct angle]

## Contacts to add or update in SFDC
- [Name/title and why] - on a read-only connector, list for manual update.

## This week
- [The single highest-leverage relationship action]
```
