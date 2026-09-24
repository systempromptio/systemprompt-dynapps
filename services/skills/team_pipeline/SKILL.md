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

# Team Pipeline

Resolve team → ONE combined read → analyze → output. Exactly two `dispatch_readonly` calls in the normal path. No schema grounding (fields below are verified for this org), no per-opp follow-ups, no evidence/email/document sources.

**Verified Opportunity fields:** `StageName ForecastCategory Amount ExpectedRevenue Probability CloseDate IsClosed IsWon LastActivityDate CreatedDate Next_Step__c OwnerId Owner.Name Account.Name`. There is **no** `NextStep` — use `Next_Step__c` (long text: selectable, **not filterable**, so count blanks client-side). User has no quota field.

## 1. Resolve team (call 1)

"my team" (default) — active reps whose Manager = current user (the bootstrap `currentUser` read gives `<CURRENT_USER_ID>`):

```
dispatch_readonly(method: "GET", url: "/services/data/v67.0/graphql",
  queryParams: { "queryInput": "{\"query\":\"query { uiapi { query { User(where: { ManagerId: { eq: \\\"<CURRENT_USER_ID>\\\" }, IsActive: { eq: true } }, first: 50) { edges { node { Id Name { value } Title { value } } } } } } }\"}" })
```

Named reps instead: same root with `where: { Name: { like: \\\"%REP%\\\" } }, first: 10` returning `Id Name Title IsActive`. One match → take it; several (e.g. regional `(AM)`/`(SG)` variants) → list Name · Title · active and ask. Never pull candidates' pipelines to guess. Zero reps → say so and ask which reps to use.

## 2. One combined read (call 2)

`<IDS>` = quoted rep Ids (`\\\"005…\\\", …`). `<Q_START>`/`<Q_END>` = current quarter, `<STALE>` = today − 14d, `<TODAY>` — all `{ value: \\\"YYYY-MM-DD\\\" }` DateInputs (bare strings fail `WrongType`). `orderBy` takes ONE field only.

```
dispatch_readonly(method: "GET", url: "/services/data/v67.0/graphql",
  queryParams: { "queryInput": "{\"query\":\"query { uiapi { aggregate { rollup: Opportunity(where: { OwnerId: { in: [<IDS>] }, IsClosed: { eq: false }, CloseDate: { gte: { value: \\\"<Q_START>\\\" }, lte: { value: \\\"<Q_END>\\\" } } }, groupBy: { OwnerId: { group: true }, ForecastCategory: { group: true } }) { edges { node { aggregate { OwnerId { value } ForecastCategory { value } Id { count { value } } Amount { sum { value } } ExpectedRevenue { sum { value } } } } } } won: Opportunity(where: { OwnerId: { in: [<IDS>] }, IsWon: { eq: true }, CloseDate: { gte: { value: \\\"<Q_START>\\\" }, lte: { value: \\\"<Q_END>\\\" } } }, groupBy: { OwnerId: { group: true } }) { edges { node { aggregate { OwnerId { value } Id { count { value } } Amount { sum { value } } } } } } stale: Opportunity(where: { OwnerId: { in: [<IDS>] }, IsClosed: { eq: false }, LastActivityDate: { lte: { value: \\\"<STALE>\\\" } } }, groupBy: { OwnerId: { group: true } }) { edges { node { aggregate { OwnerId { value } Id { count { value } } } } } } pastClose: Opportunity(where: { OwnerId: { in: [<IDS>] }, IsClosed: { eq: false }, CloseDate: { lt: { value: \\\"<TODAY>\\\" } } }, groupBy: { OwnerId: { group: true } }) { edges { node { aggregate { OwnerId { value } Id { count { value } } } } } } } query { top: Opportunity(where: { OwnerId: { in: [<IDS>] }, IsClosed: { eq: false }, CloseDate: { gte: { value: \\\"<Q_START>\\\" }, lte: { value: \\\"<Q_END>\\\" } } }, orderBy: { Amount: { order: DESC } }, first: 60) { edges { node { Id Name { value } StageName { value displayValue } ForecastCategory { value } Amount { value displayValue } CloseDate { value } Next_Step__c { value } LastActivityDate { value } Probability { value } OwnerId { value } Account { Name { value } } Owner { Name { value } } } } } } } }\"}" })
```

Aggregates give full-book numbers; `top` is a bounded 60-deal sample for named specifics. Map `OwnerId` → rep via the call-1 roster.

**Only if an aggregate root errors** (don't retry the same shape): one SOQL fallback per failed root, e.g. `GET /services/data/v67.0/query` with `q=SELECT OwnerId, ForecastCategory, COUNT(Id) cnt, SUM(Amount) amt, SUM(ExpectedRevenue) weighted FROM Opportunity WHERE OwnerId IN (<IDS>) AND IsClosed = false AND CloseDate >= <Q_START> AND CloseDate <= <Q_END> GROUP BY OwnerId, ForecastCategory` (won/stale/pastClose: same pattern, `GROUP BY OwnerId`). Never filter on `Next_Step__c`.

## 3. Analyze

- **Scoreboard per rep:** Closed = `won` Amount sum; Commit = `Commit`-category Amount sum; Best case = `BestCase` row; Weighted = Σ ExpectedRevenue. Quota not in SFDC → ask once, keep for session. **Call** = Ahead (closed ≥ quota or weighted covers gap) / On-track / Behind. Commit > Weighted → stages likely optimistic; challenge it.
- **Deals that decide the quarter (3–5):** large Amount × late stage × rep needs it × closing this period. Each: *Why it matters* (the math: "$X is N% of [rep]'s gap"), *Risk* (evidenced from the record), *Do this* (one concrete leader action).
- **Flags:** at-risk Commit deals; coaching signals (high stale-% or low coverage); hygiene ranking from `stale`/`pastClose` counts plus blank `Next_Step__c` counted in the sample (say it is sample-based); big swings (>2× average).
- **Monday questions:** one conversational, deal-grounded question per rep.

## 4. Output — widget first

Once data is assembled, call `display_widget` (at most the one-line "Displaying the visualization now (this may take a minute)." notice before it). `widgetDefinition` must be a native JSON object, never a quoted string. Hydrate the template below and call `display_widget({ resourceType: "dynamic", widgetDefinition: <hydrated> })` once, with no `{{…}}`/`{!…}` left. A slot that is only a `{{token}}` becomes the typed value (numbers stay numbers, arrays stay arrays); a token inside a string interpolates as text; a token you have no data for drops its block (never send empty arrays).

Tokens:
- `title` ("Team pipeline — Q3, week 6 of 13"), `headerStatus` ("BEHIND PACE"), `subtitle` ("$8.8M closed of $13.0M plan · 7 reps · 49 days left").
- `chartCaption`; `chartCategories` = rep names (strings); `chartSeries` = three FORECAST series `{name, data}` — Proposal / Negotiation / Closing — each `data` one raw-dollar number per rep in category order.
- Meter (one only): `meterLabel`, `meterValue`/`meterMax`/`meterTarget` (numbers, millions), `meterValueLabel` ("$8.8M of $13.0M plan — 68%"), `meterTargetLabel`, `meterStatus`, `meterBands` = `[{from,to,variant:error|warning|success,label}]` tiling 0 → plan, target reachable below max.
- `datagridCaption`; `datagridRows` = `[{rep, plan, closed, commit, gap, attain, status?}]` — `rep` is a plain string (`Owner.Name.value`), currencies raw numbers, `attain` 0–100; `status` `{value: Behind|At risk|Ahead, badgeVariant: error|warning|success}` only on risky/ahead rows.
- `calloutTitle` ("3 deals decide the quarter"), `calloutDescription`, `ctaPrimaryLabel`/`ctaPrimaryMsg`, `ctaSecondaryLabel`/`ctaSecondaryMsg` (first-person prompts sent via action/sendMessage).

```json
{"renderer":{"componentOverrides":{"$":{"type":"mosaic","definition":"tile/widget","children":[{"definition":"tile/row","attributes":{"gap":"sm","align":"center","justify":"between","isWrapped":true},"children":[{"definition":"tile/row","attributes":{"gap":"sm","align":"center"},"children":[{"definition":"tile/icon","attributes":{"name":"dashboard","size":"xl","alt":""}},{"definition":"tile/text","attributes":{"text":"{{title}}","variant":"h1"}}]},{"definition":"tile/badge","attributes":{"label":"{{headerStatus}}","variant":"warning"}}]},{"definition":"tile/text","attributes":{"text":"{{subtitle}}","variant":"caption","color":"muted"}},{"definition":"tile/separator"},{"definition":"tile/column","attributes":{"gap":"md"},"children":[{"definition":"tile/chart","attributes":{"chartType":"column","caption":"{{chartCaption}}","categories":"{{chartCategories}}","series":"{{chartSeries}}","stackMode":"stacked","valueFormat":"compact","showValues":false}},{"definition":"tile/meter","attributes":{"label":"{{meterLabel}}","value":"{{meterValue}}","min":0,"max":"{{meterMax}}","target":"{{meterTarget}}","valueFormat":"currency","valueLabel":"{{meterValueLabel}}","targetLabel":"{{meterTargetLabel}}","status":"{{meterStatus}}","bands":"{{meterBands}}"}}]},{"definition":"tile/datagrid","attributes":{"caption":"{{datagridCaption}}","appearance":"striped","size":"sm","defaultSort":{"key":"gap","direction":"asc"},"columns":[{"key":"status","header":"Status","type":"badge"},{"key":"rep","header":"Rep","type":"avatar"},{"key":"plan","header":"Plan","type":"currency","align":"right"},{"key":"closed","header":"Closed","type":"currency","align":"right","sortable":true},{"key":"commit","header":"Commit","type":"currency","align":"right"},{"key":"gap","header":"Gap to plan","type":"currency","align":"right","sortable":true},{"key":"attain","header":"Attainment","type":"databar","target":100}],"rows":"{{datagridRows}}"}},{"definition":"tile/callout","attributes":{"variant":"warning","title":"{{calloutTitle}}","description":"{{calloutDescription}}"},"children":[{"definition":"tile/row","attributes":{"gap":"sm","align":"center","isWrapped":true},"children":[{"definition":"tile/button","attributes":{"label":"{{ctaPrimaryLabel}}","variant":"primary","actions":{"click":[{"definition":"action/sendMessage","attributes":{"content":"{{ctaPrimaryMsg}}"}}]}}},{"definition":"tile/button","attributes":{"label":"{{ctaSecondaryLabel}}","variant":"secondary","actions":{"click":[{"definition":"action/sendMessage","attributes":{"content":"{{ctaSecondaryMsg}}"}}]}}}]}]}]}}}}
```

## 5. Text — FALLBACK ONLY — DO NOT USE IF `display_widget` SUCCEEDED

```
# Team Pipeline - [Period]
Target $[X] | Closed $[X] | [N] days left

## Scoreboard
| Rep | Closed | Commit | Weighted | Gap | Call |
|---|---|---|---|---|---|
| [Name] | $[X] | $[X] | $[X] | $[X] | Behind |
| **Team** | **$[X]** | **$[X]** | **$[X]** | **$[X]** | |

## Deals That Decide the Quarter
**[Account]** - $[X], [Stage], [Rep]
  Why it matters: [the math]
  Risk: [specific, evidenced]
  → **Do this:** [one concrete leader action]

## Risk Flags
- Stale 14d+: [N] opps - [Account, ...]
- Past close date: [N] opps
- Blank next step (top-60 sample): [N]

## Monday Questions
- **[Rep]:** "[question]"
```
