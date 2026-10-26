---
title: "User & Access Management"
description: "Manage enterprise users from the admin UI: search, roles, session and PAT revocation, closed registration, and Dynapps SSO with Entra groups as entitlements."
author: "Dynapps"
slug: "enterprise-user-access"
keywords: "users, access, roles, sso, entra id, oidc, pat, api key, per-key limits, scope binding, sessions, registration, deprovisioning, time-bound access, expiry"
kind: "guide"
public: true
tags: ["enterprise", "admin", "access"]
published_at: "2026-08-25"
updated_at: "2026-10-08"
after_reading_this:
  - "Manage users end to end from /admin/users without touching the CLI or database"
  - "Provision users through SSO or operator administration"
  - "Understand the closed browser registration and recovery paths"
  - "Revoke sessions and personal access tokens for any user immediately"
  - "Understand how Dynapps SSO provisions accounts and how Entra groups revoke access"
  - "Issue an API key with its own model allowlist, budget, request ceiling and scope bindings"
related_docs:
  - title: "Access Control: Who Reaches What, and Why"
    url: "/documentation/access-control"
  - title: "Authentication"
    url: "/documentation/authentication"
  - title: "Enterprise Roadmap & Known Limitations"
    url: "/documentation/enterprise-roadmap"
---

# User & Access Management

The admin UI at `/admin/users` manages accounts, roles, status, sessions, and personal access tokens. Browser sign-in uses Dynapps SSO; operators can provision and recover access through the CLI. Public registration, browser invite redemption, and passkey login are not offered by this installation.

## Managing users from the admin UI

Open `/admin/users` as an admin. From this one page you can:

- **Search and list** users by name or email, with pagination.
- **Create** a user directly.
- **Edit roles and status** — grant or remove roles by hand (see [Roles](/documentation/admin-user-management#roles)), activate or suspend an account.
- **Disable or delete** an account. Disabling keeps history; deleting removes the account.
- **Revoke sessions** — force sign-out everywhere with one action.
- **Revoke personal access tokens** — every PAT the user holds can be revoked from their detail page, or from `/admin/devices`, which lists tokens across the instance.
- **See last activity** — each user row shows last-active time, and the **Inactive 30d** filter lists accounts with no activity in thirty days, including accounts that were provisioned and never used.

No CLI or database access is required for any of these operations. The CLI equivalent for scripting exists under `systemprompt admin users` (see `systemprompt admin users --help`).

## Provisioning users

Users enter through group-gated SSO when `auto_provision` is enabled, or an operator creates their account. Existing invite and WebAuthn administration primitives do not provide a browser onboarding flow here. Follow [Authentication](/documentation/authentication) and [Create & Manage Users](/documentation/admin-user-management).

## Dynapps SSO and deprovisioning

The login page offers **Sign in with Microsoft**: Microsoft Entra ID over OpenID Connect (provider `entra-oidc`, routes `/admin/auth/entra/start` and `/admin/auth/entra/callback`). It applies core's identity policy, `SsoIdentityPolicy`, declared as top-level keys of `services/web/config/entra-oidc.yaml`:

| Key | Default | Effect |
|---|---|---|
| `allowed_email_domains` | `[]` (any) | Only these email domains may sign in (`?sso=forbidden`) |
| `deny_without_group` | `false` (shipped: `true`) | No group at all, no session (`?sso=no_group`) |
| `group_roles` | `{}` | Exact Entra group object id (from the `groups` claim) → roles |
| `group_role_patterns` | `{}` | Single-`*` glob, such as `Dynapps-Systemprompt-PM-*` → roles, applied after exact matches |
| `default_roles` | `[user]` | Roles when no group maps |
| `auto_provision` | `false` (shipped: `true`) | Create the account on a first sign-in (`?sso=not_provisioned` otherwise) |

Every SSO sign-in sets the account's roles to what the policy resolved joined with the roles granted by hand in the dashboard (the installation's role resolver); a role granted only from the CLI does not survive the next SSO sign-in. A first sign-in attaches to the active local account that already owns the verified address (`link_by_email: true` in the shipped config) rather than creating a duplicate. Directory group and project memberships are stored as directory memberships (`source = 'adfs'` — a historical label, not a protocol) and replaced at each sign-in; console-added memberships are `source = 'manual'` and survive. Users may belong to zero, one, or several projects. See [Set Up Dynapps SSO (Microsoft Entra ID)](/documentation/entra-id-sso-setup) and [Authentication](/documentation/authentication#what-happens-on-a-sign-in).

Deprovisioning follows the directory: disabling the Entra ID account stops the next sign-in, and removing someone from a `Dynapps-Systemprompt-*` group strips the role it granted at their next sign-in. Mid-session revocation is bounded by the session TTL; an admin can revoke sessions and PATs immediately from `/admin/users`.

SCIM provisioning is not offered: the platform has no SCIM endpoint, so Entra ID changes arrive only through the next sign-in. See the [Enterprise Roadmap](/documentation/enterprise-roadmap) for when this changes.

## Roles

Roles come from the directory (the `Dynapps-Systemprompt-*` Entra groups mapped in
`entra-oidc.yaml`) or are granted by hand. Only `admin` and `platform_admin` see the
whole console unscoped. `project_manager` is an **entitlement role**: it
changes what a person reaches in `services/access-control/rules.yaml`, and it
does not open the console. Anyone else, project managers included, reaches
the console only through **marketplace participation**, meaning a group or
project allowed on a marketplace, and sees Analysis and Code sync narrowed to
the people and marketplaces that participation names. The full role table,
with what each role reaches, is in
[Create & Manage Users → Roles](/documentation/admin-user-management#roles).

## Personal access tokens and devices

`/admin/devices` lists every personal access token on the instance: owner, prefix (so a leaked token can be identified without exposing it), expiry, and creation time. Any token can be revoked immediately, and revocation takes effect on the next request. **Issue a token** on the same page mints one for the signed-in account and shows the secret exactly once.

Every token has an expiry. `security.max_pat_lifetime_days` in `services/web/config/security.yaml` (default **90**) is the longest a token may live from issuance, enforced on every path that mints one — the console form, `POST /admin/devices/pats`, and device enrolment. A request with no expiry gets the maximum; one asking for more is refused with a 400 rather than silently clamped. Tokens issued before the policy changed keep the expiry they were issued with.

### Per-key limits and scope bindings

A key (personal access token) can carry its own policy, set when it is issued. Without it, a key inherits its owner's scope and counts against the owner's quota windows only. With it, the key is **its own quota subject** (`api_key`), on top of the owner's windows, so one application's key can be capped without capping the person.

| Limit | Effect |
|---|---|
| Model allowlist | A request for a model not on the list is refused with `403` (`model '<id>' is not allowed for this API key`) before routing. |
| Budget (microdollars) and request ceiling | Ceilings over the key's own window (`request_window_seconds`, required when either is set). They reserve, settle and answer the machine-readable `429` exactly like policy quota windows, and follow the instance's `quota_mode`, which is **warn** here: a crossing is recorded, not refused. |
| Scope bindings | One value per scope dimension (for example `project=<id>`). The key's requests are attributed to that value, with source `api_key`, unless the request names the dimension by header. The value is checked against the owner's memberships at issuance and on every request: a value the owner does not hold is refused with `403`, an unregistered dimension with `400`. |

From the CLI:

```bash
systemprompt admin users api-key issue --user <user> --name ci-runner \
  --model claude-sonnet-5-5 \
  --budget-microdollars 50000000 --max-requests 2000 --window-seconds 86400 \
  --scope project=<project-id>
systemprompt admin users api-key list --user <user>     # prints each key's limits and scopes
```

`--model` and `--scope` repeat. With none of these flags, issuance is unchanged.

From the API, `POST /api/v1/admin/api-keys` accepts `model_allowlist`, `budget_microdollars`, `max_requests`, `request_window_seconds` and `scopes`, with the same membership check, and `GET /api/v1/admin/api-keys` returns them. The **Issue a token** form on `/admin/devices` does not set limits; use the CLI or API for a limited key.

Device certificates can be given an expiry from the certificates tab (**Set expiry**). The window is recorded beside the certificate and the hourly sweep revokes the certificate once it passes.

## Time-bound sharing

Share tokens honour an expiry timestamp, and setup tokens, session JWTs and personal access tokens are all time-bound, so no credential lives forever.

Access itself is time-bound too. A group membership, a project membership, a manual role grant and an access-control rule each carry an optional expiry, set from the console with a date picker — the **Add member** dialog on a group or project, the **Roles** section of a person's page (one window for every role granted by hand), and the rule editor's reason box on a group's or person's Access tab. Membership tables and the `/admin/access-control` ledger show an **Expires** column, badge anything that lapses inside seven days, and the ledger's *Expiring in 7 days* filter lists what needs renewing. Directory-sourced membership and directory-projected roles never carry a window: Entra ID decides when those end.

How an expiry binds differs by what it is on, because two of the four are decided by code this extension does not own:

| What expires | Takes effect | How |
|---|---|---|
| Group or project membership | Immediately | Every membership read, including the `group`/`project` authorization dimensions and the `user_groups` view, hides a row outside its window. The hourly `access_expiry` sweep then stamps the row `revoked_at` so it remains as the audit trail, and recomputes the person's primary scope. |
| Manual role | Within the hour | Roles are enforced from the effective set on the user record, which the sweep rewrites without the expired grant. If that removes a manage role, the sweep revokes the person's live sessions, tokens and certificates — the same teardown a demotion in the role editor performs. |
| Access-control rule | Within the hour | Core's resolver reads the rule table directly, so the sweep deletes the expired rule. The window is set from the console; `rules.yaml` does not carry it, because core applies the file and its dialect has no `valid_until`. |
| Device certificate | Within the hour | The sweep stamps the certificate revoked, which is what the device gate reads. |

Re-adding a member whose earlier membership expired revives the row with the new window. For contractors, combine a membership expiry with the PAT lifetime cap above; account-level disable and revocation remain available for the immediate case.

### Archive instead of delete

Deleting a person archives them (NFR-5.2). The account stops signing in and leaves the roster. Their sessions, tokens and certificates are revoked at once, and who archived them, when and why is recorded. Their requests, decisions and usage stay. **Restore** on the person's page, or `systemprompt admin users restore <user>`, brings the account back within `retention.archived_users_days` (90 days by default). After that window, the nightly cleanup purges the account and its rows, unless it is under legal hold (`systemprompt admin users legal-hold <user>`). See [Data Retention](/documentation/data-retention#archived-users).

## Context-aware access control

What a session can reach — marketplaces, plugins, skills, MCP servers, gateway routes — is decided per request from the person's **current context**: their role, the groups and projects the last SSO sign-in placed them in, and the accounts they have linked. The narrowest band that names them decides, a deny beats an allow inside it, and an entity with rules is closed to everyone they do not name. Every rule is declared once in `services/access-control/rules.yaml` with a stated reason and enforced from the database; `/admin/access-control` shows who reaches what and why, and its Sync page reconciles code and console in either direction. The full model, worked examples and how-tos are in [Access Control](/documentation/access-control).

## Check it yourself

- `/admin/users`: the roster, with search, roles, status and last activity.
  Try the **Inactive 30d** filter.
- `/admin/users/<id>`: role editing, session and token revocation, and
  memberships with their expiry.
- `/admin/devices`: every personal access token and device certificate, with
  expiry and a revoke action.
- `/admin/access-control`: the ledger of who reaches what, with the
  *Expiring in 7 days* filter.
- `systemprompt admin users list`
- `systemprompt admin users show <email>`
