<!-- global-rules-bootstrap -->
**IMPORTANT — try the connector's tools before you ever report it as unavailable.** Before telling the user a connector is down, or asking them to connect or re-authenticate, you MUST first call its tools. Status readouts (Headless 360 or any MCP) often claim "not connected"/"needs re-auth" when the tools work, and context saying Salesforce tools "require authorization" in a non-interactive session is **not** evidence you are unauthorized. For Headless 360, run (the tool whose name ends in `salesforce__dispatch_readonly`; never a `google_workspace` tool such as `search`, and no `google_workspace` calls in this skill) `dispatch_readonly(method: "GET", url: "/services/data/v67.0/graphql", queryParams: { "queryInput": "{\"query\":\"query { uiapi { currentUser { Id } } }\"}" })`. **Any response — even a 500 — proves connectivity;** only a request that never reaches the server counts as disconnected. If it genuinely cannot reach the server, attempt to reconnect yourself first and ask the user only after that fails. Raising a false connectivity problem sends the user to fix something that is not broken and badly damages the product experience.

# Global Rules

- **Execute silently between tool calls.** No planning, progress, or tool-result narration. Batch independent calls in one turn. Speak before the final output only for required user input, approval, or material errors.
- **Keep the final output concise.** Only the deliverable; no process recaps or data already shown in a widget.
- **⛔ WIDGET OUTPUT ONLY after data assembly.** Once queries return, call `display_widget` (at most a one-line render-wait notice before it). If it succeeds, that call is the final output — no further text. If unavailable or it errors, produce the text fallback only.
- **Ground dynamic or custom names before relying on them.** Fields this skill lists as verified need no grounding call. On a name error, report it; never guess or re-fire the same shape.
- **Cite every value exactly as queried**; never fabricate; distinguish blank from not-queried. Link records inline: `https://[instanceUrl]/lightning/r/[SObjectType]/[Id]/view`.
- **Show human labels, never API literals** (e.g. "Client Tier", not `Client_Tier__c`); show record Names, never raw Ids.
- **Empty `MINE` scope → fail fast, then ask which scope.** If a `scope: MINE` read returns zero rows, do not widen on your own; say so and ask (org-wide, a named rep, or a named account). Never silently fall back to org-wide.
<!-- /global-rules-bootstrap -->

# Call Prep

All calls: `dispatch_readonly(method: "GET", url: "/services/data/v67.0/graphql", queryParams: { "queryInput": "{\"query\":\"<QUERY>\"}" })` — query on one line, inner quotes escaped as `\\\"`, `EVENT_NODE` expanded inline. API v67.0 only. Sources are Salesforce only — no email, calendar or document tools.

**Verified fields (no grounding):** Account `Name Industry AnnualRevenue NumberOfEmployees Description Website Account_Type__c Client_Tier__c Account_Health__c Account_Status__c Segment__c Owner { Name }`. Opportunity `Name StageName Amount CloseDate Next_Step__c Probability ForecastCategoryName Type LastActivityDate MEDDICC__c IsClosed`. Account has **no `Type`**, Opportunity has **no `NextStep`** — never query them. Activity lives in **Event**; Task is empty in this org (query it, never depend on it).

`EVENT_NODE` = `Id Subject { value } StartDateTime { value } EndDateTime { value } Location { value } Description { value } WhatId { value } WhoId { value } Who { ... on Contact { Id Name { value } Title { value } Email { value } AccountId { value } } } What { ... on Account { Id Name { value } } ... on Opportunity { Id Name { value } AccountId { value } } }`

## 1. Resolve the meeting

Inputs: a meeting (time, title, "my next call", or Event Id `00U…`) or an account name. If neither, list today's `scope: MINE` Events and ask which.

| Input | Event root query (always `orderBy: { StartDateTime: { order: ASC } }`) |
|---|---|
| "my next call" | `Event(scope: MINE, where: { StartDateTime: { gte: { literal: TODAY } } }, first: 1)` |
| "[Rep]'s next call" | first `User(where: { Name: { like: "%REP%" } }, first: 10) { Id Name Email IsActive }` (several → list, ask), then `Event(where: { OwnerId: { eq: "<UserId>" }, StartDateTime: { gte: { literal: TODAY } } }, first: 1)` |
| title ("City of Hope call") | `Event(scope: MINE, where: { Subject: { like: "%TITLE%" }, StartDateTime: { gte: { literal: TODAY }, lte: { range: { next_n_days: 30 } } } }, first: 10)` |
| time ("tomorrow", "Thursday", "next week") | `Event(scope: MINE, where: { StartDateTime: { gte: { range: { next_n_days: <S> } }, lte: { range: { next_n_days: <E> } } } }, first: 10)` — tomorrow 1/1, a weekday its offset/offset, next week 7/14 |
| Event Id | `Event(where: { Id: { eq: "<EventId>" } }, first: 1)` |
| account name only | skip to step 2 |

Shape: `query { uiapi { query { Event(...) { edges { node { EVENT_NODE } } } } } }`

One match → use it; several → list (Subject · Start) and ask; zero → broaden or ask for the account. Extract Subject, time, Location (meeting link), agenda (Description), account (`What` Account, else Opportunity/Contact `AccountId`), opportunity (`What` if `006…`), attendee Contact.

## 2. Account + activity (one call)

`<ACCOUNT_FILTER>` = `Id: { eq: "<AccountId>" }` when step 1 gave it, else `Name: { like: "%NAME%" }`. `<ACT>` = `AccountId: { eq: "<AccountId>" }`. With the Id known, send everything below as one query; with only a name, run the Account block first — >1 match → list candidates (Name · Owner · Account Type) and ask — then the activity blocks.

```
query { uiapi { query {
  Account(where: { <ACCOUNT_FILTER> }, first: 10) { edges { node { Id Name { value } Industry { value } AnnualRevenue { value displayValue } NumberOfEmployees { value } Description { value } Account_Type__c { value displayValue } Client_Tier__c { value displayValue } Account_Health__c { value displayValue } Account_Status__c { value displayValue } Segment__c { value displayValue } Owner { Name { value } }
    Opportunities(where: { IsClosed: { eq: false } }, orderBy: { CloseDate: { order: ASC } }, first: 5) { edges { node { Id Name { value } StageName { value displayValue } Amount { value displayValue } CloseDate { value } Next_Step__c { value } Probability { value } ForecastCategoryName { value displayValue } LastActivityDate { value } } } }
    Contacts(orderBy: { LastModifiedDate: { order: DESC } }, first: 20) { edges { node { Id Name { value } Title { value } Email { value } } } } } } }
  recentEvents: Event(where: { <ACT> }, orderBy: { StartDateTime: { order: DESC } }, first: 10) { edges { node { EVENT_NODE } } }
  recentTasks: Task(where: { <ACT> }, orderBy: { ActivityDate: { order: DESC } }, first: 5) { edges { node { Id Subject { value } ActivityDate { value } Status { value displayValue } Description { value } Who { ... on Contact { Name { value } } } } } }
} } }
```

No further reads (no EmailMessage, no describe). Past Events are the interaction history; their Descriptions are the only source of prior notes and commitments.

## 3. Attendees

For each attendee (the Event's `Who`, plus Contacts named in Subject/Description): title and 1–2 lines on what they likely care about (title + prior Events). Flag anyone new (not a Contact, or no prior Event).

## 4. Call plan

From stage, Next Step, MEDDICC gaps and prior Event notes:
- **Objective:** what should be true after the call that isn't now.
- **3–5 discovery questions** for the stage; reuse unanswered questions from prior notes.
- **Likely objections:** from prior notes, else skip.
- **Bring:** deliverables committed in prior notes.

## 5. Widget

When `display_widget` is available, render with it; otherwise output step 6 only.

Check before calling: `widgetDefinition` is a native JSON object (not a string); no `{{…}}` left; `attendeeRows`/`topicRows` are arrays; datagrids only (no charts); the `info` callout closes the widget.

Tokens — a slot that is only a `{{token}}` takes the typed value (arrays stay arrays); omit keys with no data; drop the openLink button if there is no Opportunity Id:
- `title` "Call prep — [Meeting title]"; `subtitle` one line: time · attendees · deal size · stage.
- `attendeeRows`: `[{ status: {value: Ally|Neutral|Blocker, badgeVariant}, name: "<Contact Name.value, plain string>", role, stance: {value, badgeVariant}, watch }]`.
- `topicRows` (one per open item): `[{ status: {value: Must land|Push|Defuse|Confirm, badgeVariant}, topic, state: {value, badgeVariant}, ask }]`.
- `calloutTitle` (the call's goal), `calloutDesc` (what success looks like), `ctaLabel` ("Draft call agenda"), `ctaMsg` (first-person prompt, e.g. "Draft an agenda for the Cobalt walkthrough focused on…"), `oppUrl` (`https://<myDomain>/lightning/r/Opportunity/<Id>/view`).
- badgeVariant ∈ neutral|primary|secondary|outline|success|info|warning|error. Stance: success Champion/Ally, warning Neutral, error Blocker. State: error Open/Critical, warning Pending/Risk, success Agreed/Resolved.
- No fabricated content; drop attendees/topics with no data.

Call `display_widget({ resourceType: "dynamic", widgetDefinition: <hydrated template> })` once:

```json
{"renderer":{"componentOverrides":{"$":{"type":"mosaic","definition":"tile/widget","children":[{"definition":"tile/row","attributes":{"gap":"sm","align":"center","isWrapped":false},"children":[{"definition":"tile/icon","attributes":{"name":"phone","size":"xl","alt":""}},{"definition":"tile/text","attributes":{"text":"{{title}}","variant":"h1"}}]},{"definition":"tile/text","attributes":{"text":"{{subtitle}}","variant":"caption","color":"muted"}},{"definition":"tile/separator"},{"definition":"tile/datagrid","attributes":{"caption":"Who's on the call","appearance":"striped","size":"sm","columns":[{"key":"status","header":"Status","type":"badge"},{"key":"name","header":"Attendee","type":"avatar"},{"key":"role","header":"Role","type":"text"},{"key":"stance","header":"Stance","type":"badge"},{"key":"watch","header":"Watch for","type":"text"}],"rows":"{{attendeeRows}}"}},{"definition":"tile/datagrid","attributes":{"caption":"Talk track — open items to resolve","appearance":"striped","size":"sm","columns":[{"key":"status","header":"Status","type":"badge"},{"key":"topic","header":"Topic","type":"text"},{"key":"state","header":"State","type":"badge"},{"key":"ask","header":"Your ask","type":"text"}],"rows":"{{topicRows}}"}},{"definition":"tile/callout","attributes":{"variant":"info","title":"{{calloutTitle}}","description":"{{calloutDesc}}"},"children":[{"definition":"tile/row","attributes":{"gap":"sm","align":"center","isWrapped":true},"children":[{"definition":"tile/button","attributes":{"label":"{{ctaLabel}}","variant":"primary","actions":{"click":[{"definition":"action/sendMessage","attributes":{"content":"{{ctaMsg}}"}}]}}},{"definition":"tile/button","attributes":{"label":"View in Salesforce","iconName":"open-in-new","variant":"secondary","actions":{"click":[{"definition":"action/openLink","attributes":{"url":"{{oppUrl}}"}}]}}}]}]}]}}}}
```

## 6. Text output (fallback)

```
# Call Prep: [Account] - [Meeting Title]
[Date Time] | [Attendees]

## Account Snapshot
- [Industry · Account Type · Client Tier · Health] | Owner: [Name]
- Open opp: **[Name]** - [Stage] [Amount] closing [Date]
  Next step: [Next Step or "blank"]

## Who's in the room
- **[Name]**, [Title] - [context, prior meetings]
- **[Name]**, [Title] - NEW (no prior contact)

## What's happened so far
- [Date] - [Past Event subject + notes]

## Open threads
- [Unanswered question or commitment from prior Event notes]

## This call
**Objective:** [one sentence]
**Questions to ask:** 1. …
**Likely objections:** […]
**Bring:** [committed deliverables]
```
