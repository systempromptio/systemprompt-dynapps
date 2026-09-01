# REQ-044 — User Awareness: evidence that this is largely Supported, not a Gap

Status: evidence response for the register (row 45, marked "Gap — stakeholder
identified"). Recommendation: re-status to "Supported with caveat (project
dimension pending DEC-001)" after a demo of the matrix below.

## The requirement

> Enforce dynamic access control based on the user's active session context —
> project, role, and permissions — determining which MCP servers, models, and
> knowledge sources the user can access.

## What already exists

Every session is authorized through a single rule table
(`access_control_rules`, core `crates/infra/security/schema/`) evaluated by a
deny-overrides, narrowest-band-wins resolver
(`crates/infra/security/src/authz/resolver.rs`). It gates exactly the entity
classes the requirement names:

| Requirement asks | Enforced entity type | Where checked |
|---|---|---|
| MCP servers | `mcp_server` | runtime authz webhook + signed bridge manifest |
| Models | `gateway_route` | gateway dispatch (`gateway_entities.rs`) |
| Knowledge sources | `plugin` / `skill` / `mcp_server` | same resolver |

Subject dimensions on this instance, by precedence band (lower = wins):

| Band | Dimension | Source |
|---|---|---|
| 0 | user | per-user override rows |
| 150 | adfs_group (derived, multi-valued) | `adfs_group_memberships` rows written at SSO sign-in, 60s cache |
| 200 | role | `users.roles` + `roles.yaml` |

*(Superseded 2026-09-01: the `department` (100) and `organization` (300)
bands were removed with the multi-organization model. A user's project —
Commerce or Core — is an AD group, so it is expressed through `adfs_group`.)*

Properties matching the acceptance criteria:

- **Different contexts → different authorized sets.** Two users with different
  role/AD-group/linked state resolve different entity sets. The
  admin "effective access" matrix
  (`extensions/web/admin/src/repositories/users/access_control/matrix.rs`,
  `repositories/governance/effective.rs`) renders this per user.
- **Unauthorized resources are not discoverable.** Denied entities are removed
  from the signed bridge manifest (`marketplace_filter.rs`) — the client never
  sees them — and are refused again at call time by the runtime resolver, so
  discovery and callability are both gated.
- **Decisions are auditable.** Every decision lands in the governance spine
  with policy + trace id (`systemprompt infra logs trace list`); denials are
  recorded like any other call.

Live example on this instance: the `adfs_group` derived dimension confines any
entity listed in `services/access-control/adfs.yaml` to members of the named
`Systemprompt-*` AD groups, in both the manifest and at runtime.
(Superseded the `salesforce: linked` dimension on 2026-08-28.)

## The project axis (closed 2026-09-01)

DEC-001 landed: a user belongs to exactly one project, Commerce or Core,
decided by the AD groups `Systemprompt-Commerce` / `Systemprompt-Core` at
sign-in (`group_projects` in `services/web/config/adfs.yaml`). It is a
reporting split — every dashboard, listing and report is scoped by it
(`util/project_scope.rs`) — and, as an AD group, already an entitlement
boundary through the `adfs_group` dimension. No separate dimension was added.

## Verification to add now (register's own method)

Role/project/session matrix + negative access-control test: an e2e/contract
matrix that, for a set of principals differing in role and AD-group
membership, asserts (a) the manifest/entity lists differ as
configured, (b) a denied MCP server / gateway route returns a deny that is
audited, (c) no denied entity appears in discovery surfaces.
