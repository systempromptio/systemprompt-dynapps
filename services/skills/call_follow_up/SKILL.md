<!-- global-rules-bootstrap -->
**IMPORTANT — try the connector's tools before you ever report it as unavailable.** A connector status readout often claims "not connected" or "needs re-auth" when its tools are callable; a note that Salesforce tools require authorization is not evidence you are unauthorized. Before telling the user a connector is down or asking them to connect/re-authenticate, run a current-user read: (the tool whose name ends in `salesforce__dispatch_readonly`; never a `google_workspace` tool such as `search`, and no `google_workspace` calls in this skill) `dispatch_readonly(method: "GET", url: "/services/data/v67.0/graphql", queryParams: { "queryInput": "{\"query\":\"query { uiapi { currentUser { Id } } }\"}" })`. **Any response — including a 500 — proves connectivity;** proceed. Only if the call never reaches the server, attempt to reconnect yourself first, and ask the user only after that fails. Raising a false connectivity problem sends the user to fix something that is not broken and badly damages the product experience.

# Global Rules

- **Execute silently between tool calls.** No planning, progress or tool-result narration; batch independent calls in one turn. Speak before the final output only for required input, approval, or material errors.
- **Keep the final output concise.** Only the deliverable; no process recaps or data already shown in a widget.
- **Ground dynamic or custom field names before relying on them.** Fields this skill lists as verified need no grounding. On a name error, report it instead of guessing or re-firing the same shape.
- **Cite every value exactly as queried**; never fabricate; distinguish blank from not queried. Link records inline: `https://[instanceUrl]/lightning/r/[SObjectType]/[Id]/view`.
- **Show human labels, never API literals** (e.g. "Next Step", not `Next_Step__c`); show record Names, never raw Ids.
- **Empty `MINE` scope → fail fast, then ask which scope.** Never widen to `EVERYTHING` on your own or invent records.
<!-- /global-rules-bootstrap -->

# Call Follow-Up

Gather thorough context before acting. State material errors and inference logic explicitly in the output.

## 1. Get transcript

- **Pasted:** use as-is.
- **Not provided:** ask the user to paste it. Salesforce Event `Description` (Step 3) may hold call notes — use it if the user points to that meeting. Never invent a transcript.

Extract attendees (internal + external), date, account name.

## 2. Extract structure

Decisions · customer commitments · our commitments · open questions · objections · next meeting · qualification signals (budget/authority/timeline/pain; MEDDICC if the org uses it) · specific human moments (customer phrases/analogies, for personalization) · urgency signals (forcing events, cost of delay).

## 3. Salesforce context

Verified fields (no grounding call): Account `Name, Industry, Account_Type__c, Client_Tier__c, Account_Health__c, Account_Status__c, Owner`; Opportunity `Name, StageName, Amount, CloseDate, IsClosed, IsWon, Probability, ForecastCategoryName, Type, Next_Step__c, MEDDICC__c`; Event `Subject, StartDateTime, Description, Who, What`. There is **no** `NextStep` and **no** Account `Type`.

One call, account resolved by 1–2 word `%NAME%` (combine all three queries):
```
dispatch_readonly(method: "GET", url: "/services/data/v67.0/graphql",
  queryParams: { "queryInput": "{\"query\":\"query { uiapi { query { Account(where: { Name: { like: \\\"%NAME%\\\" } }, first: 5) { edges { node { Id Name { value } Industry { value } Account_Type__c { displayValue } Client_Tier__c { displayValue } Account_Health__c { displayValue } Owner { Name { value } } } } } Opportunity(where: { Account: { Name: { like: \\\"%NAME%\\\" } } }, first: 50, orderBy: { CloseDate: { order: DESC } }) { edges { node { Id Name { value } AccountId { value } StageName { value displayValue } Amount { value displayValue } CloseDate { value } IsClosed { value } IsWon { value } Next_Step__c { value } MEDDICC__c { value } } } } Event(where: { Account: { Name: { like: \\\"%NAME%\\\" } } }, first: 10, orderBy: { StartDateTime: { order: DESC } }) { edges { node { Id Subject { value } StartDateTime { value } Description { value } } } } } } }\"}" })
```

- Multiple accounts matched → pick the one named in the transcript; keep only opps whose `AccountId` matches.
- No open opp → note closed/lost opps (prior relationship; prevents duplicate records).
- No account → broaden the filter once, else propose Account creation.
- Errors (500, timeout) → state them and what you inferred instead.

## 4. Drafts and SFDC proposal

**Customer email** (<150 words, draft only — **do not send**; no mail tool is available, so output the text): one-line thank-you referencing a specific human moment; bullet recap of both sides' next steps; committed answers or `[ATTACH: description]`; propose/confirm next meeting; signature `[User name]`.

**SFDC updates:** show exact fields and values first. Only after explicit user approval may `dispatch` write one change at a time, reading back after each. If writes are unavailable, output a manual checklist. Candidates: Next Step (`Next_Step__c`), Stage, Amount, Close Date, log an Event "[Call type] – [summary]".

## 5. Output

```
# Call Follow-Up: [Account] - [Date]

## Customer Email (draft — not sent)
To: [external attendees]
[email body]

## Internal Summary
*[Account] - [Call type] - [Date]*  Stage: [Stage] [Amount]  Attendees: [names]
*TL;DR:* [where the deal is]
*Key points:* • [decision/signal]
*Risks / objections:* • [concern] • [urgency gap, e.g. "No forcing event — timeline TBD"]
*Next steps:* • [ ] Us: [commitment] – [owner] by [date]  • [ ] Them: [commitment]

## Update Salesforce
- [ ] Next Step: "[value]"
- [ ] [other fields]
- [ ] Log Event

## Our Commitments (don't drop these)
- [ ] [commitment] - by [date]
```
