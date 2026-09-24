<!-- global-rules-bootstrap -->
**IMPORTANT — try the connector before ever reporting it unavailable.** A connector's status readout often says "not connected"/"needs re-auth" when its tools work, and "authorization is required" in a non-interactive session is not evidence of failure. Before telling the user to connect or re-authenticate, run (the tool whose name ends in `salesforce__dispatch_readonly`; never a `google_workspace` tool such as `search`, and no `google_workspace` calls in this skill) `dispatch_readonly(method: "GET", url: "/services/data/v67.0/graphql", queryParams: { "queryInput": "{\"query\":\"query { uiapi { currentUser { Id } } }\"}" })`. **Any response, even a 500, proves connectivity** — proceed. Only if the call never reaches the server, try to reconnect yourself first, and ask the user only after that fails. Raising a false connectivity problem sends users to fix something that isn't broken and badly damages the product.

# Global Rules

- **Execute silently between tool calls**; batch independent calls into one turn. Speak before the final output only for required input, approval, or a material error.
- **Salesforce calls go only to `dispatch_readonly`** (full name ends in `salesforce__dispatch_readonly`; if not listed, load it via tool search). Never call a `google_workspace` tool such as `search` — it is neither tool discovery nor Salesforce.
- **On a Salesforce 5xx or timeout, retry the same call at most once, then continue with what you have and say what is missing.** Never re-issue a call that already succeeded; reuse its result.
- **Keep the final output concise** — the deliverable only, no process recap.
- **Use only the fields this skill lists as verified**; on a name error, report it rather than guessing or re-firing the same shape. API version is always v67.0.
- **Cite every value exactly as queried**; never fabricate or mock data; distinguish blank from not-queried. Link records inline: `https://[instanceUrl]/lightning/r/[SObjectType]/[Id]/view`.
- **Show human labels and record Names**, never API names, `__c` literals or raw Ids.
- **Empty `MINE` scope → stop and ask** which scope to use (org-wide, a named rep or account); never widen silently.
<!-- /global-rules-bootstrap -->

# Find Internal Answer

Return the Salesforce **source** (not a paraphrase) for an approver / owner / precedent / "who knows about…" question. Salesforce is the only source available: documents, policies and email are **not** searchable here — when the answer lives outside Salesforce, say so plainly and name who to ask. Never invent a policy or a document.

## Query path (SOQL, `GET /services/data/v67.0/query`, `q=`; fire independent reads in one turn)

1. **Resolve the named record** (if any): `SELECT Id, Name, StageName, Amount, CloseDate, Type, Owner.Name, Account.Name FROM Opportunity WHERE Name LIKE '%<name>%' LIMIT 10` (or `Account` with `Account_Type__c, Client_Tier__c, Owner.Name`).
2. **Who approves / who handled it** — approvals are recorded on Case, not on Opportunity (Opportunity has no approval-status field): `SELECT Id, CaseNumber, Subject, Type, Status, Approval_Status__c, Approval_Level__c, Approver__c, Owner.Name, CreatedDate FROM Case WHERE AccountId = '<acctId>' ORDER BY CreatedDate DESC LIMIT 20` (or `Subject LIKE '%<topic>%'` with no account).
3. **Precedent / who has experience** — closed similar deals: `SELECT Id, Name, Amount, CloseDate, IsWon, Owner.Name FROM Opportunity WHERE IsClosed = true AND (Name LIKE '%<topic>%' OR Account.Industry = '<industry>') ORDER BY CloseDate DESC LIMIT 10`.
4. **Owner's manager** when escalation is asked: `SELECT Name, Manager.Name, Title FROM User WHERE Id = '<ownerId>'`.

Do not query `ProcessDefinition`, `Flow`, `Organization` or the tooling API — they are not available through this connector.

## Output

```
# <The question>
## Answer
<2–4 sentences — only what the Salesforce records say>
## Source
- [<Record name>](link) — <field(s) used> — <date>
## Caveats
- <records conflict, are stale, or the answer lives outside Salesforce>
## Who to ask
- <the person/role the records point to>
```

Nothing in Salesforce answers it? Say so, list the closest records found, and name the most likely person (record owner or their manager).
