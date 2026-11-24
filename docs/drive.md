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

With `?embed=1` the workspace drops what takes a person away from the
document: the home link, the name menu with the format badge and the
notification bell. It marks `<html data-embed>`. The navigation rail stays,
because it is the document's own (the outline of a text, the sheet tabs of a
workbook, the slides of a deck). The bar keeps the save status, the presence
avatars and the Comments, Present, Details and Share buttons, and the editor
keeps its own toolbar. A command that would
leave the page (`leaves: true` in `workspace/commands.js`: back to Drive,
block editor, make a copy, move to trash) is neither listed nor run, and the
"access removed" link opens in a new tab. A client (DynWork desktop) shows
this page as a top-level view of its own, as it does the live shell; the
framing policy is the instance's, unchanged, and without a web session the
Drive's gate sends the page to `/admin/login` with `?embed=1` kept in the way
back.

## Live pages

A Drive page may ask for an MCP tool's result; the page never holds a
credential and never reaches a server itself. It asks the artifacts broker
under `/api/v1/artifacts/{document_id}/`, and the broker runs the tool **as
the person viewing the page**, through this instance's own MCP proxy at
`/api/v1/mcp/{server}/mcp` on the API's internal URL, with that person's own
session token as the bearer. So the proxy and the server decide exactly as
they do for that person's agent: the server's scopes and access rules, the
tool's `access:` block, and, for a connector, the person's own OAuth grant.
The shape follows Anthropic's artifact runtime, contract 0.2.99. Code:
`extensions/web/admin/platform/src/artifacts/`.

- **Who may call.** Only the web login's session: the `access_token` cookie
  with a token core signed `browser_session`, from this site's own pages
  (`Origin` or `Sec-Fetch-Site`, as `/drive` checks them). A request with an
  `Authorization` header is refused, so an agent's, the bridge's or a
  person's `ui` bearer cannot run a page's tools or decide for the viewer;
  core's MCP proxy does not take the `ui` audience either. A visitor without
  a session (a public link, an invite not yet claimed) gets nothing.

### The manifest

The agent that writes a page declares what it calls, with `capabilities` on
the `documents` server's `documents_create`, `documents_import`,
`documents_edit` or `documents_update` (topic `live_pages` of
`documents_guide`):

```json
{"mcp": {"servers": [{"server": "dynapps-odoo", "tools": ["search_read"]}]},
 "artifact": {}, "user": {}}
```

- `mcp` lists each server and the tools the page calls on it (at least one);
  `artifact` lets the page republish itself unless `publish` is `false`;
  `user` (with optional `scopes`, `profile` and `email`) lets it know who is
  viewing. Any other capability is refused.
- Before anything is written, each server must exist here and be on, the
  operator's policy must allow it and each tool, and the server must list
  each tool for the declaring person (asked through the proxy with the
  agent's own bearer). A refused declaration stops the call.
- After core's tool has written, the declaration is recorded on the document
  when it is an HTML page the caller may edit; the tool result says whether
  it was. `{}` clears it, leaving `capabilities` out keeps it, and an update
  with only `document_id` and `capabilities` only declares.
- Stored in `artifact_capabilities` (one row per document, with the contract
  version and who declared it).

### The operator's policy

`services/documents/live-pages.yaml` names the servers a page may use, and
on each `tools: all` or a list. A server it does not name is closed to every
page, and so is everything when the file is missing. Read when the broker and
the `documents` server start. Dynapps ships `dynapps-odoo`, `microsoft-365`
and `dynbrain` whole and the Drive's read tools on `documents`.

### Consent and approvals

- `GET permissions` answers `{capabilities, contract_version, revision,
  writer, states}`: the manifest (`null` without one), the version the viewer
  loaded (a publish's base), whether the viewer may edit, and the states the
  runtime's `permissions.state()` reads: `mcp:<server>` per declared server
  (`granted`, `denied` or `prompt`), `mcp` for all of them, and `artifact`
  and `user` when declared.
- `POST permissions` `{scope, decision}` records `granted` or `denied` for
  `mcp:<server>`, or for every declared server with `mcp`; `DELETE
  permissions/{scope}` forgets it. Per viewer, document and scope, in
  `artifact_consents`.
- A tool that does not declare `readOnlyHint: true` (read from the server's
  own listing for the viewer) is a write: each call needs an approval of that
  exact call. The shell asks the viewer and posts `POST mcp/approve` `{server,
  tool, input}`; the answer's `approval` is valid two minutes, for that
  viewer, document, server, tool and input (hashed with its keys sorted), and
  the call that presents it spends it (`artifact_approvals`).

### Calls

`POST mcp/call` `{server, tool, input?, approval?, cache?}`. The checks, in
order: the viewer's `view` on the document (404 otherwise, as in Drive), the
server exists and is on, the manifest declares the tool, the policy allows
it, the viewer allowed the server, the cache, then one proxy session: the
server's tool list for the viewer (a declared tool it does not list is
`blocked_by_policy`), the approval for a write, the call. `200` with the tool
result as core shapes it (`content`, `structuredContent`, `isError`), or
`{error, error_key}` (`live` on a `conflict`):

| `error_key` | Status | When |
|---|---|---|
| `reauth` | 401 | No web session, a token that is not a browser session, or one core's proxy refuses (revoked) |
| `cross_site` | 403 | The session comes from another site's page |
| `not_found` | 404 | The document does not exist or the viewer cannot see it |
| `bad_request` | 400 | Malformed body or names, an unknown server, bad arguments |
| `not_in_manifest` | 403 | The page does not declare the tool, or the viewer declined the server |
| `blocked_by_policy` | 403 | The live-pages policy, the proxy or the server refused (scopes, access rules, the tool's `access:`) |
| `approval_required` | 428 | The viewer has not allowed the server yet, or a write came without a valid approval |
| `needs_reauth` | 409 | The viewer's connector has no usable grant: never connected, or the provider refused to refresh it; the viewer connects it again on the connections page (not retryable) |
| `server_not_connected` | 409 | Core's proxy answered 409 for any other reason |
| `server_unavailable` | 503 | The server is off, not running or did not answer, or the connector accessor failed (retry) |
| `tool_error` | 422 | The tool answered with a JSON-RPC error, or a result larger than 8 MiB |
| `internal` | 500 | The document or the page's state could not be read |

- **Cache.** Only a read is cached, per viewer, document and call
  (`artifact_call_cache`). Without `cache` a read is stored for five minutes
  and never served to a call; `{staleTime, gcTime}` (ms, at most 7 days)
  serves an entry younger than `staleTime` without running the tool, marked
  `cache: {storedAt, revalidating: false}`; `refresh: true` runs and
  overwrites; `false` neither serves nor stores. A write is never stored or
  served. Revoking or declining a server, losing the document and a denied
  call drop the viewer's entries; `POST mcp/invalidate` `{server?, tool?,
  input?}` drops them on request.
- **Listing.** `POST mcp/tools` `{server?}` lists the manifest as far as the
  viewer reaches: an allowed server with the declared tools it lists for the
  viewer and their annotations (`authStatus: connected`), one not asked yet
  from the manifest alone (`authStatus: unknown`), one whose connector must
  be connected again with no tools (`authStatus: needs_reauth`); declined
  and closed servers are left out. `POST mcp/describe` `{server, tool}`
  answers one declared tool of an allowed server with its schemas
  (`not_in_manifest` otherwise).
- **Reconnect, not outage.** Core asks the extension's connector accessor
  (`/api/public/connectors/{provider}/token`) for the viewer's grant and
  maps any answer but 404 to an unavailable server (`503
  service_not_running`). The accessor therefore answers 404 when the grant
  must be connected again: none stored, or one the provider refused to
  refresh (`invalid_grant`, which also marks the account
  `reconnect_required`). Core then answers `409 provider_not_connected`,
  which the broker reads from the body and answers `needs_reauth`. A real
  outage (the provider or the accessor down) stays `server_unavailable`.
- **Audit.** Every call with a well-formed request from a signed-in viewer
  leaves one row in `artifact_tool_calls`: `user_id`, `document_id`, `server`,
  `tool`, `outcome` (`ok`, `cached`, `tool_error` for a result with
  `isError`, or the refusal's `error_key`), `duration_ms` and `called_at`.
  Never the input or the result. The proxy and the server keep their own
  records as for any call.

### Publishing

`POST artifact/publish` `{html, base_revision}` writes the page as a new
version of its document, as the viewer, byte for byte (its scripts stay).
The page must declare `artifact` (`not_declared`), the viewer must be able to
edit (`not_writer`), the HTML must start with its doctype
(`invalid_content`) and be at most 2 MiB (`too_large`), and `base_revision`
must be the head (`conflict`, with `live`). Comments anchored in the old page
detach. The manifest stays. `200` `{version}`.

### The page runtime and the viewer shell

A live page is opened at `/drive/{document_id}/live` (the shell) and runs in
`/drive/{document_id}/live/frame` (the page). The web Drive's workspace shows
an HTML document that declares capabilities through the same shell, in place
of `/rendered`. Code: `storage/files/js/workspace/live/`,
`extensions/web/admin/platform/src/handlers/ssr/drive/live*.rs`.

- **The frame** serves the stored HTML (`?revision=` the version the shell
  loaded) with the runtime inlined right after `<head>`, under
  `sandbox allow-scripts` (never `allow-same-origin`, so an opaque origin
  with no session and no cookie, framed or not), `default-src 'none'`,
  `connect-src 'none'`, `form-action 'none'`, `base-uri 'none'` and
  `frame-ancestors 'self'`. Whether the page's own scripts run, and which
  origins it loads styles, fonts and scripts from, is the document's
  rendered-HTML tier (`files.documents.renderedHtml`) as on `/rendered`; with
  that tier's scripts off only the runtime runs (by its nonce) and the page
  stays static.
- **The runtime** (`runtime-core.js`, `runtime-mcp.js`, `runtime-use.js`, each a complete function expression, and
  `runtime-boot.js`, which calls them in that order and is inlined with them as one script) defines `window.governance` and
  `window.claude`, one frozen object with only `use(name)`, before any page
  script. It follows Anthropic's artifact runtime contract 0.2.99 (copies of
  the `.d.ts` files are with the plan that built it): `use()` resolves a
  frozen namespace once the shell has answered, memoized per capability
  (`self` is `artifact`), `null` for a name not served, at the top level, or
  after 10 s with no answer. The namespaces are `permissions` (`state`,
  `request`, `manage`), `mcp` (`callTool`, `watchTool`, `listTools`,
  `server`, `describeTool`, `invalidate`), `user` (`me`, `id`, `name`,
  `email`, `avatarUrl`, `isOwner`, `canEdit`, `can`, `profiles`, `search`)
  and `artifact` (`publish(html)`; the files form rejects
  `capability_disabled`; `metadata` is not offered). A tool failure rejects
  `tool_error` with the whole result; a success carries `payload`.
  `watchTool` replays the last result it holds (`cache.revalidating: true`
  while its refresh runs), shares one flight per call identity (input keys
  sorted), polls at `refetchInterval` with a 30 s floor, pauses while the
  page is hidden and catches up on return, takes at most 64 watches, and
  hears every newer result of the identity, `invalidate` included. A request
  the shell does not answer within its reply budget (130 s, kept alive while
  the viewer decides) rejects `upstream_error`.
- **The shell** (`shell.js`, `bridge.js`, `router.js`, `prompts.js`, `viewer.js`) frames
  the page and reads only messages whose source is that frame: the
  runtime's requests and MCP Apps JSON-RPC (`ui/initialize`, `tools/call` on
  the one declared server listing the tool, size notifications). It calls the
  broker with the viewer's web session, which never reaches the frame. Before
  a first call on a connector (a `callTool` or a `watchTool`) it asks the
  viewer to allow it, records the answer with `POST permissions`, and keeps a
  decline for the rest of the load (`not_in_manifest`, never asked again).
  A call the broker answers `approval_required` once the connector is
  allowed is a write: the shell names the server, the tool and a summary of
  the input, and only on Allow posts `mcp/approve` and repeats the call with
  the approval. When a page's own call is the first on a connector, that
  one prompt names the call too and its Allow also approves it, so a first
  write asks once. A watch never approves a write. The permissions panel
  (`permissions.manage`, at most five times a load, or the shell's own
  Permissions button) sets each connector to Allowed, Ask first or Blocked.
  The status line says when data was last updated and how to reconnect a
  connector. `user` hands the page only what the manifest's `user` scopes
  allow: the id with `user: {}`, the name with `profile`, the email with
  `email`. A publish sends the version the view loaded and reloads the frame
  to the new one, or to the live one on a `conflict`.
- **Broker codes to runtime codes.** The broker's keys pass through, except
  `reauth`, `cross_site` and `internal`, which are `upstream_error` (a
  `reauth` also puts "Sign in again" on the status line), and `not_found`,
  which is `not_granted`. `server_unavailable` is stamped `retryable`;
  `needs_reauth` is not, and the status line offers Reconnect, linking to
  the instance's connections page (`/admin/connectors`).
- **Embedding** (`?embed=1`): no bar, only the page, the prompts and the
  status line; the shell posts `{type: "live.status", state, height}` to a
  parent of the same origin. The shell keeps the instance's framing policy,
  so a client (DynWork desktop, Mobile) shows it as a top-level view of its
  own, not inside another site's frame. Without a web session the Drive's
  gate sends the shell to `/admin/login?redirect=/drive/{id}/live?embed=1`;
  a client that holds the person's `ui` session opens one with the session
  handoff below instead.
- **For clients**, `GET manifest` answers `{declared, capabilities,
  contract, live_url}` (`declared: false` and the rest `null` for a document
  with none) to the web session or the person's `ui` bearer, for a reader
  who may see the document, so a client decides whether to open the shell.
  `GET permissions` also answers `viewer: {id, name, email, owner}` for the
  shell.

### The session handoff

The broker and the shell take only the web login's cookie: core's MCP proxy
admits the first-party audiences, never a person's `ui` session. A client
that holds the `ui` session (DynWork desktop; Mobile later) turns it into a
real web session for the view that shows the shell. Code:
`extensions/web/admin/platform/src/handoff/`, table `drive_handoffs`
(migration `029_drive_handoffs.sql`).

1. **Issue.** `POST /api/v1/drive/handoff` `{path}` with `Authorization:
   Bearer <ui session>`, checked as `GET manifest` checks it (core's
   signature and expiry for the `ui` audience, the `ui_session` claim, the
   shape core gives it) and, in the statement that stores the code, that
   its session row is live (not revoked, not expired) and its person active.
   Nothing else issues: no cookie is read, and a web session, an agent's
   bearer, an API key or a Device key is `401 reauth`. `path` must resolve,
   as a browser resolves it, to a path under `/drive/` on this site (`400
   bad_request` otherwise). The answer is `{url}`,
   `<api_external_url>/drive/handoff?code=<code>&path=<path>`, with
   `Cache-Control: no-store`. The code is 32 random bytes (base64url),
   stored only as its SHA-256, bound to the person, the path and the `ui`
   session's sign-in time (`auth_time`), and lives 60 s.
2. **Redeem.** The view loads the URL. `GET /drive/handoff` deletes the code
   as it reads it, live or not; storing a code sweeps the expired ones, so
   nothing else is kept. For a live code of an active person, core's
   `mint_browser_session` mints a web session whose `auth_time` is the `ui`
   session's sign-in time, never the redeem, so the web's freshness and
   step-up rules see the real sign-in; the cookie is set by core's own
   `session_cookie_redirect` (the SSO callback's `access_token`, `Path=/`,
   `HttpOnly`, `SameSite=Lax`, `Secure` on https), the login is recorded in
   `user_activity` with `metadata.via = "dynwork_handoff"` (deduplicated
   within the hour as every login), and the answer is `303` to the code's
   path. Every failure (unknown, spent or expired code, inactive person,
   failed mint) is the same `303` to `/admin/login?redirect=<path>`, the
   query's `path` when it is a Drive path, `/drive` otherwise. There is no
   grace for a second redeem: DynWork's view prefetches nothing.
3. **Scope.** Both routes are mounted on every profile, production
   included, beside the web router.

### Tests

`tests/contract/admin/src/artifacts_broker/` boots the runtime and API on a
throwaway database and hosts two servers in-process: a fake MCP server with
core's toolset host, and the extension's own `documents` server with its
router. The agent declares a page through `documents`; every other request
goes as the shell sends it, so each call crosses core's real proxy, hop token
and per-tool access. The same suite loads the shell and the frame as a
browser does (the frame's policy, the runtime inlined before the page, the
trip to sign in) and reads the manifest with the web and the `ui` session.
It also hands a `ui` session to a web session the broker takes (`handoff`),
and calls an external server whose accessor answers 404 and then 503
(`reconnect`); the accessor's own 404 for a refused refresh is
`tests/integration/admin-config/src/portal_e2e.rs`, against the fake portal.
The runtime and the shell's bridge and router have their own tests under
`tests/bridge/live-*.test.mjs` (`node --test`).
The same suite loads the editor page `/drive/{id}` with and without
`?embed=1` (`editor`): the embedded page has `data-embed` and no home link,
name menu or bell, and keeps the rail, the comments, details and share buttons
and the editor slot; the plain page has all of the chrome and no `data-embed`.

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
