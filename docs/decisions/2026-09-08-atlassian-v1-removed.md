# 2026-09-08 — The v1 Atlassian link is removed; the hosted connector is the only path

**Status:** decided (Ed, 2026-09-08). Companion to
`2026-09-08-knowledge-bank-reads-through-connected-mcp.md`.

## Decision

1. **Deleted, not deprecated:** the disabled `jira` and `confluence` external
   MCP servers (`services/mcp/{jira,confluence}.yaml`, `mcp.atlassian.com/v1`),
   the per-instance 3LO app config (`services/web/config/atlassian.yaml`), the
   OAuth handlers and mint service behind `/admin/auth/atlassian/*`,
   `/admin/api/profile/atlassian/unlink` and `/api/public/atlassian/token`, the
   `atlassian_user_credentials` table (migration 048 drops it), the
   `atlassian_client_secret` secret, and the `atlassian: linked` subject
   dimension with its `services/access-control/atlassian.yaml` grants.
2. **One Atlassian server, `atlassian`** (`services/mcp/atlassian.yaml`,
   `mcp.atlassian.com/v2`), authenticated per user by the connector stack
   (`/api/public/connectors/atlassian/token`, `mcp_connector_credentials`,
   consent on `/admin/profile#connected-accounts`). The knowledge bank, the
   admin project report and every BA skill read and write through it.
3. **Readiness is not a subject rule.** The marketplace filter already hides
   `mcp_server: atlassian` until the caller's connector account is configured,
   entitled, verified and connected. A `linked` rule would have made an
   unconnected user un-entitled and therefore unable to connect at all.
4. **Skills name capabilities, not Rovo tools.** The 34 India skills describe
   Jira and Confluence operations as `discover`, then `executeRead` or
   `executeWrite` against the live schema; `tests/unit/web/src/india_skills.rs`
   rejects the legacy tool names and `scripts/india/import-skills.py` emits the
   same wording.

## Why

- Two credential paths for one upstream is the dual-path debt the branch rules
  forbid, and the v1 path was never enabled: its own header said so, pending an
  OAuth app that the connector made unnecessary.
- The v1 token endpoint and table were dead weight that still had to be
  rewrapped on every key rotation and still occupied three admin routes.
- Nothing user-facing changes: no user ever linked through v1, and the
  connected-accounts flow is untouched.
