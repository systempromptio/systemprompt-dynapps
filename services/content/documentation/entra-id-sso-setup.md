---
title: "Set Up Dynapps SSO (Microsoft Entra ID)"
description: "The two-sided setup for Dynapps single sign-on with Microsoft Entra ID over OpenID Connect (App Registration): what to register, the config, how Entra groups map into roles and entitlements, and troubleshooting by reason code."
author: "Dynapps"
slug: "entra-id-sso-setup"
keywords: "entra id, azure ad, oidc, openid connect, app registration, sso setup, enterprise application, claims, groups, roles, entitlements, single sign-on configuration"
kind: "guide"
public: true
tags: ["authentication", "security", "sso", "administration"]
published_at: "2026-08-31"
updated_at: "2026-10-15"
after_reading_this:
  - "Register this platform in Microsoft Entra ID as an OpenID Connect App Registration"
  - "Configure the email and group claims the platform needs"
  - "Map Entra groups to platform roles and to per-entity entitlements"
  - "Store and rotate the client secret"
  - "Diagnose a failed sign-in from its ?sso= reason code"
related_playbooks:
  - title: "Authentication"
    url: "/documentation/authentication"
  - title: "User & Access Management"
    url: "/documentation/enterprise-user-access"
  - title: "Create & Manage Users"
    url: "/documentation/admin-user-management"
  - title: "Secrets Management"
    url: "/documentation/secrets-management"
---

# Set Up Dynapps SSO (Microsoft Entra ID)

**TL;DR:** Dynapps SSO trusts Microsoft Entra ID over OpenID Connect, and
the setup has two halves. Your **Entra ID administrator** registers the
platform as an App Registration and assigns the groups that may use it.
**You** put the matching values in one config file
(`services/web/config/entra-oidc.yaml`, provider id `entra-oidc`) and map each
Entra group object id to the roles it grants.

This page is the setup procedure. For how sign-in works at runtime (the
SSO driver, state and replay, the audit table, sessions and route protection)
see [Authentication](/documentation/authentication).

## What you are building

The platform is an OpenID Connect **relying party**; Entra ID is the
**identity provider**. A sign-in is the authorization code flow with PKCE:
your Conditional Access policies and MFA apply unchanged, and the platform
verifies the returned `id_token` against the tenant's published signing keys.

- **The directory is the authority.** Entra group membership is the
  entitlement. The platform re-reads it (the `groups` claim, as group object
  ids) at every sign-in and stores no credential of its own for SSO users.
- **Deny by default.** A token carrying no group gets no session.
- **One secret to rotate**: the client secret, kept in the secrets store.

## Step 1: Register the application

In the Entra admin centre, **App registrations → New registration**,
*Accounts in this organizational directory only* (single tenant), then:

| Entra ID setting | Value |
|---|---|
| Platform | Web |
| Redirect URIs | `https://dynapps.systemprompt.io/admin/auth/entra/callback` **and** `https://dynapps.systemprompt.io/api/v1/core/sso/entra-oidc/callback` |
| Front-channel logout URL | *(leave empty; single logout is not supported)* |
| ID tokens (implicit and hybrid flows) | **Off** — the code flow with PKCE is used |
| API permissions | Microsoft Graph `openid`, `profile`, `email` (delegated) |
| Certificates & secrets | **New client secret**; copy its **Value** (not the Secret ID) |

The first redirect URI is the one the platform sends (`redirect_path`, derived
onto the profile's `api_external_url`). The second is core's generic callback
for the same provider; registering it keeps that route usable too. Each host
that signs in needs its redirect URIs registered.

**Token configuration:**

| Claim | Setting | Why |
|---|---|---|
| `email` | **Add optional claim → ID → email** | The platform reads `email`; without it, it falls back to `preferred_username` when that is an address, else refuses with `?sso=no_email` |
| `groups` | **Add groups claim → Groups assigned to the application**, emitted in the **ID** token as **Group ID** | The group object ids `group_roles` is keyed by |

Then open the matching **Enterprise Application** (same name), set
**Properties → Assignment required?** to **Yes**, and under **Users and groups**
assign the groups named in `group_roles`.

**Tenant restriction.** Register single tenant. The platform also enforces it:
the `id_token` must carry this tenant's issuer
(`https://login.microsoftonline.com/<tenant-id>/v2.0`) **and** a `tid` equal
to `tenant_id`, so a token from any other tenant is refused with
`?sso=invalid_assertion` even if the registration were multi-tenant.

**Group overage.** When a user is in more groups than fit in the token, Entra
ID omits `groups` and sends a `_claim_names` pointer instead. The platform does
not follow it: the sign-in is refused with `?sso=group_overage`. Restricting
the claim to *groups assigned to the application* is the fix.

## Step 2: Store the client secret

The secret never goes in YAML. Put the value in the secrets store under
`entra_oidc_client_secret` (or the key named by `client_secret_secret`). With
the 1Password source this is one more mapping line, for example
`entra_oidc_client_secret: op://dynapps-prod/identity/entra_oidc_client_secret`
— see [Secrets Management](/documentation/secrets-management).

The provider reads the secret **once, when it is built at boot**. A missing or
empty secret leaves the provider off (a warning naming the key, never the
value, and `?sso=unavailable`). Rotating the secret in the store therefore
needs a **restart** to take effect for OIDC sign-in, even though the store
itself reloads.

## Step 3: Configure

`services/web/config/entra-oidc.yaml`. Unknown keys are rejected at load time;
a missing file is a disabled provider.

| Key | What it does |
|---|---|
| `enabled` | Turns the provider on |
| `tenant_id` | Directory (tenant) id; drives the issuer, discovery URL and `tid` check |
| `client_id` | Application (client) id; the `id_token` audience |
| `client_secret_secret` | Secrets-store key of the client secret (default `entra_oidc_client_secret`) |
| `redirect_path` | Callback path, joined onto `api_external_url` (default `/admin/auth/entra/callback`) |
| `scopes` | Default `[openid, profile, email]` |
| `groups_claim` | Claim carrying group ids (default `groups`) |
| `clock_skew_seconds` | Leeway on `exp`/`iat` (default `120`) |
| `discovery_url` | Overrides the tenant's well-known document (sovereign clouds, tests) |
| `jwks_allowed_hosts` | Hosts the JWKS may be fetched from (default `login.microsoftonline.com`) |
| `allowed_email_domains`, `group_roles`, `group_role_patterns`, `default_roles`, `deny_without_group`, `auto_provision` | The identity policy — see [Groups, roles and entitlements](#groups-roles-and-entitlements) |

```yaml
enabled: true
tenant_id: "<directory (tenant) id>"
client_id: "<application (client) id>"
client_secret_secret: entra_oidc_client_secret
redirect_path: /admin/auth/entra/callback
scopes: [openid, profile, email]
groups_claim: groups
clock_skew_seconds: 120

allowed_email_domains: [dynapps.be, dynapps.nl, dynapps.fr, dynapps.es, dynapps.ch, dynapps.eu]
deny_without_group: true
group_roles:
  "<admins group object id>": [platform_admin, admin, user]
  "<users group object id>": [user]
auto_provision: true
```

The provider stays off — with a warning in the log — while `tenant_id`,
`client_id`, the derived redirect URI or the secret is missing. Confirm it
loaded:

```bash
systemprompt infra logs view --since 5m | grep -E "Loaded Entra OIDC config|SSO providers registered|Entra OIDC"
```

## What is checked

The `id_token` must be RS256-signed by a key from the tenant's JWKS (fetched
only from `jwks_allowed_hosts`), carry this tenant's issuer and `tid`, name the
client as its audience, carry `exp`, `iat`, `iss` and `aud` and be unexpired
within `clock_skew_seconds`, and echo the nonce sent at sign-in. The
authorization code is redeemed with the PKCE verifier stored at start. A user
cancelling at Entra ID (`access_denied`) surfaces as `invalid_assertion`.

## Groups, roles and entitlements

The provider embeds core's identity policy (`SsoIdentityPolicy`) as
top-level keys of its config file. The checks run in a fixed order — email
domain, group overage, group requirement — so the same input always fails the
same way.

| Key | Default | Effect |
|---|---|---|
| `allowed_email_domains` | `[]` (every domain) | Case-insensitive match on the part after `@`; a leading `@` in an entry is ignored. Off-list: `?sso=forbidden` |
| `deny_without_group` | `false` | When `true`, an identity carrying no group at all is refused with `?sso=no_group` |
| `group_roles` | `{}` | Exact Entra group object id (as the `groups` claim emits it) → roles |
| `group_role_patterns` | `{}` | Glob with a single `*` (prefix and suffix match) → roles, applied after `group_roles` |
| `default_roles` | `[user]` | The roles granted when no group maps to any role |
| `link_by_email` | `false` | Attach a first sign-in with a verified email to the active, interactive local account that already owns it, before `auto_provision` is considered |
| `auto_provision` | `false` | Create the account on a first sign-in; off, an unknown identity gets `?sso=not_provisioned` |

The shipped file sets `deny_without_group: true`, `link_by_email: true` and
`auto_provision: true` explicitly; a file that omits them gets the defaults
above. The resolved roles are joined with the roles granted by hand in the
dashboard before they are written, so a manual grant survives every sign-in;
a role granted only from the CLI does not (see
[Authentication](/documentation/authentication#what-happens-on-a-sign-in)).

### The group decides whether there is a session at all

With `deny_without_group: true` (as shipped), an identity carrying **no
group** is refused with `?sso=no_group`. A group the file does not name
is not a refusal: the person signs in with `default_roles` (`user`).

That is also what makes `auto_provision` safe. With assignment required in
Entra ID and deny-on here, a first sign-in is one your administrator has
already authorised: the group assignment *is* the invitation. With
auto-provisioning off, an unknown identity is refused with
`?sso=not_provisioned`.

### The group decides the platform role

A user's roles are the union of the roles of every mapped group in their
token, exact matches first, then patterns, deduplicated. **Every SSO
sign-in overwrites the account's roles with that set; the platform then
re-joins the roles an admin granted by hand in the dashboard**
(`user_manual_roles`). A role given only through `systemprompt admin users
role promote|assign` is not a dashboard grant and is replaced at the next
SSO sign-in. Removing someone from the
admins group revokes `admin` at their next sign-in. What each
role opens is in
[Create & Manage Users → Roles](/documentation/admin-user-management#roles).

`group_role_patterns` maps a glob (one `*`) instead of a fixed value, for
families of groups:

```yaml
group_role_patterns:
  Dynapps-Systemprompt-PM-*: [project_manager, user]
```

Roles are read from the user record on every request through a 30-second
cache, so a role changed in the console reaches a live session within half a
minute. A role changed in Entra ID waits for the next sign-in.

### The group decides the workspace and the project

`services/web/config/groups.yaml` maps an Entra group onto a platform
**group** (who someone is, and which marketplace workspace reaches them) and a
**project** (what their usage is attributed to):

```yaml
groups:
  - id: staff
    name: Dynapps staff
    ad_groups: [Dynapps-Systemprompt-Users]
projects:
  - id: odoo-delivery
    name: Odoo delivery
    ad_groups: [Dynapps-Systemprompt-Users]
```

Directory memberships are replaced at each sign-in; memberships an admin adds
in the console are separate rows and survive. `ad_groups` holds whatever the `groups`
claim emits — Entra group object ids. A person whose groups map to
nothing lands in the derived `unassigned` group.

### The group decides per-entity entitlements

To confine a plugin, MCP server, skill, marketplace or gateway route to named
groups, list the platform group id under the entity's `allow.group` in
`services/access-control/rules.yaml`. The full model is in
[Access Control](/documentation/access-control):

```yaml
entities:
  - entity: mcp_server/systemprompt
    default: closed
    why: The admin MCP server, confined to platform administrators.
    allow:
      group: [staff]
```

These are platform group **ids**, not Entra group names. An id that
`groups.yaml` does not declare fails the services gate in CI.

## Lifecycle: joiners, movers and leavers

The directory is consulted only at sign-in. There is **no SCIM** and **no
single logout**.

| Change in Entra ID | Takes effect |
|---|---|
| New user assigned to the app | First sign-in creates the account (just-in-time provisioning, when `auto_provision: true`) |
| Moved between groups | Next sign-in re-projects roles and memberships |
| Removed from the group granting `admin` | Next sign-in; the platform then revokes every session, bridge PAT, device certificate and unredeemed link code the person holds |
| User disabled or unassigned | Entra ID refuses the next sign-in; existing sessions end at token expiry |
| Signed out of Microsoft 365 | No effect on platform sessions, which end at token expiry |

A browser session lives for `security.jwt_access_token_expiration` seconds; a
bridge token minted from a PAT lives one hour. If a leaver cannot wait, disable
the account in the admin console: every request re-reads the user record
through a 30-second cache, so the disable reaches live sessions within about
30 seconds.

## Verifying a live sign-in

1. Open `/admin/login` and choose **Sign in with Microsoft**
   (`/admin/auth/entra/start`).
2. Confirm the roles the directory granted: `systemprompt admin users list`.
3. Read the attempt back from the audit table:
   `systemprompt infra db query "SELECT occurred_at, provider_id, outcome, reason, email, groups_count FROM sso_login_events ORDER BY occurred_at DESC LIMIT 10"`.
4. Check for errors: `systemprompt infra logs view --level error --since 10m`.

## Troubleshooting by reason code

Every failure returns to `/admin/login?sso=<reason>`. The browser sees only
the reason; `sso_login_events.reason` splits the folded cases, and the server
log has the detail.

| `?sso=` | What to fix |
|---|---|
| `unavailable` | `enabled: false`; `tenant_id`/`client_id` empty; `entra_oidc_client_secret` missing from the secrets store (restart after adding it); discovery document unreachable |
| `correlation_lost` | The `sp_sso_bind` cookie was missing, the sign-in took over 600 s or was replayed from history (`state_lost`), or a return arrived with no state (`unsolicited`; OIDC never accepts one). Start again from the platform. Check the browser kept the cookie on the callback (served over `https`) |
| `invalid_assertion` | Wrong tenant (`tid`/issuer), audience (`client_id`), expired token or clock skew, nonce mismatch, code redemption refused (wrong secret, redirect URI mismatch), or the user cancelled at Entra ID |
| `no_subject` | No `sub` in the `id_token` |
| `no_email` | Add the `email` optional claim, or make `preferred_username` an address |
| `forbidden` | The email's domain is not in `allowed_email_domains` |
| `no_group` | No `groups` claim — check Token configuration and the assignment |
| `group_overage` | `_claim_names` instead of `groups` — restrict the claim to groups assigned to the application |
| `not_provisioned` | No account and `auto_provision` off; or the account is inactive or a service account |
| `error` | Anything else; the server log has the detail |

A refusal for `forbidden`, `no_group` or `group_overage` also revokes the
sessions and bridge credentials the account already held — see
[Authentication → Deprovisioning](/documentation/authentication#deprovisioning).
