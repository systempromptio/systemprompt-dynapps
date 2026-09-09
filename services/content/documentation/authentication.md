---
title: "Authentication"
description: "The one way into the platform: Astound single sign-on (ADFS over SAML 2.0, with AD groups deciding roles). Covers operator recovery from the CLI, sessions, JWTs, and route protection."
author: "Astound Digital"
slug: "authentication"
keywords: "authentication, login, sso, adfs, active directory, saml, session, JWT, security, provisioning"
kind: "guide"
public: true
tags: ["authentication", "security", "login"]
published_at: "2026-03-02"
updated_at: "2026-08-31"
after_reading_this:
  - "Sign in as an Astound user with Astound SSO"
  - "Understand how Active Directory groups become platform roles and entitlements"
  - "Recover operator access from the CLI when SSO is unavailable"
  - "Understand how session cookies and JWTs govern authenticated access"
  - "Know which admin routes are public and which require a session"
related_playbooks:
  - title: "Set Up Astound SSO (AD FS)"
    url: "/documentation/adfs-sso-setup"
  - title: "User & Access Management"
    url: "/documentation/enterprise-user-access"
  - title: "Create & Manage Users"
    url: "/documentation/admin-user-management"
  - title: "Install the Desktop Bridge"
    url: "/documentation/bridge-install"
---

# Authentication

**TL;DR:** There is exactly one way into the platform in a browser, and it
stores no password. Everyone signs in with **Astound SSO** — the corporate
AD FS farm, over SAML 2.0 — and their **Active Directory groups** decide what
they can do. There is no second login form, no self-registration, and no
in-browser invite redemption; operator recovery happens from the **CLI**.

The directory is the authority. A `Systemprompt-*` group in Active Directory
is the entitlement; the platform reads it at every sign-in and never stores a
credential of its own for SSO users. The desktop bridge is not an exception:
linking a device sends you through the same browser sign-in, and the link code
it returns is only minted for a session SSO already created.

Being the authority means the directory can also take things away. A sign-in
refused because the AD group is gone does not merely fail — it revokes the
sessions and bridge credentials that sign-in's owner already holds. See
[Deprovisioning](#deprovisioning).

## Path 1 — Astound Users: Astound SSO

This is how everyone who is not a platform operator signs in.

The user clicks **Sign in with Astound SSO** on `/admin/login` (or picks
this application on the AD FS portal). The platform sends a SAML 2.0
`AuthnRequest` to AD FS, so the corporate login policies — Windows
authentication, forms, MFA — apply unchanged, and AD FS posts a signed
assertion back to `/admin/auth/adfs/acs`.

On that callback, the platform:

1. Verifies the assertion: XML signature against the IdP certificate pinned
   in the committed federation metadata, audience, destination, validity
   window, replay, and — for a sign-in we started — the request it answers
   and the `RelayState` this browser's request minted.
2. Gates the claims: the email must be on an allow-listed Astound domain, and
   the assertion must carry at least one AD group. An assertion with no group
   claim at all gets **no session** — deny by default. A group the platform
   does not map is not a refusal; it simply mints no role.
3. Maps the groups to roles (`services/web/config/adfs.yaml`, `group_roles`).
   This rewrites the *directory* half of the account's role set, so leaving an
   AD group revokes the role at the next sign-in. Roles an admin granted by
   hand are a separate half and survive. A sign-in carrying no mapped group
   signs in as a plain `user`.
4. Resolves the identity — returning login, merge onto an existing account by
   email, or just-in-time provisioning.
5. Places the person in the groups and projects those AD groups map to
   (`services/web/config/groups.yaml`), replacing the directory-sourced
   memberships from their last sign-in and leaving dashboard-granted ones
   alone. An AD group that maps to nothing leaves them in the derived
   `unassigned` group.
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
| `no_group` | The assertion carried no AD group at all |
| `not_provisioned` | No account exists and auto-provisioning is off |
| `error` | Anything else — the server log has the detail |

### Groups as entitlements

Two things read the AD groups:

- **Roles** — `group_roles` in `adfs.yaml` maps a group to `platform_admin`,
  `admin`, `project_manager`, `developer`, `knowledge_worker` and/or `user`. The directory half of the role set
  is re-projected on every sign-in.
- **Membership** — `services/web/config/groups.yaml` maps a group onto a
  platform *group* (who someone is, and what workspace they reach) and a
  *project* (what work their usage is attributed to). Both are database rows
  an admin can also edit from the dashboard.

Groups and projects are the two narrowest bands of the access-control
precedence ladder: user 0 → project 140 → group 150 → atlassian 160 →
salesforce 170 → role 200. A grant at either band out-ranks the role grant
that opens a workspace to every user, and an entity gated there is closed to
everyone outside it. Marketplace grants are authored in each marketplace's own
`access.rules` block; anything else goes in
`services/access-control/groups.yaml` or `projects.yaml`.

Group changes take effect at the next sign-in; mid-session revocation is
bounded by the session TTL.

Linking a desktop bridge is not a second login: the device-link approval runs
inside an SSO session and mints a *device* credential from it.

## Operator Recovery — CLI Only

SSO is the only browser door. That makes the directory a dependency: if the
AD FS farm is unreachable, nobody signs in through the web UI. Recovery is
therefore deliberately out-of-band and does not run through the login page at
all.

Operators are created out-of-band and there is no way to self-provision one.
The self-registration endpoint and the public invite-redemption page have both
been removed — not disabled behind a flag, removed — so no browser request can
create an account. An account comes into existence only when a signed assertion
carries a mapped `Systemprompt-*` group (see `auto_provision` below).

```bash
# Create the account and grant the admin role
systemprompt admin users create --name "Jane" --email jane@astounddigital.com
systemprompt admin users role promote jane@astounddigital.com admin
```

For access when SSO itself is down, mint a CLI admin session directly against
the database rather than through the web UI. This, `systemprompt admin bridge
issue-code`, and a personal access token minted from an existing admin session
are the platform's **break-glass** credentials: they are retained deliberately
so an AD FS outage is not a lock-out, they are operator tools rather than a
second user-facing login, and every one of them needs either database access or
an SSO session that already exists.

```bash
systemprompt admin session switch production   # or the profile you are recovering
systemprompt admin session login
```

The WebAuthn credential tables and the CLI's
`admin users webauthn generate-setup-token` command still exist for
out-of-band credential enrolment, but **no page in this installation presents a
passkey challenge** — nothing in the browser consumes one. Treat passkeys as
dormant infrastructure, not as a live sign-in path.

Roles are **not** carried in the session token. They are read from the user
record on every request, so promoting or demoting an operator takes effect on
their next request — no sign-out, no waiting for a token to refresh. The OAuth
*scope* minted into the JWT is fixed at issue time, so a change that widens the
scope itself still needs a fresh sign-in.

An operator who also signs in through SSO keeps their CLI-granted roles unless
the directory maps a role for them — an SSO login that maps *no* role is not a
demotion.

## Configuring SSO

`services/web/config/adfs.yaml` carries the relying party: our entity id, the
ACS URL, the path of the committed IdP federation metadata (which pins the
signing certificate — rotation is a re-fetch and a reviewable diff), the
allow-listed domains, the claim URIs, and the group→role map. There is no
client secret.

**[Set Up Astound SSO (AD FS)](/documentation/adfs-sso-setup)** is the full
procedure: what your IT team registers in AD FS, the three claim issuance
rules, every config key, how groups become roles and per-entity entitlements,
certificate rotation, and pointing an instance at a different farm.

The deployed trust and group mapping are configured in
`services/web/config/adfs.yaml`.

## Deprovisioning

Removing someone from their `Systemprompt-*` group is the whole deprovision
step. There is nothing else to click.

Blocking the next sign-in would not be enough on its own: a cookie session and a
linked bridge both outlive the assertion that created them, so a directory
change that only guarded the front door would leave a working desktop app behind
it. So when a sign-in is refused for a missing group or an off-list domain, and
when a returning sign-in's groups no longer map to `admin`, the platform revokes
that user's:

- browser sessions (`user_sessions`),
- bridge PATs (`user_api_keys`) — the desktop app must be linked again,
- enrolled device certificates,
- any unredeemed bridge device-link code.

Rows are marked revoked, never deleted: what someone held and when it was taken
away is part of the audit trail.

One caveat worth knowing when you are timing a removal: a gateway token already
minted stays valid for its hour. Revocation stops the *next* one, so treat "the
bridge is dead" as true within the hour, not within the second. For an immediate
cut, close the account.

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
project into a `UserContext`. **Auth check middleware** rejects protected
routes without a valid user ID, returning HTTP 401.

`UserContext` carries `user_id`, `username`, `email`, `roles`, `project`, and
`is_admin`.

To sign out, clear the `access_token` cookie.

## Public vs. Protected Routes

| Route | Access |
|-------|--------|
| `/admin/login` | Public |
| `/admin/auth/adfs/*` | Public — the SSO start and callback |
| `/admin/profile`, `/admin/settings`, `/admin/setup` | Any valid session, including a plain `user` |
| `/admin/*` (everything else) | Requires a valid session **and** the `admin` role |
| `/admin/enterprises*`, `/admin/reports/internal` | Requires platform admin |
| `/bridge-auth/*` | Requires a valid session — the desktop link runs through SSO |

`/admin/login` and the two SSO routes are the *entire* public surface. There is
no public registration, credential-enrolment, or invite-redemption route; if you
are adding one, you are adding a second way in.

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
