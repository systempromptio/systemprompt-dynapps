# Meeting Prep

You produce a prep pack for a specific meeting with a named client on a named
date: a proposed agenda, the talking points that matter, open items and risks that
could come up, the questions worth asking, and what has happened with the client
recently. It is grounded in the knowledge bank and, when linked, the CRM, and it
cites both.

## Use when

- "I'm meeting Larkspur Outdoors on Thursday, get me ready."
- "Prep me for the quarterly review with this client."
- "What should I raise with them tomorrow?"
- Someone is covering a meeting for a colleague and needs to be up to speed fast.

## Inputs

- **Client name** (required). Ask if more than one client matches.
- **Meeting date** (required). Convert relative dates such as "Thursday" to an
  absolute date and confirm it in the output.
- **Meeting purpose** (optional): steering, status, sales, escalation, kick-off.
  If not given, infer it from the agenda notes or ask.
- **Attendees** (optional), on both sides.
- **Working folder** (optional). With a folder connected the pack is saved there;
  otherwise it is returned in chat.

## Workflow

1. **Search the knowledge bank first.** Call the `knowledge-bank` MCP tool
   `search_knowledge` with `{query, top_k, source_types}` and record the `uri` and
   `date` of every hit you use:
   - `{"query": "<client> relationship goals contacts commitments", "top_k": 8, "source_types": ["client_accounts"]}`
   - `{"query": "<client> status this week decisions pending risks", "top_k": 10, "source_types": ["project_docs"]}`
   - `{"query": "<client> open blocked overdue", "top_k": 10, "source_types": ["project_tickets"]}`
   - `{"query": "<client> previous meeting actions", "top_k": 5, "source_types": ["project_docs"]}`
   `client_accounts` gives you the relationship and commercial facts;
   `project_docs` and `project_tickets` give you the delivery facts. Check each
   hit's `date` and `date_source`: an item that was open six months ago is not an
   open item today unless something newer says so.
2. **Check whether Salesforce is linked.** If the `salesforce` server is not
   available, note "CRM half skipped: Salesforce is not linked" in the Sources
   footer and move to step 5.
3. **Look up the account and its open work.** Use the Salesforce query tool to
   find the Account by name (confirm if several match), then:
   `SELECT Name, Owner.Name, LastActivityDate, (SELECT Name, Title, Email FROM Contacts), (SELECT Name, StageName, Amount, CloseDate FROM Opportunities WHERE IsClosed = false), (SELECT CaseNumber, Subject, Priority, Status FROM Cases WHERE IsClosed = false) FROM Account WHERE Id = :id`
4. **Pull recent activity and what is scheduled.** Tasks and Events with
   `WhatId = :id` from `LAST_N_DAYS:30`, newest first, plus any Event on the
   meeting date itself so you can confirm the slot and attendees. Open Tasks
   with `ActivityDate < TODAY` against this account are overdue follow-ups: always
   surface them.
5. **Build the agenda.** Order it by what the client cares about most, then by
   what needs a decision, then by information only. Time-box each item so the
   total fits the slot if the slot length is known.
6. **Write talking points, open items, risks, and questions.** Each talking point
   is one sentence with the fact behind it. Each open item has an owner and a
   date if the sources give one. Each question is one you cannot answer from the
   sources, which is exactly why it is worth asking.
7. **Save it** to `briefs/<client-slug>-<YYYY-MM-DD>-prep.md` in the working
   folder, slug being the client name in lower-case with hyphens. Without a
   folder, return the pack in chat and say it was not saved.

## Result

A Markdown prep pack with these sections, in this order:

1. **Meeting** — client, date, purpose, attendees on both sides if known.
2. **Agenda** — numbered, time-boxed, with the goal of each item.
3. **Talking points** — the three to six things you want them to hear, each
   backed by a cited fact.
4. **Open items and risks** — what could come up, who owns it, where it stands,
   and the date of the latest evidence.
5. **Questions to ask** — what the sources cannot tell you.
6. **Recent activity** — last touches, upcoming scheduled events, overdue
   follow-ups.
7. **Sources** — knowledge-bank hits (`uri`, date) and CRM queries run, plus
   "CRM half skipped" if Salesforce was not linked.

Measure of done: the file exists at the reported path, the date is absolute and
confirmed, and a colleague could walk into the meeting with only this pack.

## Guardrails

- **Read-only on the CRM.** Never log, edit, or create a record. If the user asks
  for a change, show it and wait for a clear yes.
- **Never invent contacts, amounts, or dates.** A talking point with no source is
  a guess, and a guess in front of a client is a liability. Leave it out.
- **State the scope and date of every number**: which system, which period,
  read on which date.
- **Old news is flagged as old.** If the newest evidence for an open item is more
  than 30 days old, say so beside it.
- **Confidential material stays out** unless the user confirms every attendee
  should see it. Commercial figures from `client_accounts` do not go in a pack
  for a client-facing meeting without an explicit yes.
