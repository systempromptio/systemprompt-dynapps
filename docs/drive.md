# Drive

Drive is this install's view of core's shared document store. It is on when
`files.documents.enabled` is `true` in `services/config/files.yaml`; while it is
off, every page below answers 404 and no sidebar entry is shown.

## People

`/drive` is every signed-in person's drive, linked as **My drive** in the
Account group of the sidebar. It mounts core's `sp-document-browser`, which
switches between the person's own documents, the ones shared with them and the
ones they worked on last. **New document** opens a dialog that lists the
templates under `services/documents/templates/` and creates a document through
`POST /api/v1/documents`, then opens it.

`/drive/{id}` shows one document. The page asks the document service for the
viewer's access before it renders, so a person without a grant gets a 404. It
picks the body by format and access:

| Format | Can edit | Body |
|---|---|---|
| HTML, Markdown | yes or no | `sp-document-editor` (read-only without edit access) |
| xlsx | yes | `sp-sheet-grid` |
| pptx | yes | `sp-slide-text` |
| any other, or no edit access | | `sp-document-preview` |

Beside the body sit `sp-presence-bar`, `sp-comments`, `sp-proposal-list` and
`sp-history`. A person with `share` access gets a **Share** button that opens
`sp-share-dialog`. The components and their styles are core's, served from
`/assets/documents/`. This install sends no `Content-Security-Policy`, so the
editor's WebAssembly needs no `wasm-unsafe-eval` exception.

## The workspace

`/drive` is the one listing for every signed-in person; `/admin/drive` paths
redirect to it. Administrators also get an **Agent edits** control on each
folder row, which sets `agent_edit_policy` through
`PATCH /api/v1/document-folders/{id}`: `direct` lands an agent's edits, and
`propose` turns them into proposals for review. `/drive/{id}` opens the
full-screen multiplayer workspace (Word, Excel, PowerPoint); its name menu
reaches the block editor at `/drive/{id}/blocks`. Vendored editor code is
allowlisted in `services/web/vendor.yaml` (`scripts/check-vendor.sh`). Names
come from `/drive/people`. **New document** creates one from a template,
`dynapps-word` first.

## Live pages

A Drive page may ask for an MCP tool's result; the page never holds a
credential and never reaches a server itself. It asks the artifacts broker,
`POST /api/v1/artifacts/{document_id}/mcp/call` with `{server, tool, input}`
(`input` is the tool's arguments, a JSON object; omitted means `{}`). The
broker runs the tool **as the person viewing the page**, through this
instance's own MCP proxy at `/api/v1/mcp/{server}/mcp` on the API's internal
URL, with that person's own session token as the bearer. So the proxy and the
server decide exactly as they do for that person's agent: the server's scopes
and access rules, the tool's `access:` block, and, for a connector, the
person's own OAuth grant. Code: `extensions/web/admin/platform/src/artifacts/`.

- **Who may call.** Only the web login's session: the `access_token` cookie
  with a token core signed `browser_session`, from this site's own pages
  (`Origin` or `Sec-Fetch-Site`, as `/drive` checks them). A request with an
  `Authorization` header is refused, so an agent's, the bridge's or a
  person's `ui` bearer cannot run a page's tools; core's MCP proxy does not
  take the `ui` audience either.
- **What is checked here.** The viewer's `view` on the document, asked of the
  document service as the viewer (a document they cannot see is a 404, as in
  Drive), then that the server is declared and enabled. The page's declared
  capabilities and the viewer's consent will be checked at the same point.
- **The answer.** `200` with the tool result as core shapes it (`content`,
  `structuredContent`, `isError`), or an error `{error, error_key}`:

| `error_key` | Status | When |
|---|---|---|
| `reauth` | 401 | No web session, a token that is not a browser session, or one core's proxy refuses (revoked) |
| `cross_site` | 403 | The session comes from another site's page |
| `not_found` | 404 | The document does not exist or the viewer cannot see it |
| `bad_request` | 400 | Malformed body or names, an unknown server or tool, bad arguments |
| `blocked_by_policy` | 403 | The proxy or the server refused the viewer (scopes, access rules, the tool's `access:`) |
| `server_not_connected` | 409 | The viewer has not connected this connector |
| `server_unavailable` | 503 | The server is off, not running or did not answer (retry); also a connector grant that needs signing in again, which core's proxy does not tell apart |
| `tool_error` | 422 | The tool answered with a JSON-RPC error, or a result larger than 8 MiB |
| `internal` | 500 | The document check failed |

- **Audit.** Every call with a well-formed request from a signed-in viewer
  leaves one row in `artifact_tool_calls`: `user_id`, `document_id`, `server`,
  `tool`, `outcome` (`ok`, `tool_error` for a result with `isError`, or the
  refusal's `error_key`), `duration_ms` and `called_at`. Never the input or
  the result. The proxy and the server keep their own records as for any
  call.
- **Tests.** `tests/contract/admin/src/artifacts_broker/` boots the runtime
  and API on a throwaway database and hosts a fake MCP server in-process with
  core's toolset host, so every call goes through core's real proxy, hop
  token and per-tool access.

## Analytics

The skill and conversation pages under `/admin/analysis` show a **Documents**
panel: the documents the subject created or edited, with their revision count,
each linked to `/drive/{id}`. Revisions come from core's
`report_document_revisions` view. A conversation owns a revision that carries its
context id, and a skill owns the revisions of the conversations its harness
sessions ran: the sessions come from `analysis_skill_version_events`, the source
the skill page counts runs from, joined to their conversations through
`conversation_facts`. The overview shows a **Documents touched** tile: distinct
documents that gained a revision in the window.

## Smoke test

`just docs-smoke` runs the store end to end on a throwaway database, against
the installation running. It builds the `systemprompt-mcp-documents` binary,
boots the runtime and API on a profile over that database, and starts the
binary on the same profile. Agent steps are MCP tool calls to the binary:
creating from the catalog's `dynapps-word` and the compiled-in
`dynapps-note`, guarded edits, history, a comment, a proposal, and a trash
and untrash. People's steps (sharing, `/drive`, the bridge Drive feed) are
REST calls to the API. A refusal is checked in each surface's shape: the
refused tool result's structured `code`, and the HTTP envelope's `error_key`.
The run also holds the tools to a size budget: `tools/list` within 6144
bytes, a one-op edit result within 400 bytes and a three-revision history
within 800 bytes, measured as the model receives a result. The Gates
workflow runs it as the `docs-smoke` job.
