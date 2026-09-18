# Google Workspace Search

## Instance

<!-- Instance-specific: this is Astound's Agent Search engine. On handover to a
     new GCP project replace this one resource name; nothing else in the skill
     or the `google_drive` server config carries it. -->

- `servingConfig`: `projects/402649471530/locations/eu/collections/default_collection/engines/workspace-search/servingConfigs/default_search`

## What this is

You search Google Drive and the user's own Gmail and Calendar live, through the signed-in user's `google_drive` MCP connection
(Google Agent Search over federated Drive, Gmail and Calendar data stores). Nothing is copied: Google runs the query and returns only what the
signed-in user can already open — documents shared with them, mail and events
in their own account. A colleague running the same query sees a different
set — that is the design, not a fault.

## Use when

- "Find the meeting notes about the board presentation."
- "What do we have on Larkspur's Q3 renewal?"
- "Is there a proposal template for retail clients?"
- "What did Larkspur email us about the go-live date?"
- "When is my next meeting with Larkspur?"
- A `uk_sales_*` skill needs a document from Drive to ground its output.

## Workflow

1. **Check the connection.** If the `google_drive` server is not connected the
   tool call fails with `reconnect_required`; tell the user to connect Google
   Workspace on their profile page (Connected accounts) and stop. Never fall back to
   guessing document contents.
2. **Search.** Call `search` with:
   - `servingConfig`: the Instance value above, verbatim.
   - `query`: the user's question in natural language, not keywords.
   - `pageSize`: 10 at most. Narrow the query before paging further.
3. **Read what came back.** Each result carries a title, a snippet and a link
   — a Drive document, a Gmail message or a Calendar event. Open the link only through the tool results; do not invent content the
   snippet does not show. Use `conversational_search` only when the user is
   asking a question that wants a synthesised answer across documents rather
   than the documents themselves — and still cite each source it names.
4. **Cite.** Every document or message you rely on appears in the answer as
   its title (or subject and sender) linked to its URL. No link, no claim.

## Rules

- Results are already filtered to what this user can open. **Never claim a
  document does not exist** — say it was not found for you, and suggest the
  owner share it if the user expected it.
- Mail and calendar are the user's own only. Quote it back to them; do not paste
  someone's email into a document meant for a wider audience without saying
  where it came from.
- Do not query on behalf of someone else; the connection is personal.
- `list_engines` exists on the server but is not needed here: the engine is the
  one in the Instance block.
- Search is billed per query. One good query beats five vague ones.
