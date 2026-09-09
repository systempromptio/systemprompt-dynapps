# Access audit — 2026-09-09

Scope: repository-owned dashboard, browser sign-in, bridge identity, connector
authentication, marketplace listing, governance and knowledge-bank access checks.
Static source review; no live endpoint or exhaustive core audit.

## Corrected

Five admin-role checks recognized only `admin`, despite the shared admin-plane
policy and services/access-control/roles.yaml explicitly recognizing
`platform_admin` too. They now call `roles_grant_manage`:

- handlers/adfs_auth/session.rs: platform-admin-only sign-ins received User
  permissions without Admin permissions.
- handlers/bridge_whoami.rs: the same accounts were reported as non-admin.
- repositories/marketplace/hooks.rs: disabled and role-restricted hooks were
  hidden from platform administrators.
- repositories/marketplace/plugins/overview.rs: both counts and plugin listings
  applied non-admin filtering to platform administrators.

Paths above are under extensions/web/admin/src. Regression cases cover
platform-admin token permissions, ordinary roles remaining non-admin, and hook
visibility. The earlier connector marketplace/group restriction is absent from
current source.

## Additional findings and implemented fixes

1. **Gateway authorization fails open on database errors.**
   extensions/web/admin/src/gateway_entitlement.rs explicitly allows a request
   when entity/rule/role lookup fails. This is documented as an availability
   policy, but a database failure can bypass a configured route denial.
   Fixed: lookup errors return a retryable 503; pending approval does not permit execution.

2. **Governance can lose marketplace restrictions after lookup failure.**
   extensions/web/admin/src/handlers/webhook/governance/authz.rs replaces failed
   marketplace rule reads with empty rules, or failed entity listing with no
   parents, and caches that result. Depending on defaults, resolution can differ
   from the intended policy. Fixed: all attribute and parent-policy reads propagate
   errors, and hooks return an explicit deny envelope with an audit attempt.

3. **Legacy listings do not use the complete access resolver.**
   repositories/marketplace/plugins/overview.rs filters by plugin.roles alone.
   handlers/plugins.rs, handlers/resources.rs and marketplace_context.rs consume
   this view. Group, project and per-user grants/denials can therefore disagree
   with plugin listings, agent listings and counts. This is a listing mismatch;
   it does not by itself prove an execution authorization bypass. Fixed: personal
   lists and deduplicated counts use current subject attributes and configured
   parent chains. The shared unkeyed sidebar cache has been removed.

4. **Knowledge-bank administrative tools still require literal admin.**
   extensions/mcp/knowledge-bank/src/server/tool.rs::require_admin rejects a
   platform_admin-only principal. The server ACL allows platform_admin, but the
   tool gate does not. Fixed: the web and knowledge-bank gates share the same
   admin/platform_admin role helper.

5. **DB-derived governance scope requires literal user for non-admins.**
   handlers/webhook/governance/scope.rs yields Unknown for developer,
   project_manager or knowledge_worker without user. Browser tokens compensate
   by carrying Permission::User, but callers relying on stored roles can get a
   different result. Fixed: every active account gets User scope, while admin
   scope follows current database roles. Stale token roles cannot restore revoked
   privilege; missing/inactive accounts cannot execute.

6. **Profile connector inventory is fixed to three providers.**
   services/connector_oauth/config.rs::Provider::ALL includes only Atlassian,
   GitHub and Salesforce. A newly configured OAuth MCP server will not appear
   automatically. Provider-specific verification/credential handling means this
   needs an extension mechanism, not merely removal of a filter. Fixed: enabled
   configured servers populate the profile, with generic OAuth discovery, PKCE,
   trusted origins, refresh and MCP verification. Migration 051 preserves existing
   provider grants. See [configuration guide](CONFIGURED-CONNECTORS.md).

## Reviewed constraints that are not the original bug

- Marketplace audiences belong in services/marketplaces and access-control YAML;
  their group rules govern those resources, not personal account authentication.
- ADFS email-domain defaults are configurable, and mapped-group provisioning is
  an explicit sign-in policy.
- Atlassian/Salesforce hostname validation constrains credential destinations;
  it is not an India marketplace requirement.
- Active-account, same-origin, OAuth state and token-audience checks protect
  authentication and should remain.

## Validation

- Unit suite: 926 passed, zero skipped, including generic TLS OAuth exchange,
  refresh and MCP verification, trusted-origin rejection, authorization lookup
  failure, and an explicit HTTP 200 deny envelope from the governance hook.
- Bridge profile JavaScript: four tests passed, including a fourth connector and
  a server requiring no personal authorization.
- Database integration validation is in progress; fixture initialization was
  updated to provide the services configuration now required by strict reads.
- Full preflight currently stops on formatting in concurrent core evaluation
  changes. Source gates additionally report evaluation repository construction.
- Test-environment deployment preflight passed with existing keys preserved;
  deployment and endpoint verification are pending.
