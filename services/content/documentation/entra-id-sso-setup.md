---
title: "Set Up Dynapps SSO (Microsoft Entra ID)"
description: "The two-sided setup for Dynapps single sign-on with Microsoft Entra ID, over SAML 2.0 (Enterprise Application) or OpenID Connect (App Registration): what to register, the config for each, how Entra groups map into roles and entitlements, and troubleshooting by reason code."
author: "Dynapps"
slug: "entra-id-sso-setup"
keywords: "entra id, azure ad, saml, oidc, openid connect, app registration, sso setup, enterprise application, claims, groups, roles, entitlements, federation metadata, single sign-on configuration"
kind: "guide"
public: true
tags: ["authentication", "security", "sso", "administration"]
published_at: "2026-08-31"
updated_at: "2026-10-13"
after_reading_this:
  - "Register this platform in Microsoft Entra ID as a SAML Enterprise Application or an OpenID Connect App Registration"
  - "Configure the NameID, email, display name and group claims the platform needs"
  - "Map Entra groups to platform roles and to per-entity entitlements"
  - "Pin or fetch the tenant's federation metadata and handle certificate rotation"
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

**TL;DR:** Dynapps SSO trusts Microsoft Entra ID over one of two protocols,
and the setup has two halves either way. Your **Entra ID administrator**
registers the platform and assigns the groups that may use it. **You** put the
matching values in one config file and map each Entra group to the roles it
grants.

| | Option A — SAML 2.0 | Option B — OpenID Connect |
|---|---|---|
| Entra object | Enterprise Application (non-gallery, SAML) | App Registration (Web) |
| Provider id | `entra-saml` | `entra-oidc` |
| Config file | `services/web/config/adfs.yaml` | `services/web/config/entra-oidc.yaml` |
| Trust anchor | Tenant's SAML signing certificate, from federation metadata | Tenant's JWKS, plus a client secret |
| Secret to rotate | None | `entra_oidc_client_secret` in the secrets store |
| Sign-in from My Apps tile | Yes (`allow_idp_initiated`) | No — every sign-in starts at the platform |
| Group keys in `group_roles` | Names or object ids, whichever the claim emits | Object ids |

Pick one per deployment and run it against the same Entra groups. Both feed
the same identity policy, so Step 4 (groups, roles and entitlements), the
lifecycle and the reason codes are identical.

This page is the setup procedure. For how sign-in works at runtime (the
SSO driver, state and replay, the audit table, sessions and route protection)
see [Authentication](/documentation/authentication).

> **About the `adfs` names.** The SSO routes (`/admin/auth/adfs/*`) and the
> config file (`adfs.yaml`) keep their original names. They are stable
> identifiers pinned by the platform's route contract, not a statement about
> which identity provider is in use. Any SAML 2.0 IdP works behind them; this
> instance uses Microsoft Entra ID.

## Option A — SAML 2.0

### What you are building

The platform is the **service provider**. Entra ID is the **identity
provider**. A sign-in is standard SAML 2.0 Web Browser SSO: the platform sends
an `AuthnRequest` to `https://login.microsoftonline.com/<tenant-id>/saml2`,
your Conditional Access policies and MFA apply unchanged, and Entra ID posts a
signed assertion back to the platform's assertion consumer service.

Three consequences worth stating up front:

- **No secret to distribute or rotate.** The assertion's XML signature is
  verified against the certificate in the tenant's federation metadata.
- **The directory is the authority.** Entra group membership is the
  entitlement. The platform re-reads it at every sign-in and stores no
  credential of its own for SSO users.
- **Deny by default.** An assertion carrying no group claim gets no session.

### Step 1: Create the Enterprise Application

In the Entra admin centre: **Enterprise applications → New application →
Create your own application → Integrate any other application you don't find
in the gallery (Non-gallery)**. Name it, for example, `Dynapps Systemprompt`,
then open **Single sign-on → SAML**.

**Basic SAML Configuration:**

| Entra ID setting | Value |
|---|---|
| Identifier (Entity ID) | `https://dynapps.systemprompt.io/saml/metadata` |
| Reply URL (Assertion Consumer Service URL) | `https://dynapps.systemprompt.io/admin/auth/adfs/acs` |
| Sign on URL | `https://dynapps.systemprompt.io/admin/auth/adfs/start` |
| Relay State | *(leave blank)* |
| Logout Url | *(leave blank; single logout is not supported)* |

Both SAML URLs are derived at boot from the profile's external URL, so there is
nothing to edit in `adfs.yaml` and they cannot disagree with the host the
instance serves. **Each host that signs in needs its own Enterprise
Application** (or its own Identifier and Reply URL pair).

> **The Reply URL must match character for character.** Entra ID writes it
> into the response as `Destination`, and the platform rejects an assertion
> whose `Destination` disagrees with its own ACS URL.

**SAML Certificates:**

| Setting | Value |
|---|---|
| Signing option | **Sign SAML assertion** (or *Sign SAML response and assertion*) |
| Signing algorithm | SHA-256 |
| Token encryption | **Off**. Encrypted assertions are not supported |

Copy the **App Federation Metadata Url** from this section. Step 4 uses it.

**Properties → Assignment required?** Set to **Yes**, then under **Users and
groups** assign the groups that may sign in. Entra ID then refuses everyone
else before the platform sees them.

Users may also start a sign-in from the My Apps tile. The platform allows that
by default (`allow_idp_initiated: true`); such an assertion is verified exactly
like a solicited one, only without the request-id correlation.

### Step 2: Attributes and claims

Under **Attributes & Claims**:

| Claim | Source | Notes |
|---|---|---|
| Unique User Identifier (Name ID) | `user.userprincipalname` or `user.mail`, format **Email address** | Without it the sign-in fails with `?sso=no_subject` |
| `http://schemas.xmlsoap.org/ws/2005/05/identity/claims/emailaddress` | `user.mail` | Present by default. Must be a routable address on an allowed domain |
| `http://schemas.microsoft.com/identity/claims/displayname` | `user.displayname` | Add it; the platform uses it as the display name |
| `http://schemas.microsoft.com/ws/2008/06/identity/claims/groups` | **Groups assigned to the application** | Add a group claim and restrict it to assigned groups |

Restricting the group claim to *groups assigned to the application* matters
twice. It stops a user's whole directory membership being shipped on every
sign-in. And Entra ID emits at most 150 groups in a SAML token: past that it
sends an overage link instead of the list, which the platform cannot follow,
so the sign-in is refused with `?sso=group_overage`.

Choose whether the claim carries group **object IDs** (the default) or
**display names** (`sAMAccountName` or cloud-only group names). The keys under
`group_roles` must use the same form.

### Step 3: Map the claims into configuration

Everything lives in `services/web/config/adfs.yaml`. Unknown keys are rejected
at load time, so a typo fails loudly.

| Key | What it does |
|---|---|
| `enabled` | Turns SSO on. Off, the routes still answer, with `?sso=unavailable` |
| `idp_metadata_path` | A committed metadata file, relative to `services/web/config/` |
| `idp_metadata_url` | Alternatively, the App Federation Metadata Url, fetched at boot and every six hours; `idp_metadata_path` is then an optional fallback |
| `idp_entity_id` | Pins the IdP issuer federated identities are keyed by; otherwise read from the metadata |
| `entity_id`, `acs_url` | Overrides for the derived SP URLs — setting them pins every deployment to one host |
| `email_attribute`, `name_attribute`, `groups_attribute` | Claim URIs, if they differ from Step 2 |
| `allowed_email_domains`, `group_roles`, `group_role_patterns`, `default_roles`, `deny_without_group`, `auto_provision` | The shared identity policy — see [Step 4](#groups-roles-and-entitlements-both-options) |
| `allow_idp_initiated` | Accept sign-ins started from My Apps |
| `clock_skew_seconds` | Tolerance for clock drift (default `120`) |

A complete file:

```yaml
enabled: true

# entity_id and acs_url derive from the profile's api_external_url.
idp_metadata_path: entra-federation-metadata.xml
# or: idp_metadata_url: "https://login.microsoftonline.com/<tenant-id>/federationmetadata/2007-06/federationmetadata.xml?appid=<app-id>"

allowed_email_domains:
  - dynapps.es
  - wearedynapps.com

email_attribute: "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/emailaddress"
name_attribute: "http://schemas.microsoft.com/identity/claims/displayname"
groups_attribute: "http://schemas.microsoft.com/ws/2008/06/identity/claims/groups"

deny_without_group: true
allow_idp_initiated: true
clock_skew_seconds: 120

group_roles:
  Dynapps-Systemprompt-Admins: [platform_admin, admin, user]
  Dynapps-Systemprompt-Users: [user]

auto_provision: true
```

If the group claim carries object IDs, key `group_roles` by the ID
(`"3f2b…-…": [admin, user]`) instead of the name.

### Step 4: The signing certificate

Two supported ways to give the platform the tenant's metadata.

**Committed file (`idp_metadata_path`).** The certificate is pinned and any
change is a reviewable diff:

```bash
curl -sS "<App Federation Metadata Url>" \
  -o services/web/config/entra-federation-metadata.xml
```

Commit and redeploy. When Entra ID rolls the SAML signing certificate (by
default every three years, or when you add a new one), repeat this. A stale
pinned certificate rejects every assertion with `?sso=invalid_assertion`.

**Fetched URL (`idp_metadata_url`).** The platform fetches the metadata at
boot and refreshes it periodically, so a certificate rollover is picked up
without a redeploy. The trade-off is that a certificate change is no longer a
reviewed diff.

If `enabled: true`, no `idp_metadata_url` is set and the metadata file cannot
be read, the platform treats it as a configuration error rather than quietly
serving sign-in as unavailable. With a URL set, a failed fetch falls back to
the file when there is one.

### Per-profile overrides

The loader prefers `adfs.<profile>.yaml` (the active deployment profile's
name) and falls back to `adfs.yaml`. A misspelt override is silently absent,
so confirm which file loaded:

```bash
systemprompt infra logs view --since 5m | grep "Loaded SSO config"
```

## Option B — OpenID Connect

The same tenant can instead be trusted over OpenID Connect: an **App
Registration** with a client secret, the authorization code flow with PKCE,
and an `id_token` verified against the tenant's published signing keys.

### Step 1: Register the application

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
the claim to *groups assigned to the application* is the fix, as with SAML.

### Step 2: Store the client secret

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

### Step 3: Configure

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
| `allowed_email_domains`, `group_roles`, `group_role_patterns`, `default_roles`, `deny_without_group`, `auto_provision` | The shared identity policy — see [Step 4](#groups-roles-and-entitlements-both-options) |

```yaml
enabled: true
tenant_id: "<directory (tenant) id>"
client_id: "<application (client) id>"
client_secret_secret: entra_oidc_client_secret
redirect_path: /admin/auth/entra/callback
scopes: [openid, profile, email]
groups_claim: groups
clock_skew_seconds: 120

allowed_email_domains: [dynapps.es, wearedynapps.com]
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

### What is checked

The `id_token` must be RS256-signed by a key from the tenant's JWKS (fetched
only from `jwks_allowed_hosts`), carry this tenant's issuer and `tid`, name the
client as its audience, carry `exp`, `iat`, `iss` and `aud` and be unexpired
within `clock_skew_seconds`, and echo the nonce sent at sign-in. The
authorization code is redeemed with the PKCE verifier stored at start. A user
cancelling at Entra ID (`access_denied`) surfaces as `invalid_assertion`.

## Groups, roles and entitlements (both options)

Both providers embed the same identity policy (core's `SsoIdentityPolicy`), as
top-level keys of their config file. The checks run in a fixed order — email
domain, group overage, group requirement — so the same input always fails the
same way under either protocol.

| Key | Default | Effect |
|---|---|---|
| `allowed_email_domains` | `[]` (every domain) | Case-insensitive match on the part after `@`; a leading `@` in an entry is ignored. Off-list: `?sso=forbidden` |
| `deny_without_group` | `false` | When `true`, an identity carrying no group at all is refused with `?sso=no_group` |
| `group_roles` | `{}` | Exact group (name or object id, as the claim emits it) → roles |
| `group_role_patterns` | `{}` | Glob with a single `*` (prefix and suffix match) → roles, applied after `group_roles` |
| `default_roles` | `[user]` | The roles granted when no group maps to any role |
| `link_by_email` | `false` | Attach a first sign-in with a verified email to the active, interactive local account that already owns it, before `auto_provision` is considered |
| `auto_provision` | `false` | Create the account on a first sign-in; off, an unknown identity gets `?sso=not_provisioned` |

Both shipped files set `deny_without_group: true`, `link_by_email: true` and
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
assertion, exact matches first, then patterns, deduplicated. **Every SSO
sign-in overwrites the account's roles with that set; the platform then
re-joins the roles an admin granted by hand in the dashboard**
(`user_manual_roles`). A role given only through `systemprompt admin users
role promote|assign` is not a dashboard grant and is replaced at the next
SSO sign-in. Removing someone from
`Dynapps-Systemprompt-Admins` revokes `admin` at their next sign-in. What each
role opens is in
[Create & Manage Users → Roles](/documentation/admin-user-management#roles).

`group_role_patterns` maps a glob (one `*`) instead of a fixed name, for
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
in the console are separate rows and survive. A person whose groups map to
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

1. Open `/admin/login`. SAML shows **Sign in with Dynapps SSO (Microsoft
   Entra ID)** (`/admin/auth/adfs/start`, also reachable from the My Apps
   tile); OIDC shows **Sign in with Microsoft** (`/admin/auth/entra/start`).
2. Confirm the roles the directory granted: `systemprompt admin users list`.
3. Read the attempt back from the audit table:
   `systemprompt infra db query "SELECT occurred_at, provider_id, outcome, reason, email, groups_count FROM sso_login_events ORDER BY occurred_at DESC LIMIT 10"`.
4. Check for errors: `systemprompt infra logs view --level error --since 10m`.

## Troubleshooting by reason code

Every failure returns to `/admin/login?sso=<reason>`. The browser sees only
the reason; `sso_login_events.reason` splits the folded cases, and the server
log has the detail.

| `?sso=` | SAML (Option A) — what to fix | OIDC (Option B) — what to fix |
|---|---|---|
| `unavailable` | `enabled: false`, or the metadata is unreachable | `enabled: false`; `tenant_id`/`client_id` empty; `entra_oidc_client_secret` missing from the secrets store (restart after adding it); discovery document unreachable |
| `correlation_lost` | The `sp_sso_bind` cookie was missing, or the sign-in took over 600 s, or was replayed from history (`state_lost`); an unsolicited return with `allow_idp_initiated: false` (`unsolicited`). Start again from the platform | Same, minus the My Apps case: OIDC never accepts an unsolicited return. Check the browser kept the cookie on the callback (served over `https`) |
| `invalid_assertion` | Signature (stale certificate), audience (Identifier), `Destination` (Reply URL), validity, or token encryption left on; `replayed` if the same assertion came twice | Wrong tenant (`tid`/issuer), audience (`client_id`), expired token or clock skew, nonce mismatch, code redemption refused (wrong secret, redirect URI mismatch), or the user cancelled at Entra ID |
| `no_subject` | No Name ID claim | No `sub` in the `id_token` |
| `no_email` | No email claim and no email-format Name ID | Add the `email` optional claim, or make `preferred_username` an address |
| `forbidden` | The email's domain is not in `allowed_email_domains` | Same |
| `no_group` | No group claim — check the group claim and the app assignment | No `groups` claim — check Token configuration and the assignment |
| `group_overage` | Overage link (`groups.link`) instead of groups — restrict the claim to groups assigned to the application | `_claim_names` instead of `groups` — same fix |
| `not_provisioned` | No account and `auto_provision` off; or the account is inactive or a service account | Same |
| `error` | Anything else; the server log has the detail | Same |

A refusal for `forbidden`, `no_group` or `group_overage` also revokes the
sessions and bridge credentials the account already held — see
[Authentication → Deprovisioning](/documentation/authentication#deprovisioning).
