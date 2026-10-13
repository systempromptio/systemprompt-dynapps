---
title: "Authentication"
description: "The one way into the platform: Dynapps single sign-on (Microsoft Entra ID over SAML 2.0, with Entra groups deciding roles). Covers operator recovery from the CLI, sessions, JWTs, and route protection."
author: "Dynapps"
slug: "authentication"
keywords: "authentication, login, sso, entra id, saml, session, JWT, security, provisioning"
kind: "guide"
public: true
tags: ["authentication", "security", "login"]
published_at: "2026-03-02"
updated_at: "2026-10-13"
after_reading_this:
  - "Sign in as a Dynapps user with Dynapps SSO"
  - "Understand how Entra ID groups become platform roles and entitlements"
  - "Recover operator access from the CLI when SSO is unavailable"
  - "Understand how session cookies and JWTs govern authenticated access"
  - "Know which admin routes are public and which require a session"
related_playbooks:
  - title: "Set Up Dynapps SSO (Microsoft Entra ID)"
    url: "/documentation/entra-id-sso-setup"
  - title: "User & Access Management"
    url: "/documentation/enterprise-user-access"
  - title: "Create & Manage Users"
    url: "/documentation/admin-user-management"
  - title: "Install the Desktop Bridge"
    url: "/documentation/bridge-install"
---

# Authentication

**TL;DR:** There is exactly one way into the platform in a browser, and it
stores no password. Everyone signs in with **Dynapps SSO** (Microsoft Entra
ID, over SAML 2.0) and their **Entra groups** decide what
they can do. There is no second login form, no self-registration, and no
in-browser invite redemption; operator recovery happens from the **CLI**.

The directory is the authority. A `Dynapps-Systemprompt-*` group in Entra ID
is the entitlement; the platform reads it at every sign-in and never stores a
credential of its own for SSO users. The desktop bridge is not an exception:
linking a device sends you through the same browser sign-in, and the link code
it returns is only minted for a session SSO already created.

Being the authority means the directory can also take things away. A sign-in
refused because the Entra group is gone does not merely fail — it revokes the
sessions and bridge credentials that sign-in's owner already holds. See
[Deprovisioning](#deprovisioning).

## Path 1 — Dynapps Users: Dynapps SSO

This is how everyone who is not a platform operator signs in.

The user clicks **Sign in with Dynapps SSO** on `/admin/login` (or picks
the application tile in My Apps). The platform sends a SAML 2.0
`AuthnRequest` to Entra ID, so Conditional Access and MFA apply
unchanged, and Entra ID posts a signed
assertion back to `/admin/auth/adfs/acs`.

On that callback, the platform:

1. Verifies the assertion: XML signature against the IdP certificate pinned
   in the committed federation metadata, audience, destination, validity
   window, replay, and — for a sign-in we started — the request it answers
   and the `RelayState` this browser's request minted.
2. Gates the claims: the email must be on an allow-listed Dynapps domain, and
   the assertion must carry at least one Entra group. An assertion with no group
   claim at all gets **no session** — deny by default. A group the platform
   does not map is not a refusal; it simply mints no role.
3. Maps the groups to roles (`services/web/config/adfs.yaml`, `group_roles`).
   This rewrites the *directory* half of the account's role set, so leaving an
   Entra group revokes the role at the next sign-in. Roles an admin granted by
   hand are a separate half and survive. A sign-in carrying no mapped group
   signs in as a plain `user`.
4. Resolves the identity — returning login, merge onto an existing account by
   email, or just-in-time provisioning.
5. Places the person in the groups and projects those Entra groups map to
   (`services/web/config/groups.yaml`), replacing the directory-sourced
   memberships from their last sign-in and leaving dashboard-granted ones
   alone. An Entra group that maps to nothing leaves them in the derived
   `unassigned` group.
6. Mints a session JWT and sets it as the `access_token` cookie.

Failures return to `/admin/login?sso=<reason>`:

| Reason | Meaning |
|---|---|
| `unavailable` | No usable `adfs.yaml` (or unreadable IdP metadata) on this instance |
| `correlation_lost` | The sign-in state cookie was missing or expired |
| `no_subject` | The assertion carries no `NameID` — the Enterprise Application needs a Name ID claim |
| `invalid_assertion` | Signature, audience, destination, validity, or replay check failed |
| `no_email` | The assertion carried no email attribute and no email-format `NameID` |
| `forbidden` | The email domain is not allow-listed |
| `no_group` | The assertion carried no group at all |
| `group_overage` | Entra ID sent a group overage link instead of the group list |
| `not_provisioned` | No account exists and auto-provisioning is off |
| `error` | Anything else — the server log has the detail |

### Groups as entitlements

Two things read the Entra groups:

- **Roles** — `group_roles` in `adfs.yaml` maps a group to `platform_admin`,
  `admin`, `project_manager`, `developer`, `knowledge_worker` and/or `user`
  (`group_role_patterns` adds glob matches such as
  `Dynapps-Systemprompt-PM-*`). The directory half of the role set is
  re-projected on every sign-in. What each role opens is on
  [Create & Manage Users → Roles](/documentation/admin-user-management#roles).
- **Membership** — `services/web/config/groups.yaml` maps a group onto a
  platform *group* (who someone is, and what workspace they reach) and a
  *project* (what work their usage is attributed to). Both are database rows
  an admin can also edit from the dashboard.

Groups and projects are the two narrowest shared bands of the access-control
precedence ladder: person 0 → project 140 → group 150 → linked account 170 →
role 200. The narrowest band that names a person decides, and an entity with
rules is closed to everyone they do not name. Every grant — marketplaces
included — is declared in `services/access-control/rules.yaml`; see
[Access Control](/documentation/access-control).

A directory change is observed at the **next sign-in**, and until then an
existing session keeps what it was given, bounded by the session TTL
(`security.jwt_access_token_expiration`; 30 days on the production profile).
The exceptions are the ones that revoke: losing the group that grants `admin`
and a refused sign-in both tear down the person's credentials, and disabling
the account from the console is enforced on every JWT-bearing request within
about 30 seconds. The full table is in
[Lifecycle](/documentation/entra-id-sso-setup#lifecycle-joiners-movers-and-leavers).
There is no SCIM endpoint and no single logout, so nothing observes a
change between sign-ins.

Linking a desktop bridge is not a second login: the device-link approval runs
inside an SSO session and mints a *device* credential from it.

## Operator Recovery — CLI Only

SSO is the only browser door. That makes the directory a dependency: if
Entra ID is unreachable, nobody signs in through the web UI. Recovery is
therefore deliberately out-of-band and does not run through the login page at
all.

Operators are created out-of-band and there is no way to self-provision one.
The self-registration endpoint and the public invite-redemption page have both
been removed — not disabled behind a flag, removed — so no browser request can
create an account. Accounts are provisioned by operators or by SSO when a signed assertion passes the configured domain and group-claim gates and `auto_provision` is enabled.

```bash
# Create the account and grant the admin role
systemprompt admin users create --name "Jane" --email jane@dynapps.es
systemprompt admin users role promote jane@dynapps.es admin
```

For access when SSO itself is down, mint a CLI admin session directly against
the database rather than through the web UI. This, `systemprompt admin bridge
issue-code`, and a personal access token minted from an existing admin session
are the platform's **break-glass** credentials: they are retained deliberately
so an Entra ID outage is not a lock-out, they are operator tools rather than a
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
record on every request through a 30-second cache, so promoting or demoting an
operator takes effect within half a minute — no sign-out, no waiting for a
token to refresh. The OAuth
*scope* minted into the JWT is fixed at issue time, so a change that widens the
scope itself still needs a fresh sign-in.

An operator who also signs in through SSO keeps their CLI-granted roles unless
the directory maps a role for them — an SSO login that maps *no* role is not a
demotion.

## Configuring SSO

`services/web/config/adfs.yaml` carries the relying party: our entity id, the
ACS URL, the committed IdP federation metadata file or its URL (which
supplies the signing certificate), the
allow-listed domains, the claim URIs, and the group→role map. There is no
client secret.

**[Set Up Dynapps SSO (Microsoft Entra ID)](/documentation/entra-id-sso-setup)** is the full
procedure: what your Entra ID administrator registers, the claims, every
config key, how groups become roles and per-entity entitlements, and
certificate rotation.

The deployed trust and group mapping are configured in
`services/web/config/adfs.yaml`.

## Deprovisioning

Removing someone from their `Dynapps-Systemprompt-*` group is the whole deprovision
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
| **Expiry** | `security.jwt_access_token_expiration` in the profile, also the cookie's `Max-Age`; there is no default — the profile must set it. Production: 2,592,000 seconds (30 days) |
| **Cookie flags** | `path=/`, `HttpOnly`, `SameSite=Lax`, `Secure` on HTTPS |
| **Required scopes** | `user` or `admin` |

That expiry is the outer bound on a session, not the only thing that ends
one. Every request also confirms the session row is still active and the user
record is still active (the latter through a 30-second cache), so revocation
and account disablement cut a session short well before the token expires. A
bridge token minted from a PAT is separate and lives one hour.

Every admin request passes through two middleware layers. **User context
middleware** extracts and validates the JWT, then loads the user's roles and
project into a `UserContext`. **Auth check middleware** rejects protected
routes without a valid user ID, returning HTTP 401.

`UserContext` carries `user_id`, `username`, `email`, `roles`, `group_ids`,
`project_ids`, the caller's marketplace `participation`, and the derived flags
`is_admin`, `is_console`, `is_platform_admin` and `is_developer`.

To sign out, clear the `access_token` cookie.

## Public vs. Protected Routes

| Route | Access |
|-------|--------|
| `/admin/login` | Public |
| `/admin/auth/adfs/*` | Public — the SSO start and callback |
| `/admin/profile`, `/admin/history`, `/admin/connect`, `/admin/connectors` | Any valid session, including a plain `user` |
| `/admin/analysis/conversations`, `/admin/analysis/versions`, `/admin/sync` and the session, request, trace and context detail pages | A valid session with `admin` / `platform_admin`, **or** marketplace participation — scoped to the participant's marketplaces |
| `/admin/*` (everything else) | A valid session **and** a console role (`admin` or `platform_admin`) |
| Admin JSON writes (`/api/public/admin/*` mutations) | `admin` or `platform_admin` |
| Adding or removing AD-group mappings on groups and projects (`POST`/`DELETE /api/public/admin/{groups,projects}/{id}/ad-mappings`) | `platform_admin` only |
| `/bridge-auth/*` | Requires a valid session — the desktop link runs through SSO |

`/admin/login` and the two SSO routes are the *entire* public surface. There is
no public registration, credential-enrolment, or invite-redemption route; if you
are adding one, you are adding a second way in.

Anonymous requests to a protected route are redirected to
`/admin/login?redirect=…`. A signed-in user **without** a console role is not
shown a 403 for console pages — they are redirected to `/admin/profile`, unless
the page is one their marketplace participation opens. JSON admin API routes
return HTTP 403 instead. The role model, including participation, is on
[Create & Manage Users → Roles](/documentation/admin-user-management#roles).

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

## Service accounts and machine credentials

A workflow that writes back to an external system (opening a GitHub pull request, updating a Salesforce record, commenting in Jira) acts as a **service account**, never as a shared global key. A service account is a `users` row with `account_kind = 'service'`. Core refuses every interactive login for it: session login, passkey, bridge code and OAuth authorization code. Each workflow definition names the account it runs as, and an agent farm names one with `run_as` in `services/web/config/agent-farms.yaml`: its runs mint their credentials for that account and carry the purpose `farm:<run_id>`.

Administrators manage them at **People & access → Service accounts** (`/admin/identity/service-accounts`):

1. **Create** the account with a name and an email. It starts with the plain `user` role.
2. **Attach a machine credential** per provider, keyed by the MCP server id (`github`, `salesforce-<org>`, `atlassian`):

| Kind | Non-secret identifiers | Secret | What the broker returns |
|------|------------------------|--------|-------------------------|
| `github_app` | App ID, installation ID | App private key (PEM) | An installation token: core signs a 10-minute RS256 App JWT and exchanges it at `/app/installations/{id}/access_tokens`. |
| `salesforce_client_credentials` | Instance URL, consumer key (client ID) | Client secret | An access token from the org's `client_credentials` grant. |
| `atlassian_api_token` | Token owner email | API token | `Basic base64(email:token)`. |

The secret is sealed with the platform master key in an envelope bound to the account and provider. It is never rendered or returned, and attaching again rotates it. Only a service account may hold a machine credential, and an account that still holds one cannot be deleted.

When core's MCP client asks the credential broker (`/api/public/connectors/{provider}/token`) for a service account's bearer, the broker requires the `x-systemprompt-caller` purpose core forwards from the run. It issues a token only for `workflow:<definition_id>:write` or `farm:<id>`. Any other purpose, or none, is refused with 403 and logged under the `connector_broker` target with the account, provider and caller. The broker does not mint anything itself. It opens the sealed secret and hands it to core's machine grant (`connector_grant::MachineGrant`), the same code that serves core-managed connectors declared with `grant: app_installation` or `client_credentials`, so there is one App JWT signer, one token exchange and one token cache. Core caches a minted token until two minutes before it expires; the cache key includes the credential's rotation time, so a rotated credential never serves the previous token. People keep the per-user OAuth path unchanged.

GitHub's hosted MCP server is declared in `services/mcp/github.yaml` for this purpose. No plugin enables it. `services/access-control/rules.yaml` grants it, and the Atlassian and Salesforce servers, to the `service-accounts` group that every new service account joins; among people only administrators reach GitHub.
