---
title: "Create & Manage Users"
description: "Every way to create and manage users beyond SSO: admin CLI, bulk creation, operator provisioning, roles, and conversational admin via the systemprompt MCP server."
author: "Dynapps"
slug: "admin-user-management"
keywords: "users, create user, bulk users, roles, role table, entitlement roles, marketplace participation, promote, admin, user management, systemprompt mcp, csv"
kind: "guide"
public: true
tags: ["documentation", "admin", "users"]
published_at: "2026-08-19"
updated_at: "2026-09-29"
after_reading_this:
  - "Create a user from the CLI and get them connected"
  - "Bulk-create users from a CSV"
  - "Grant and revoke the admin role"
  - "Tell console roles from entitlement roles, and what marketplace participation opens"
  - "Manage users conversationally through the systemprompt MCP server"
related_playbooks:
  - title: "Authentication"
    url: "/documentation/authentication"
  - title: "Install the Desktop Bridge"
    url: "/documentation/bridge-install"
  - title: "User & Access Management"
    url: "/documentation/enterprise-user-access"
---

# Create & Manage Users

Dynapps SSO is one way accounts come to exist — not the only way. This page
covers the paths that need no SSO at all: the admin CLI, bulk creation,
roles, and the conversational admin workflow.

## Account provisioning

1. **Admin CLI** — an operator creates the account directly (below). Full
   control, scriptable, works headless.
2. **Dynapps SSO just-in-time** — first sign-in creates the account, with the
   role and project the Entra groups map to, provided the assertion passes the domain and group-claim gates and `auto_provision` is on. See [Authentication](/documentation/authentication).

Public self-registration, passkey login, and browser invite redemption are not available in this installation. See [Authentication](/documentation/authentication) for operator recovery.

## Create one user, end to end

```bash
# 1. Create the account (idempotent with --if-not-exists)
systemprompt admin users create --name jane --email jane@example.com \
  --full-name "Jane Doe" --if-not-exists

# 2. Optional: make them an admin
systemprompt admin users role promote jane@example.com

# 3. Issue a bridge exchange code so they can connect their laptop
systemprompt admin bridge issue-code --user-id jane@example.com
```

Send the code through a channel you trust — it is single-use with a 10-minute
TTL. The user then installs the bridge with it:
[Install the Desktop Bridge](/documentation/bridge-install).

Browser access requires Dynapps SSO. The CLI retains WebAuthn administration commands, but this installation has no browser passkey setup or sign-in flow.

## Bulk creation

`--if-not-exists` makes creation idempotent, so a CSV loop is safe to re-run:

```bash
# users.csv contains name,email,full_name rows without a header or quoted commas
while IFS=, read -r name email full_name; do
  systemprompt admin users create --name "$name" --email "$email" \
    --full-name "$full_name" --if-not-exists
done < users.csv
```

Follow with `role promote` for the rows that need it, and `bridge issue-code`
per user as they onboard.

## The full CLI surface

`systemprompt admin users --help` is the authoritative list. The highlights:

| Command | Purpose |
|---------|---------|
| `list` / `show` / `search` / `count` / `stats` | Inspect the user base |
| `create` / `update` / `delete` | Lifecycle |
| `export` | Dump users for reporting or migration |
| `merge` | Combine duplicate accounts |
| `bulk …` | Batch operations |
| `ban …` | Block access |
| `role promote` / `role demote` / `role assign` | Role management |
| `session …` | Inspect and revoke sessions |
| `webauthn …` | Passkey management and setup tokens |
| `apikey` | API key management |

## Roles

This is the one role table for the installation; other pages link here. The
set is closed — `Role` in `extensions/web/shared/admin-core/src/types/role.rs`
owns the spelling — and seven roles ship.

| Role | Kind | Admin console | What it opens |
|---|---|---|---|
| `user` | baseline | No (participation only) | Own profile, history, connectors and connected clients |
| `developer` | entitlement | No (participation only) | Whatever `rules.yaml` allows to `role: [developer]` — engineering marketplaces and MCP servers |
| `knowledge_worker` | entitlement | No (participation only) | Whatever `rules.yaml` allows to `role: [knowledge_worker]` — the Cowork workspace, business skills |
| `project_manager` | entitlement | No (participation only) | Whatever `rules.yaml` allows to `role: [project_manager]`; nothing on the admin plane |
| `super_admin` | entitlement | No | The `dynapps-super-admin` marketplace, its plugin, the admin skills and the `systemprompt` MCP server |
| `admin` | console | Yes, unscoped | Every admin page and write action |
| `platform_admin` | console | Yes, unscoped | Everything `admin` has, plus moving `platform_admin` / `super_admin` |

Only `admin` and `platform_admin` are console roles (`ROLES_CONSOLE`). They
see the whole instance, unscoped. Every other role is an **entitlement role**:
it decides which marketplaces, plugins, skills and MCP servers reach a person
through [Access Control](/documentation/access-control), and opens nothing on
the admin dashboard by itself.

### Console access by marketplace participation

Anyone else reaches the console through **marketplace participation**: when a
marketplace's allow rule in `services/access-control/rules.yaml` names a group
or project the person belongs to, they participate in that marketplace. A
participant gets a scoped console:

- **Analysis → Conversations** and **Analysis → Versions**, narrowed to the
  people in the groups and projects their marketplaces name, and to those
  marketplaces' versions.
- **Code sync** (`/admin/sync`) for the bundles that own their marketplaces,
  with each apply scoped to that marketplace.
- The session, request, trace and conversation detail pages those views link
  to. A detail page outside their view returns 404.

Every other admin route redirects a participant to their profile. A person
who participates in no marketplace sees only their account pages (`/admin/profile`,
`/admin/history`, `/admin/connect`, `/admin/connectors`). The allowlists are
`ACCOUNT_PATHS` and `PARTICIPANT_PATHS` in
`extensions/web/shared/admin-core/src/middleware/gates.rs`.

So a project manager sees the conversations and versions of the marketplaces
their groups and projects participate in — not every project, and never a
write action on users, roles, access rules or gateway configuration. They do
not get the `systemprompt` MCP server, the `dynapps-super-admin` plugin or the
admin skills.

### Where roles come from

Roles have two halves, recomputed together on `users.roles`:

- **Directory half** — projected from Entra groups on **every** SSO sign-in
  (replace, not union), by the same identity policy (`SsoIdentityPolicy`)
  of the OpenID Connect provider. For example,
  `Dynapps-Systemprompt-Admins` → `platform_admin, admin, user` and
  `Dynapps-Systemprompt-Users` → `user`. A sign-in whose groups map to nothing
  gets `default_roles` (`[user]`). The map is `group_roles` and
  `group_role_patterns` (one `*` glob) in
  `services/web/config/entra-oidc.yaml`, keyed by Entra group object id; see
  [Set Up Dynapps SSO (Microsoft Entra ID)](/documentation/entra-id-sso-setup#groups-roles-and-entitlements).
- **Manual half** — granted by an admin on `/admin/roles` or on a user's
  detail page (`/admin/users/<id>`), stored in `user_manual_roles`. Core's
  sign-in writes the directory roles over `users.roles`, and the platform
  re-joins this half immediately afterwards, so a manual grant survives
  sign-in. A directory role cannot be revoked by hand — remove the Entra group
  instead.

The CLI commands below write `users.roles` directly and record no manual
grant. That is right for operators who never sign in through SSO; for anyone
who does, the role is replaced at their next SSO sign-in — grant it from the
dashboard instead.

Only the `admin` role mints the session's `admin` permission
(`permissions_for_roles`); `platform_admin` on its own mints only `user`, so
always pair it with `admin`.

```bash
systemprompt admin users role promote jane@example.com   # grant admin
systemprompt admin users role demote jane@example.com    # revoke admin
systemprompt admin users role assign jane@example.com knowledge_worker
```

Only a platform admin may grant or remove `platform_admin` or `super_admin`,
and the last platform admin cannot be demoted (`handlers/roles.rs`).

Role checks read the user record on every request, so revocation is
immediate. The OAuth *scope* in a session token is fixed when the token is
issued — a freshly promoted admin must **sign out and back in** (and re-link
the bridge) before admin surfaces appear.

## Conversational admin: the systemprompt MCP server

Admins who have connected Claude Code through the bridge get the
`systemprompt` MCP server automatically. It exposes the admin surface as
tools, so user management becomes conversational:

> "Create accounts for these 40 people from this spreadsheet, and give the
> team leads admin roles."

Three properties make this safe:

- **Admin-gated** — a non-admin token gets HTTP 403 on every tool; the server
  does not even appear in their client until promotion (and a re-link).
- **Audited** — every tool call lands in the same governance spine as
  inference calls, with identity, arguments, decision, and trace ID.
- **Same surface** — the tools wrap the same CLI commands documented above;
  there is no separate, wider API.

For non-admins the server is simply absent — nothing to misconfigure.

## The users page

`/admin/users` is the roster. The KPI strip across the top doubles as filters:
**Accounts**, **Enabled**, **Unassigned** (no group), **No role** (no grant
beyond `user`), **Idle 30d** (not seen by any signal — provisioned but unused),
plus 30-day gateway cost and request counts. Every filter exports to CSV.

A user's detail page (`/admin/users/<id>`) holds their roles, groups and
projects, sessions (revoke from here), and a **Usage** tab with their
requests, cost and conversations.

**New user** creates an account from the browser only when no registered SSO
provider has `auto_provision: true` (`entra-oidc.yaml`). This installation
runs with `auto_provision: true` — the Entra group is the invitation — so the
button is disabled and explains why. The CLI and the MCP route above work
either way.
