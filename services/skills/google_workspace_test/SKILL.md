# Google Workspace Connector Test

A read-only check of the `google_workspace` MCP server (Google Agent Search over Drive, Gmail and
Calendar). It proves the connector works before any sales skill depends on it. It never scores or
summarises content.

## Steps

Run these three calls in order. Load the tools with ToolSearch if they are deferred. Do not retry a
failed call more than once, and do not try other tools instead.

1. `list_engines` with no arguments.
2. `search` with exactly `{"query": {"text": "<the user's term, or: quarterly report>"}}`. The query is
   an object with a `text` field, never a bare string. Do not pass `servingConfig`, `parent`, `name`
   or `method`: the gateway supplies the serving config.
3. `conversational_search` with exactly `{"query": {"text": "What meetings do I have this week?"}}`.

## Output

One table, then one line of diagnosis. Nothing else.

| call | outcome | detail |
|---|---|---|
| list_engines | ok / empty / error | engine names, or the error message verbatim |
| search | ok / empty / error | result count and up to 3 titles, or the error verbatim |
| conversational_search | ok / empty / error | first sentence of the answer, or the error verbatim |

Diagnosis, from the first error:
- `Permission 'mcp.googleapis.com/tools.call' denied`: the connected Google account lacks
  `roles/mcp.toolUser` on the engine's project.
- `Invalid JSON payload` / `Invalid value at 'query'`: the call shape was wrong. Report it as a
  skill defect.
- Every call ok but `search` empty: Workspace trust or smart-features settings are not active yet
  (can take up to 24 hours).
- All ok with results: "Google Workspace connector working."
