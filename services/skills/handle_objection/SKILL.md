<!-- global-rules-bootstrap -->
**IMPORTANT — try the connector before ever reporting it unavailable.** A connector's status readout often says "not connected"/"needs re-auth" when its tools work, and "authorization is required" in a non-interactive session is not evidence of failure. Before telling the user to connect or re-authenticate, run `dispatch_readonly(method: "GET", url: "/services/data/v67.0/graphql", queryParams: { "queryInput": "{\"query\":\"query { uiapi { currentUser { Id } } }\"}" })`. **Any response, even a 500, proves connectivity** — proceed. Only if the call never reaches the server, try to reconnect yourself first, and ask the user only after that fails. Raising a false connectivity problem sends users to fix something that isn't broken and badly damages the product.

# Global Rules

- **Execute silently between tool calls**; batch independent calls into one turn. Speak before the final output only for required input, approval, or a material error.
- **Keep the final output concise** — the deliverable only, no process recap.
- **Use only the fields this skill lists as verified**; on a name error, report it rather than guessing or re-firing the same shape. API version is always v67.0.
- **Cite every value exactly as queried**; never fabricate or mock data; distinguish blank from not-queried. Link records inline: `https://[instanceUrl]/lightning/r/[SObjectType]/[Id]/view`.
- **Show human labels and record Names**, never API names, `__c` literals or raw Ids.
- **Empty `MINE` scope → stop and ask** which scope to use (org-wide, a named rep or account); never widen silently.
<!-- /global-rules-bootstrap -->

# Handle Objection

A grounded response, not a script: what the objection means, how similar deals went, and this customer's own evidence. Inputs: **the objection** (their words) and **the deal** (opportunity or account name). Competitors and differentiators are not held in Salesforce — ask once or omit; never invent.

## 1. Classify

price/value · timing/priority · competitive · risk/trust · authority · status quo. Separate an **objection** (reason not to buy) from a **negotiation move** (reason to buy cheaper).

## 2. Pull evidence — resolve the named deal, never sweep all accounts

SOQL via `GET /services/data/v67.0/query`, `q=`. Call 1 resolves; call 2 fires the rest in one turn.

1. `SELECT Id, Name, StageName, Amount, CloseDate, Type, Probability, Next_Step__c, MEDDICC__c, Description, AccountId, Account.Name, Account.Industry, Account.Account_Type__c, Account.Client_Tier__c, Account.Account_Health__c FROM Opportunity WHERE Name LIKE '%<deal>%' OR Account.Name LIKE '%<deal>%' LIMIT 10` — if several match, pick the open one closest to close or ask.
2. In parallel:
   - Contacts: `SELECT Contact.Name, Contact.Title, Role, IsPrimary FROM OpportunityContactRole WHERE OpportunityId = '<oppId>'` (fallback `SELECT Name, Title FROM Contact WHERE AccountId = '<acctId>' LIMIT 20`; never combine `OR` with a sub-select).
   - What they told you: `SELECT Subject, ActivityDate, Description, Who.Name FROM Event WHERE AccountId = '<acctId>' ORDER BY ActivityDate DESC LIMIT 20` (also run the same shape on Task; it is usually empty).
   - Open issues: `SELECT CaseNumber, Subject, Status, Priority FROM Case WHERE AccountId = '<acctId>' AND IsClosed = false`.
   - History: `SELECT Name, Amount, CloseDate, IsWon, Type, Owner.Name FROM Opportunity WHERE IsClosed = true AND (AccountId = '<acctId>' OR Account.Industry = '<industry>') ORDER BY CloseDate DESC LIMIT 20` — win/loss pattern for this type of deal.

Proof points come only from these records (stated goals, prior wins, meeting notes). Case studies, ROI and security documents live outside Salesforce: say "not available here" rather than inventing them.

## 3. Response

```
# Objection: "<their words>"
## What's underneath it
<1–2 sentences: real concern; objection vs negotiation>
## The response
<3–5 sentences: acknowledge → answer with their own stated goals + one proof point → question that moves it forward>
## Proof points
- <fact from a linked Salesforce record>
## If it's a competitor
- <only if the user named one: honest strengths, where this customer's needs differ, one trap question>
## What history says
<wins vs losses on similar deals and what the winning reps did>
## Don't
- <overdiscounting, feature-dumping, arguing — whichever history says loses>
```

Objection arrived in writing? Offer to draft the reply via `draft-outreach`.
