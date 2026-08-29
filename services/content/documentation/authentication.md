---
title: "Authentication"
description: "The two ways into the platform: Astound single sign-on (ADFS over SAML 2.0, with AD groups deciding roles) and operator passkeys. Covers sessions, JWTs, and route protection."
author: "Astound Digital"
slug: "authentication"
keywords: "authentication, login, sso, adfs, active directory, saml, passkey, webauthn, session, JWT, security, provisioning"
kind: "guide"
public: true
tags: ["authentication", "security", "login"]
published_at: "2026-03-02"
updated_at: "2026-08-28"
after_reading_this:
  - "Sign in as an organization user with Astound SSO"
  - "Understand how Active Directory groups become platform roles and entitlements"
  - "Provision a platform operator from the CLI and enrol their passkey"
  - "Understand how session cookies and JWTs govern authenticated access"
  - "Know which admin routes are public and which require a session"
related_playbooks:
  - title: "User & Access Management"
    url: "/documentation/enterprise-user-access"
  - title: "Create & Manage Users"
    url: "/documentation/admin-user-management"
  - title: "Install the Desktop Bridge"
    url: "/documentation/bridge-install"
---

# Authentication

**TL;DR:** There are two ways into the platform, and neither creates or
stores a password. Organization users sign in with **Astound SSO** — the
corporate AD FS farm, over SAML 2.0 — and their **Active Directory
groups** decide what they can do. Platform operators are created from the
**CLI** and sign in with a **passkey**.

The directory is the authority. A `Systemprompt-*` group in Active Directory
is the entitlement; the platform reads it at every sign-in and never stores a
credential of its own for SSO users.

## Path 1 — Organization Users: Astound SSO

This is how everyone who is not a platform operator signs in.

The user clicks **Sign in with Astound SSO** on `/admin/login` (or picks
this application on the AD FS portal). The platform sends a SAML 2.0
`AuthnRequest` to AD FS, so the corporate login policies — Windows
authentication, forms, MFA — apply unchanged, and AD FS posts a signed
assertion back to `/admin/auth/adfs/callback`.

On that callback, the platform:

1. Verifies the assertion: XML signature against the IdP certificate pinned
   in the committed federation metadata, audience, destination, validity
   window, replay, and — for a sign-in we started — the request it answers
   and the `RelayState` this browser's request minted.
2. Gates the claims: the email must be on an allow-listed Astound domain, and
   the assertion must carry at least one AD group the platform maps. An
   assertion with no mapped group gets **no session** — deny by default.
3. Maps the groups to roles (`services/web/config/adfs.yaml`, `group_roles`)
   and writes them onto the account — replacing, not adding to, whatever the
   account held. Leaving an AD group revokes the role at the next sign-in.
4. Resolves the identity — returning login, merge onto an existing account by
   email, or just-in-time provisioning against the organization's seat
   allocation.
5. Records the mapped groups for the `adfs_group` access dimension (below).
6. Mints a session JWT and sets it as the `access_token` cookie.

Failures return to `/admin/login?sso=<reason>`:

| Reason | Meaning |
|---|---|
| `unavailable` | No usable `adfs.yaml` (or unreadable IdP metadata) on this instance |
| `denied` | AD FS refused or the user cancelled |
| `no_subject` | The assertion carries no `NameID` — the relying-party trust needs a Name ID claim |
| `invalid_assertion` | Signature, audience, destination, validity, or replay check failed |
| `no_email` | The assertion carried no email attribute and no email-format `NameID` |
| `forbidden` | The email domain is not allow-listed |
| `no_group` | The assertion carried no `Systemprompt-*` group the platform maps |
| `not_provisioned` | No account exists and auto-provisioning is off |
| `seat_limit` | The organization's plan has no free seat |
| `error` | Anything else — the server log has the detail |

### Groups as entitlements

Two things read the AD groups:

- **Roles** — `group_roles` in `adfs.yaml` maps a group to `admin` and/or
  `user`. Roles are re-projected on every sign-in.
- **The `adfs_group` access dimension** — entity grants in
  `services/access-control/adfs.yaml` confine a plugin, MCP server, or route
  to members of a named group. It sits between department and role in the
  precedence ladder (user 0 → department 100 → adfs_group 150 → role 200 →
  organization 300), so a group rule out-ranks the role grant that opens the
  marketplace to every user, and an entity listed there is closed to everyone
  outside its groups.

Group changes take effect at the next sign-in; mid-session revocation is
bounded by the session TTL.

All sign-in paths link a desktop bridge equally: the device-link approval
resumes after either an SSO or a passkey sign-in.

## Path 2 — Platform Operators: CLI + Passkey

Operators are created out-of-band. There is no way to self-provision one, and
the passkey self-registration door on the login page ships closed
(`allow_self_registration: false` in `adfs.yaml`).

```bash
# 1. Create the account
systemprompt admin users create --name "Jane" --email jane@astounddigital.com

# 2. Grant the admin role
systemprompt admin users role promote jane@astounddigital.com admin

# 3. Mint a one-shot passkey setup link
systemprompt admin users webauthn generate-setup-token --email jane@astounddigital.com
```

The third command prints a copy-paste URL of the form
`{api_external_url}/auth/link-passkey?token=…`, valid for 15 minutes by default
(`--expires-minutes` to change). Send it to the operator through a channel you
already trust; they open it, create a passkey, and sign in with that passkey
from then on.

An operator who also signs in through SSO keeps their CLI-granted roles
unless the directory maps a role for them — an SSO login that maps *no*
role is not a demotion.

Roles are **not** carried in the session token. They are read from the user
record on every request, so promoting or demoting an operator takes effect on
their next request — no sign-out, no waiting for a token to refresh. The OAuth
*scope* minted into the JWT is fixed at issue time, so a change that widens the
scope itself still needs a fresh sign-in.

### Passkey Sign-In

Passkey authentication uses public-key cryptography. The browser generates a key
pair bound to this domain; the private key stays on the device or in the user's
password manager. The server stores only the public key, verifies a signed
challenge, and issues an OAuth 2.0 session token via PKCE.

### Lost Passkey

There is no self-service recovery. An operator who loses passkey access needs
another operator to mint a fresh setup link with the same
`generate-setup-token` command. Keep more than one operator account so this is
never a single point of failure.

## Configuring SSO

`services/web/config/adfs.yaml` carries the relying party: our entity id,
the ACS URL, the path of the committed IdP federation metadata (which pins
the signing certificate — rotation is a re-fetch and a reviewable diff), the
allow-listed domains, the claim URIs, and the group→role map. There is no
client secret. The full decision record is in
`docs/decisions/2026-08-28-adfs-sso-replaces-salesforce.md`.

## Session Management

| Property | Value |
|----------|-------|
| **Cookie name** | `access_token` |
| **Token format** | JWT |
| **Default expiry** | 3600 seconds (1 hour) |
| **Cookie flags** | `path=/`, `HttpOnly`, `SameSite=Lax`, `Secure` on HTTPS |
| **Required scopes** | `user` or `admin` |

Every admin request passes through two middleware layers. **User context
middleware** extracts and validates the JWT, then loads the user's roles and
department into a `UserContext`. **Auth check middleware** rejects protected
routes without a valid user ID, returning HTTP 401.

`UserContext` carries `user_id`, `username`, `email`, `roles`, `department`, and
`is_admin`.

To sign out, clear the `access_token` cookie.

## Public vs. Protected Routes

| Route | Access |
|-------|--------|
| `/admin/login` | Public |
| `/admin/auth/adfs/*` | Public — the SSO start and callback |
| `/auth/link-passkey` | Public — consumes a one-shot setup token |
| `/admin/profile`, `/admin/settings`, `/admin/setup` | Any valid session, including a plain `user` |
| `/admin/*` (everything else) | Requires a valid session **and** the `admin` role |
| `/admin/enterprises*`, `/admin/reports/internal` | Requires platform admin |
| `/bridge-auth/*` | Requires a valid session |

Anonymous requests to a protected route are redirected to
`/admin/login?redirect=…`. A signed-in user **without** the `admin` role is not
shown a 403 for console pages — they are redirected to `/admin/profile`, which
is the only part of the console addressed to them. JSON admin API routes return
HTTP 403 instead, and the platform-admin routes return an HTML 403.

## System-originated actions

Every action recorded by the platform — including scheduled jobs, hooks, and MCP-server invocations — traces to a real `users` row. There is no separate "system user" or synthesized principal. The platform refuses to attribute work to an invented identity.

### How ownership is declared

Each scheduled job in `services/scheduler/config.yaml` carries an explicit `owner:` field naming an existing admin user:

```yaml
- name: publish_pipeline
  extension: web
  owner: admin
  schedule: "0 */15 * * * *"
  enabled: true
```

At startup the scheduler resolves `owner:` to a `users.id`. If the named user does not exist or is inactive, startup fails loudly — the platform refuses to run with unowned jobs. To change ownership, edit the YAML and restart.

### How attribution flows

The resolved owner becomes `JobContext.actor` for every `execute()` call. Job implementations consume it through `ctx.actor()` and pass it to any audit-row write. Governance audit rows carry three fields that together give full forensic clarity:

| Column | Meaning |
|--------|---------|
| `user_id` | The accountable principal — a real `users.id`. |
| `actor_kind` | The surface that ran the action: `user`, `job`, `mcp`. |
| `actor_id` | A label for that surface (job name, MCP server name, etc.). |

A direct human action shows as `(user_id = alice, actor_kind = 'user', actor_id = 'alice')`. A scheduled job owned by Alice shows as `(user_id = alice, actor_kind = 'job', actor_id = 'publish_pipeline')`. Same accountability column, different surface, queryable separately:

```sql
SELECT actor_kind, user_id, COUNT(*)
FROM governance_decisions
GROUP BY actor_kind, user_id;
```

### Why no separate "system" user

A dedicated "system" identity would be either a synthesized principal (impersonation) or a backdoor account with no real human accountability. Neither passes the "every action traces to a real user" bar. The designated owner is a normal admin who legitimately authorized the platform's existence by installing it — same accountability model as a unix crontab. Compromising the designated owner is exactly as bad as compromising that admin's credentials directly; there is no additional power and no amplification path.
