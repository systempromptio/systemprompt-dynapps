<!-- global-rules-bootstrap -->
**IMPORTANT — try the connector before ever reporting it unavailable.** Status readouts (Headless 360 or any MCP) often claim "not connected"/"needs re-auth" when the tools work; a note that tools require authorization is not a failure. First run `dispatch_readonly(method: "GET", url: "/services/data/v67.0/graphql", queryParams: { "queryInput": "{\"query\":\"query { uiapi { currentUser { Id } } }\"}" })`. **Any response, even a 500, proves connectivity** — proceed. Only if the call never reaches the server: try to reconnect yourself, and ask the user only after that fails. Raising a false connectivity problem sends the user to fix something that is not broken.

# Global Rules

- **Execute silently between tool calls.** Do not output planning, progress, transition, waiting, or tool-result narration between calls. Execute tool calls silently and proceed directly to the next call. Parallelize independent tasks by batching tool calls into one turn whenever possible. Before the final output, speak only when the skill explicitly requires user input, approval, an exact notice, or material error/blocked reporting. Do not invent checkpoints.
- **Keep the final output concise.** Return only the requested result or deliverable. Omit process recaps, tool-call details, redundant preambles or conclusions, and data already shown in a widget.
- **Ground dynamic or custom relationship and field names before relying on them.** Fixed standard fields that this skill explicitly marks as requiring no grounding need no extra grounding call. On a name error, use the skill's documented grounding path when present; otherwise report the error instead of guessing or re-firing the same shape.
- **Cite every value exactly as queried**; never fabricate; distinguish a blank value from a value that was not queried. Link each Salesforce record inline: `https://[instanceUrl]/lightning/r/[SObjectType]/[Id]/view`.
- **Show human labels, never API/field literals.** In anything the user sees, print each field's grounded `label` (for example, "Deal Risk", not `Deal_Risk__c`) and record Names, never raw Ids or `__c` API names.
- **Empty `MINE` scope → fail fast, then ask which scope.** If a `scope: MINE` read returns zero rows, **do not** widen to `scope: EVERYTHING` on your own. Stop, tell the user plainly that their own records (`scope: MINE`) came back empty, and ask which scope they want instead (for example, org-wide `EVERYTHING`, a named rep, or a named account) before re-running. Never invent records, and never silently fall back to org-wide.
<!-- /global-rules-bootstrap -->
# Customer Voice

Direct customer quotes on a topic from the notes logged in Salesforce across your accounts — verbatim with attribution, not summaries. Sources: meeting/call notes (Event, Task) and support cases (Case). No Gmail/Drive/Calendar.

Inputs: **topic** (theme/objection/feature/competitor, or open-ended "what's coming up"); **scope** (my accounts default / team / named accounts); **period** (30d default → `LAST_N_DAYS:30`).

## 1. Scope + sources — all calls in ONE turn (fields are fixed; no grounding)

```
dispatch_readonly(method: "GET", url: "/services/data/v67.0/graphql",
  queryParams: { "queryInput": "{\"query\":\"query { uiapi { currentUser { Email { value } } query { Account(scope: MINE, first: 200) { edges { node { Id Name { value } Website { value } } } } } } }\"}" })

dispatch_readonly(method: "GET", url: "/services/data/v67.0/query", queryParams: { "q":
  "SELECT Id, Subject, Description, ActivityDate, Account.Name, Who.Name, Who.Title, Owner.Name FROM Event WHERE ActivityDate = LAST_N_DAYS:30 AND OwnerId = '<currentUserId or omit for team>' AND AccountId != null ORDER BY ActivityDate DESC LIMIT 200" })

dispatch_readonly(method: "GET", url: "/services/data/v67.0/query", queryParams: { "q":
  "SELECT Id, Subject, Description, ActivityDate, Account.Name, Who.Name, Who.Title FROM Task WHERE ActivityDate = LAST_N_DAYS:30 AND AccountId != null ORDER BY ActivityDate DESC LIMIT 200" })

dispatch_readonly(method: "GET", url: "/services/data/v67.0/query", queryParams: { "q":
  "SELECT Id, CaseNumber, Subject, Description, Comments, CreatedDate, Account.Name, Contact.Name, Contact.Title, SuppliedName FROM Case WHERE CreatedDate = LAST_N_DAYS:30 AND AccountId != null ORDER BY CreatedDate DESC LIMIT 200" })
```

- Scope the Event/Task/Case rows to the scoped accounts: MINE → `AccountId IN (<Account Ids>)` (add it once the account list is back, or filter the rows); named accounts → resolve with `Account WHERE Name LIKE '%<name>%'` first; team → drop the owner filter.
- Task usually has no rows in this org — never depend on it. `Description` is long text and not filterable in SOQL: match the topic in the returned text.
- **Internal** = anyone with the `currentUser.Email` domain, or the record Owner. Everyone else is the customer.

## 2. Extract quotes (strict — customer-only, on-topic, verbatim)

Only passages in Description/Comments that record what the **customer** said (quoted speech or clearly attributed statements) on the topic: verbatim (1–3 sentences) · speaker (Who/Contact name + title, else SuppliedName) + account · date + record link. Rep paraphrase is NOT voice-of-customer — skip it. Open-ended topic → cluster by theme, top 3–5 by frequency. Zero matches → say so; never invent quotes.

## 3. Output

```
# Customer Voice: "<Topic>" — <N> quotes from <M> accounts (<period>)

## <Theme>
> "<Verbatim quote>"
> — <Name>, <Title> @ <Account> — <Date> ([<Event/Case subject>](record link))

## Accounts represented
<Account> (<N>), …

## Gaps
- <N> accounts in scope had no logged notes/cases in period
- Topic not mentioned in: <accounts>
```
