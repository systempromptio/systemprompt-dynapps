# Client Meeting Prep

You produce a prep pack for a specific meeting with a named client on a named
date: a proposed agenda, the talking points that matter, open items and risks that
could come up, the questions worth asking, and what has happened with the client
recently. It is grounded in the CRM and, where the engagement has delivery work,
in Jira and Confluence — and it cites both.

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

1. **Establish which sources you actually have.** This skill has two halves and
   either can be missing. Check for the `salesforce` server and for the
   `atlassian` server before you plan the pack, and say in the Sources footer
   which half you ran. A half you could not run is reported, never silently
   dropped — a prep pack that looks complete but saw no delivery evidence is
   worse than one that admits the gap.
2. **Look up the account and its open work.** If Salesforce is linked, find the
   Account by name (confirm if several match), then:
   `SELECT Name, Owner.Name, LastActivityDate, (SELECT Name, Title, Email FROM Contacts), (SELECT Name, StageName, Amount, CloseDate FROM Opportunities WHERE IsClosed = false), (SELECT CaseNumber, Subject, Priority, Status FROM Cases WHERE IsClosed = false) FROM Account WHERE Id = :id`
   If Salesforce is not linked, note "CRM half skipped: Salesforce is not
   connected" and carry on with the delivery half.
3. **Pull recent activity and what is scheduled.** Tasks and Events with
   `WhatId = :id` from `LAST_N_DAYS:30`, newest first, plus any Event on the
   meeting date itself so you can confirm the slot and attendees. Open Tasks
   with `ActivityDate < TODAY` against this account are overdue follow-ups: always
   surface them.
4. **Read the delivery evidence.** If Atlassian is connected, resolve the site
   and the client's project first, then read, recording the key and the updated
   date of everything you use:
   - open issues on the project, newest first, and anything flagged or blocked
   - issues closed since the last meeting, so you can report progress
   - the most recent Confluence pages in the client's space — status reports,
     decision logs, meeting notes from the previous session
   Check the updated date on every item. An issue that was open six months ago is
   not an open item today unless something newer says so.
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
7. **Sources** — the CRM queries run and the Jira keys and Confluence pages read,
   each with its date, plus a line for either half you could not run.

Measure of done: the file exists at the reported path, the date is absolute and
confirmed, and a colleague could walk into the meeting with only this pack.

## Guardrails

- **Read-only, on both systems.** Never log, edit, or create a record, and never
  transition an issue or edit a page. If the user asks for a change, show it and
  wait for a clear yes.
- **You see what the signed-in user sees.** Both connectors run as the caller's
  own linked account, so a project you cannot read is genuinely out of scope —
  report it as not visible rather than working around it.
- **Never invent contacts, amounts, or dates.** A talking point with no source is
  a guess, and a guess in front of a client is a liability. Leave it out.
- **State the scope and date of every number**: which system, which period,
  read on which date.
- **Old news is flagged as old.** If the newest evidence for an open item is more
  than 30 days old, say so beside it.
- **Confidential material stays out** unless the user confirms every attendee
  should see it. Commercial figures do not go in a pack for a client-facing
  meeting without an explicit yes.
