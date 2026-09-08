# 2026-09-08 — The knowledge bank reads Atlassian through the connected MCP

**Status:** decided (Ed, 2026-09-08). Supersedes the header-credential
contract in `2026-09-01-knowledge-bank-remote-shared-index.md` §3 and
`2026-09-01-launch-scope-identity-and-deploy.md`. Grants, retrieval and the
shared index are unchanged.

## Decision

1. **No Atlassian credential in any `.mcp.json`, ever.** The
   `x-atlassian-email` / `x-atlassian-api-token` headers, the REST client
   behind them and the `MissingHeader` error are deleted, not deprecated.
2. **`sync_knowledge` reads Confluence and Jira through the hosted Atlassian
   MCP** (`services/mcp/atlassian.yaml`) as an MCP client of this instance's
   own gateway route, `/api/v1/mcp/atlassian`, carrying the caller's
   systemprompt bearer. Core's `external_auth` swaps that bearer for the
   Atlassian token the caller banked when they connected their account on
   `/admin/profile#connected-accounts` (`mcp_connector_credentials`). The
   knowledge bank never holds an Atlassian credential and needs no broker
   secret; every upstream read is governed and audited like any other MCP
   call, attributed to the caller.
3. **Degrade, never guess.** The connection is attempted *before* a run
   starts. An unscoped sync that cannot reach Atlassian — account not
   connected, upstream down, gateway refusing — syncs the files categories,
   records the reason as each Atlassian category's `last_error` (so
   `list_knowledge_sources` shows it), and says so in the reply with the link
   to connect. A sync that names an Atlassian category explicitly is refused
   with the same reason. A single page or issue that fails to read is logged
   and skipped, never fatal to its space or project; the listing stays
   complete so nothing is pruned by mistake. `search_knowledge` is untouched.
4. **The connector consents read *and* write.** `ATLASSIAN_SCOPES` gains
   `write:jira:agent-interface` and `write:confluence:agent-interface`, so the
   BA skills (`createJiraIssue`, `createConfluencePage`) and the knowledge bank
   share one grant. Widening the scopes invalidates every banked grant: users
   reconnect once on `/admin/profile`.

## Why

- The per-user connector shipped after the 2026-09-01 launch-scope decision,
  which chose header credentials only because no OAuth flow existed yet.
  Keeping two credential paths for one upstream is exactly the dual-path
  debt the branch rules forbid.
- The MCP route is the only transport the connector's grant is guaranteed to
  work with: the token was issued for `mcp.atlassian.com`. Calling
  `api.atlassian.com` REST with it would have needed a scope probe, the
  broker secret inside the knowledge bank, and a second URL scheme.
- Reading through the gateway puts every Atlassian read in the same audit
  spine as every other tool call (`mcp_tool_executions`, trace ids), which
  the header path bypassed entirely.

## Architecture page

The flow, access model, degradation matrix, test pyramid and the measured
timings (full rebuild of 32 Atlassian documents through the hosted MCP on the
local provider: 30 s; incremental: 1 s) are drawn up at
<https://claude.ai/code/artifact/83933607-08c2-4850-92db-35b7c8a8eae8>.

## Contract pinned by tests

- `tests/unit/mcp/src/atlassian_mcp_gateway.rs` fakes the gateway route
  with wiremock; `atlassian_sources.rs` pins bearer forwarding, site
  resolution (one site inferred, several need `atlassian.cloud_id`), paging,
  the incremental cursor, comments, refused reads, and per-page skipping.
- `tests/integration/web/src/knowledge_dispatch.rs` pins the two degradation
  branches through the real tool dispatch.
- `tests/integration/mcp/src/knowledge_live.rs` runs the whole pipeline
  against a live gateway when `KNOWLEDGE_BANK_LIVE_BEARER` is set.
- `playwright/tests/knowledge-atlassian-e2e.spec.ts` is the end-to-end
  proof: seed a labelled demo workspace through the Atlassian MCP, sync,
  search, and check grants and audit rows.

## Operating note: restarts under open sessions

The `knowledge-bank` server is a local child process; rmcp's streamable-HTTP
session handles live in that process, while the Postgres session store keeps
only the identity row. Restarting the child (any `services/` reload, a
`just start`) therefore invalidates every open MCP session on it: the next
request with the old `mcp-session-id` gets a 404, the client re-initialises,
and Claude Code shows one "Connection error. Retrying…" before the call
succeeds. The `atlassian` route is an external proxy with no child and keeps
its state. Do not restart the server under a demo; a single visible retry
right after a restart is expected and is not a knowledge-bank fault.

## Tool contract (hosted Atlassian MCP, pinned 2026-09-08)

`getAccessibleAtlassianResources` → `resources[].cloudId`;
`listConfluenceSpaces {cloudId, keys}` → `results[].id`;
`listConfluenceContent {cloudId, type: page, spaceId, status, limit, cursor}`;
`getConfluenceContent {cloudId, content_id, detail: full, content_format:
markdown, include_metadata}` → `body.value`, `metadata.version.createdAt`;
`listConfluenceComments {cloudId, content-id, comment-type: footer,
body-format: markdown}`; `searchJiraIssuesUsingJql {cloudId, jql, fields,
maxResults, nextPageToken, view: full, responseContentFormat: markdown}`;
`listJiraIssueComments {cloudId, issueIdOrKey}`. Every payload arrives wrapped
in `{"data": …}` inside a text content block.
