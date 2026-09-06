# Client Research

You produce a one-page 360° brief on a named client: who they are, the platforms
they run, what Astound Digital has done and is doing for them, the people worth
knowing, open opportunities and cases, risks, the last few touches, and three
suggested next actions. Everything in it is traceable to a knowledge-bank hit or a
CRM query, and the brief says so in a Sources footer.

## Use when

- "Brief me on Larkspur Outdoors before I meet them."
- "What do we know about this client and where are we with them?"
- A new account manager is taking over a client and needs the full picture.
- Sales wants to know the history before proposing new work.

## Inputs

- **Client name** (required). If several clients match, ask which one before
  querying anything.
- **Focus** (optional): commercial, delivery, or relationship. Default is all three.
- **Working folder** (optional). If Cowork has a folder connected, the brief is
  saved there; otherwise it is returned in chat.

## Workflow

1. **Search the knowledge bank first.** Call the `knowledge-bank` MCP tool
   `search_knowledge` with `{query, top_k, source_types}`. Run at least three
   searches and note every hit you use, with its `uri` and `date`:
   - `{"query": "<client> overview platforms engagement scope", "top_k": 8, "source_types": ["client_accounts"]}`
   - `{"query": "<client> delivery status risks decisions", "top_k": 8, "source_types": ["project_docs"]}`
   - `{"query": "<client> open issues blockers", "top_k": 8, "source_types": ["project_tickets"]}`
   Use `client_accounts` for who they are, platforms, contract scope and history;
   use `project_docs` and `project_tickets` for how delivery is going right now.
   Read each hit's `date` and `date_source` before you trust it: results rank by
   similarity, not by authority or recency. If nothing comes back, say so and ask
   the user for a brief or a document to work from.
2. **Check whether Salesforce is linked.** If the `salesforce` server is available
   in this session, continue to step 3. If it is not, write "CRM half skipped:
   Salesforce is not linked to this workspace" in the Sources footer and go to
   step 5 with what the knowledge bank gave you.
3. **Look the account up.** Use the Salesforce query tool to find the Account by
   name. If several match, show them and ask. Then pull the account and its
   children in one query:
   `SELECT Name, Industry, AnnualRevenue, Owner.Name, Website, LastActivityDate, (SELECT Name, Title, Email FROM Contacts), (SELECT Name, StageName, Amount, CloseDate, Owner.Name FROM Opportunities WHERE IsClosed = false), (SELECT CaseNumber, Subject, Status, Priority FROM Cases WHERE IsClosed = false) FROM Account WHERE Id = :id`
4. **Pull recent activity.** Query Tasks and Events with `WhatId = :id` from
   `LAST_N_DAYS:90`, ordered newest first, and keep the last five touches with who
   made them. If aggregate or child queries are not permitted for this user, fetch
   the rows with a sensible `LIMIT`, total them yourself, and say the result was
   capped.
5. **Reconcile.** Where the knowledge bank and the CRM disagree (a contact who has
   left, an opportunity already closed), report both with their dates and say
   which is newer. Never merge them silently.
6. **Write the brief** using the layout under Result. Every number carries its
   scope and date: "3 open opportunities worth $410,000 USD (Salesforce, 5
   September 2026)". Suggested next actions come from evidence in the brief, such
   as a stale opportunity, an unanswered case, or a risk with no owner.
7. **Save it.** Write to `briefs/<client-slug>-brief.md` in the working folder,
   where the slug is the client name in lower-case with hyphens. If there is no
   folder, return the same content in chat and say it was not saved.

## Result

A Markdown brief with these sections, in this order, each dropped if empty:

1. **Headline** — who they are, size, industry, account owner, and one line on
   the health of the relationship.
2. **Platforms** — the commerce and supporting platforms they run, and versions
   if known.
3. **Our engagement** — scope, contract shape, start date, key milestones, and
   what is live today.
4. **People to know** — name, title, role in the relationship, and the last time
   we spoke to them. Only people that appeared in a source.
5. **Open opportunities** — name, stage, amount, close date, owner.
6. **Open cases** — number, subject, priority, status, age.
7. **Risks** — each with the evidence for it and an owner if one is known.
8. **Last touches** — the last five interactions, newest first.
9. **Suggested next actions** — exactly three, each tied to a line above.
10. **Sources** — every knowledge-bank hit used (`uri`, date) and every CRM query
    run, plus "CRM half skipped" if Salesforce was not linked.

Measure of done: the file exists at the path you report, every fact in it points
to a source in the footer, and the three actions are specific enough to start today.

## Guardrails

- **Read-only on the CRM.** This skill never creates, edits, or deletes a
  Salesforce record. If the user asks you to update something, show the exact
  change and wait for a clear yes before touching anything.
- **Never invent people, amounts, or dates.** If a section has no source, leave it
  out or mark it "not found in the knowledge bank or CRM".
- **State the scope and date of every number**: whose data, which period, which
  system, and when it was read.
- **Ambiguity is a question, not a guess.** Two clients with similar names, two
  contacts with the same name, or a conflicting fact all go back to the user.
- Keep the brief to one page. Detail that does not fit belongs in the Sources
  footer, not the body.
