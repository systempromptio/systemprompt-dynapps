---
title: "Authentication"
description: "The one way into the platform: Dynapps single sign-on (Microsoft Entra ID over SAML 2.0 or OpenID Connect, with Entra groups deciding roles). Covers the SSO provider model, reason codes, the sign-in audit table, operator recovery from the CLI, sessions, JWTs, and route protection."
author: "Dynapps"
slug: "authentication"
keywords: "authentication, login, sso, entra id, saml, oidc, openid connect, sso_login_events, session, JWT, security, provisioning"
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
  - title: "Secrets Management"
    url: "/documentation/secrets-management"
  - title: "Install the Desktop Bridge"
    url: "/documentation/bridge-install"
---

# Authentication

**TL;DR:** There is exactly one way into the platform in a browser, and it
stores no password. Everyone signs in with **Dynapps SSO** (Microsoft Entra
ID, over SAML 2.0 or OpenID Connect) and their **Entra groups** decide what
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

### The provider model

Sign-in is run by core's **SSO driver** (`SsoDriver`). A *provider* only
speaks its protocol: it builds the redirect to the identity provider and
validates what comes back. Everything around that — state correlation, replay,
the identity policy, provisioning, the session and the audit row — belongs to
the driver, so every provider gets the same guarantees and the same failure
codes.

Two providers are registered in this installation, both against Microsoft
Entra ID. A provider whose config is disabled or incomplete is simply not
registered; run one or the other against the same Entra groups.

| Provider id | Protocol | Config | Start | Callback | Metadata |
|---|---|---|---|---|---|
| `entra-saml` | SAML 2.0 (form-POST return) | `services/web/config/adfs.yaml` | `/admin/auth/adfs/start` | `POST /admin/auth/adfs/acs` | `/saml/metadata` |
| `entra-oidc` | OpenID Connect, code flow with PKCE (query return) | `services/web/config/entra-oidc.yaml` | `/admin/auth/entra/start` | `GET /admin/auth/entra/callback` | — |

Core also mounts every registered provider generically at
`/api/v1/core/sso/{id}/start`, `/api/v1/core/sso/{id}/callback` (query or
form-POST) and `/api/v1/core/sso/{id}/metadata`. The `/admin/auth/*` and
`/saml/metadata` paths above are aliases onto the same driver, kept because
they are the URLs the Entra ID applications were registered with. The
difference is only in failure handling: the aliases always send a refused or
unconfigured sign-in to `/admin/login?sso=<reason>`.

The login page shows **Sign in with Dynapps SSO (Microsoft Entra ID)** when
`entra-saml` is registered and **Sign in with Microsoft** when `entra-oidc`
is; a `?redirect=` on the login page is carried through the start.

### What happens on a sign-in

**Start.** The driver asks the provider for its redirect, then stores the
attempt server-side in `sso_state_bindings`: the provider, the post-login
destination, the protocol nonces (SAML request id; OIDC nonce and PKCE
verifier), and a hash of a fresh **binder** value. The raw state and binder
are never stored — both are keyed by HMAC-SHA-256 with the at-rest pepper. The
row lives **600 seconds** and is **single-use**. The binder goes to the
browser as the `sp_sso_bind` cookie (`HttpOnly`, `Path=` the provider's
callback, `Max-Age=600`, `Secure` unless the API is served over plain
`http`). SAML uses `SameSite=None`, because Entra ID posts the ACS
cross-site; OIDC uses `SameSite=Lax`. The post-login destination must be a
same-origin relative path; anything else falls back to `/admin`.

**Callback.** The driver, for every provider:

1. Correlates the return: the `state` (OIDC) or `RelayState` (SAML) must
   match an unconsumed, unexpired row for this provider, and the `sp_sso_bind`
   cookie must match the row's binder. A return with no state is accepted only
   when the provider allows unsolicited sign-ins — SAML's
   `allow_idp_initiated` (My Apps tile); OIDC never does.
2. Lets the provider validate the response. SAML: XML signature against the
   tenant's signing certificate from its federation metadata, audience,
   destination, validity window, and the request id it answers. OIDC: an
   RS256 `id_token` from the tenant's JWKS, issuer, audience, `tid`, expiry
   and nonce.
3. Requires a subject, then claims the assertion id in the **replay guard**
   (`sso_replay_guard`, scoped per provider) until the assertion expires —
   one hour if the provider gives no expiry. A second sight of the same
   assertion is refused.
4. Requires an email and lowercases it.
5. Applies the provider's identity policy (`SsoIdentityPolicy`, the same for
   both protocols): email domain, then group overage, then the group
   requirement, then group → role mapping. See
   [Groups as entitlements](#groups-as-entitlements).
6. Resolves the local user by `(issuer, subject)`; if there is no link and
   the policy sets `link_by_email` (both shipped configs do), links a verified,
   allow-listed email to the active, interactive local account that already
   owns it, so an operator created with `admin users create` signs in without
   a duplicate account. Otherwise, if `auto_provision` is on, provisions one.
   An inactive or non-interactive (service) account is refused.
7. **Writes the resolved roles onto the account** (see below), mints a
   session, and sets it as the `access_token` cookie.
8. Records the outcome in `sso_login_events` and runs the installation's
   sign-in sink: the `login` activity and the directory group and project memberships
   (`services/web/config/groups.yaml`), replacing the directory-sourced rows
   from the last sign-in (`source = 'adfs'`) and leaving dashboard-granted
   ones alone. An Entra group that maps to nothing leaves the person in the
   derived `unassigned` group.

**Roles are overwritten on every SSO sign-in.** This installation's role
resolver hands core's driver the policy's roles — the mapped roles, or
`default_roles` (`[user]`) when no group maps — joined with the roles an admin
granted by hand in the dashboard (`user_manual_roles`), and the driver writes
that set to `users.roles` before minting the session. A role granted only by
`systemprompt admin users role promote|assign` is written to `users.roles`
alone, not to `user_manual_roles`, and is therefore **replaced at that
person's next SSO sign-in** (it survives only if the policy resolves no roles
at all, i.e. `default_roles: []` and no group maps). A sign-in that removes
every admin-minting role revokes the account's older sessions and tokens
before the new session is minted. Grant a role
that must survive SSO from `/admin/roles` or the user's detail page, or map it
from an Entra group.

**Admin is a permission, not just a role.** The session token's permissions
come from `permissions_for_roles`: only the `admin` role mints the `admin`
permission (alongside `user`); every other role, `platform_admin` included,
mints `user` alone. Map `platform_admin` together with `admin`, as the shipped
configs do (`[platform_admin, admin, user]`).

### Reason codes

Every refused sign-in lands on `/admin/login?sso=<reason>`. The code is
deliberately coarse; the finer reason is on the audit row.

| `?sso=` | Audit `reason` | Cause | Message shown on the login page |
|---|---|---|---|
| `unavailable` | `unavailable` | Provider not registered (disabled, incomplete config, OIDC client secret missing) or its IdP metadata/discovery is unreachable | Single sign-on is not configured on this instance yet. |
| `correlation_lost` | `state_lost` | State missing, expired (600 s), already used, or the `sp_sso_bind` cookie does not match | Sent a sign-in this platform did not start. Start again. |
| `correlation_lost` | `unsolicited` | A return with no state, for a provider that does not accept unsolicited sign-ins | (same) |
| `invalid_assertion` | `invalid_assertion` | Signature, audience, destination, issuer, tenant, nonce or validity failed — or the user cancelled at Entra ID | Refused or cancelled the sign-in, or returned a response that could not be verified. |
| `invalid_assertion` | `replayed` | The assertion id was already used | (same) |
| `no_subject` | `no_subject` | No subject (SAML `NameID`, OIDC `sub`) | Did not identify your account. |
| `no_email` | `no_email` | No email claim (and, for SAML, no email-format `NameID`; for OIDC, no address in `preferred_username`) | Did not return an email address. |
| `forbidden` | `forbidden` | Email domain not in `allowed_email_domains` | Your account is not on an email domain this platform accepts. |
| `group_overage` | `group_overage` | Entra ID sent an overage pointer instead of the group list | Your account is in too many Entra ID groups. |
| `no_group` | `no_group` | No group at all and `deny_without_group: true` | Your account is not in a Systemprompt-* Entra ID group. |
| `not_provisioned` | `not_provisioned` | No linked account and `auto_provision` off, or the account is inactive or a service account | No account exists for you yet. |
| `error` | `error` | Anything else — the server log has the detail | Sign-in failed or the sign-in page expired. |

The messages live in `storage/files/js/pages/admin-login.js`; an unknown code
shows the `error` message.

### The sign-in audit table

Every attempt — success or refusal — is one row in `sso_login_events`:
`occurred_at`, `provider_id`, `issuer`, `outcome` (`success` or `refused`), `reason` (refusals only, the
audit reason above), `user_id` (successes only), `email`, `external_sub`, `ip`
and `groups_count`. A refused sign-in is as visible as an accepted one.

```sql
-- Refusals in the last day, by provider and reason
SELECT provider_id, reason, count(*)
FROM sso_login_events
WHERE outcome = 'refused' AND occurred_at > now() - interval '1 day'
GROUP BY 1, 2 ORDER BY 3 DESC;

-- One person's recent attempts
SELECT occurred_at, provider_id, outcome, reason, ip, groups_count
FROM sso_login_events
WHERE email = 'jane@dynapps.es'
ORDER BY occurred_at DESC LIMIT 20;
```

Run them with `systemprompt infra db query "<sql>"` (see
`systemprompt infra db --help`).

### Groups as entitlements

Two things read the Entra groups:

- **Roles** — `group_roles` in the provider's config (`adfs.yaml` or
  `entra-oidc.yaml`) maps a group to `platform_admin`,
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

An operator who also signs in through SSO does **not** keep a role granted only
from the CLI: each SSO sign-in replaces the account's roles with what the
directory maps (or `user`) joined with only the roles granted by hand in the
dashboard. Grant a break-glass operator's `admin` on `/admin/roles` (or map it
from an Entra group) if that operator also signs in through SSO. See
[The provider model](#what-happens-on-a-sign-in).

## Configuring SSO

Each provider has one config file, and both embed the same identity policy
(`allowed_email_domains`, `group_roles`, `group_role_patterns`,
`default_roles`, `deny_without_group`, `auto_provision`):

- `services/web/config/adfs.yaml` — **SAML** (`entra-saml`): the IdP
  federation metadata file or its URL (which supplies the signing
  certificate), the claim URIs, `allow_idp_initiated`. There is no client
  secret; our entity id and ACS URL derive from the profile's external URL.
- `services/web/config/entra-oidc.yaml` — **OIDC** (`entra-oidc`): tenant id,
  client id, `redirect_path`, scopes, the groups claim. The client secret is
  not in the file: it is read from the secrets store under
  `entra_oidc_client_secret` (see
  [Secrets Management](/documentation/secrets-management)).

**[Set Up Dynapps SSO (Microsoft Entra ID)](/documentation/entra-id-sso-setup)** is the full
procedure: what your Entra ID administrator registers, the claims, every
config key, how groups become roles and per-entity entitlements, and
certificate rotation.


## Deprovisioning

Removing someone from their `Dynapps-Systemprompt-*` group is the whole deprovision
step. There is nothing else to click.

Blocking the next sign-in would not be enough on its own: a cookie session and a
linked bridge both outlive the assertion that created them, so a directory
change that only guarded the front door would leave a working desktop app behind
it. So when a sign-in is refused for a missing group, a group overage or an
off-list domain (`no_group`, `group_overage`, `forbidden`), and
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
| `/admin/auth/adfs/*`, `/admin/auth/entra/*`, `/saml/metadata` | Public — the SSO start, callback and SP metadata |
| `/api/v1/core/sso/{id}/*` | Public — core's generic routes onto the same providers |
| `/admin/profile`, `/admin/history`, `/admin/connect`, `/admin/connectors` | Any valid session, including a plain `user` |
| `/admin/analysis/conversations`, `/admin/analysis/versions`, `/admin/sync` and the session, request, trace and context detail pages | A valid session with `admin` / `platform_admin`, **or** marketplace participation — scoped to the participant's marketplaces |
| `/admin/*` (everything else) | A valid session **and** a console role (`admin` or `platform_admin`) |
| Admin JSON writes (`/api/public/admin/*` mutations) | `admin` or `platform_admin` |
| Adding or removing AD-group mappings on groups and projects (`POST`/`DELETE /api/public/admin/{groups,projects}/{id}/ad-mappings`) | `platform_admin` only |
| `/bridge-auth/*` | Requires a valid session — the desktop link runs through SSO |

`/admin/login` and the SSO routes above are the *entire* public surface. There is
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
