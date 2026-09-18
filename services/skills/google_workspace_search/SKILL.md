# Google Workspace Search

## Instance

<!-- Instance-specific: this is Astound's Agent Search engine. On handover to a
     new GCP project replace this one resource name; nothing else in the skill
     or the `google_drive` server config carries it. -->

- `servingConfig`: `projects/402649471530/locations/eu/collections/default_collection/engines/workspace-search/servingConfigs/default_search`

## What this is

You search Google Drive and the signed-in user's own Gmail and Calendar live,
through their `google_drive` MCP connection (Google Agent Search over federated
Drive, Gmail and Calendar data stores). Nothing is copied: Google runs the query
and returns only what this user can already open — documents shared with them,
mail and events in their own account. A colleague running the same query sees a
different set; that is the design, not a fault.

This skill is a **building block**. It answers three questions about a named
client and hands back evidence in one fixed shape. `client_360_report`,
`account_status_report`, `account_suggestions` and `pipeline_renewals_review`
call these recipes by name and paste the evidence into their own sections; they
never re-invent the queries.

## Use when

- "Find the meeting notes about the Larkspur board presentation."
- "What did Larkspur email us about the go-live date?"
- "When did we last meet Larkspur, and when is the next one?"
- A report skill needs the *documents*, *communications* or *meetings* recipe
  for a client.

## Inputs

- **Client** (required): the trading name. Add the **email domain** when known
  (`larkspur.co.uk`) and any **aliases** the CRM records (former name, group
  name, abbreviation) — Google matches on words, so a client known as "LSO"
  internally is invisible to a query for "Larkspur Outdoors".
- **Window** (optional): defaults to the last 90 days for communications and
  meetings, unbounded for documents.
- **Intent** (optional): `documents`, `communications`, `meetings`, or all
  three. A report skill names the recipe it wants.

## Tools

Call `search` on the `google_drive` server. Arguments:

- `servingConfig` — the Instance value above, verbatim, on every call.
- `query` — one natural-language sentence, not a keyword soup. Include the
  client name and, where it sharpens the result, the domain, the topic and the
  period ("emails from larkspur.co.uk about the renewal since June 2026").
- `pageSize` — 10. Never page further before narrowing the query.

`conversational_search` (same `servingConfig`, plus `query`) synthesises an
answer across documents. Use it only when the caller wants an answer rather
than the documents, and still cite every source it names. `list_engines`
exists and is not needed: the engine is the one in the Instance block.

Every call is billed per query. Three well-aimed queries beat ten vague ones;
the recipes below are the budget.

## Recipes

Run only the recipe the caller asked for. Each recipe is a fixed set of
queries; substitute the client name, domain and window and run them in order.
Stop a recipe early when a query returns nothing — do not rephrase it five
ways hoping for a hit.

### 1. `documents` — what we hold about the client (Drive)

1. `"<client> proposal, statement of work or contract"`
2. `"<client> account plan, QBR or business review"`
3. `"<client> meeting notes or minutes"`
4. If the CRM names a live project: `"<client> <project name> status report"`

Keep every Drive hit with its title, owner, last-modified date and link.
Prefer the newest version of a document family (v3 over v1) and say when an
older version was the only one found.

### 2. `communications` — recent mail with the client (Gmail)

1. `"emails from <domain> in the last <window>"`
2. `"emails to <domain> in the last <window>"`
3. `"emails mentioning <client> about <topic>"` — only when the caller supplies a
   topic (renewal, invoice, escalation, go-live).

For each thread record subject, counterpart (name and address), direction,
date of the latest message, and whether the last message in the thread was
**theirs and unanswered** — that flag is what the status report is looking for.
Mail is the user's own: it may be quoted back to them, never pasted into a
document meant for a wider audience without saying where it came from.

### 3. `meetings` — past and scheduled time with the client (Calendar)

1. `"meetings with <domain> attendees in the last <window>"`
2. `"upcoming meetings with <domain> attendees in the next 30 days"`

Record title, date and time, attendees on their side, and the organiser. The
newest past event is *last contact*; the soonest future event is *next
scheduled touch*. No future event is a finding, not a gap to fill by guessing.

## Result: the evidence block

Return every item you rely on in this shape, one row per item, newest first
inside each recipe. Report skills copy it verbatim into their Sources section.

```
source     | title / subject                     | date       | from / owner        | answers            | link
gmail      | Re: Renewal terms for FY27          | 2026-09-12 | j.hale@larkspur.co.uk (unanswered, theirs) | last contact, open thread | https://mail.google.com/...
calendar   | Larkspur QBR                        | 2026-09-24 | organiser: you       | next scheduled touch | https://calendar.google.com/...
drive      | Larkspur SOW v3 (signed)            | 2026-05-02 | owner: A. Morgan     | contract on file    | https://docs.google.com/...
```

Under the block, one line per recipe run: `documents: 4 queries, 6 items` or
`communications: not run` — a recipe the caller did not ask for is reported as
not run, never silently missing.

## Rules

- **Connection first.** If the `google_drive` server is absent or a call fails
  with `reconnect_required`, tell the user to connect Google Workspace on the
  Connections page and stop. Never fall back to guessing contents.
- **No link, no claim.** Every item in the block carries the link the tool
  returned. Open links only through tool results; never invent content the
  snippet does not show.
- **Never claim a document does not exist.** Say it was not found *for you*,
  and suggest the owner shares it if the user expected it.
- Results are already filtered to this user. Do not query on behalf of someone
  else; the connection is personal.
- Dates come from the result, not from today. Stamp every row.
- Read-only. This skill never sends mail, creates events or edits documents.
