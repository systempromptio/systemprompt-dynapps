---
title: "Set Up Dynapps SSO (Microsoft Entra ID)"
description: "The two-sided setup for Dynapps single sign-on: the SAML Enterprise Application your Entra ID administrator registers, and how every claim, domain and Entra group maps into platform roles and entitlements."
author: "Dynapps"
slug: "entra-id-sso-setup"
keywords: "entra id, azure ad, saml, sso setup, enterprise application, claims, groups, roles, entitlements, federation metadata, single sign-on configuration"
kind: "guide"
public: true
tags: ["authentication", "security", "sso", "administration"]
published_at: "2026-08-31"
updated_at: "2026-10-12"
after_reading_this:
  - "Register this platform as a non-gallery SAML Enterprise Application in Microsoft Entra ID"
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
---

# Set Up Dynapps SSO (Microsoft Entra ID)

**TL;DR:** SSO is a SAML 2.0 trust with two halves. Your **Entra ID
administrator** creates a non-gallery Enterprise Application, sets three URLs
and four claims, and assigns the groups that may use it. **You** put the
matching values in `services/web/config/adfs.yaml` and map each Entra group to
the roles it grants. There is no client secret anywhere: trust is anchored on
the tenant's SAML signing certificate, read from its federation metadata.

This page is the setup procedure. For how sign-in works at runtime (the
callback, sessions, cookies and route protection) see
[Authentication](/documentation/authentication).

> **About the `adfs` names.** The SSO routes (`/admin/auth/adfs/*`) and the
> config file (`adfs.yaml`) keep their original names. They are stable
> identifiers pinned by the platform's route contract, not a statement about
> which identity provider is in use. Any SAML 2.0 IdP works behind them; this
> instance uses Microsoft Entra ID.

## What you are building

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

## Step 1: Create the Enterprise Application

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

Copy the **App Federation Metadata Url** from this section. Step 5 uses it.

**Properties → Assignment required?** Set to **Yes**, then under **Users and
groups** assign the groups that may sign in. Entra ID then refuses everyone
else before the platform sees them.

Users may also start a sign-in from the My Apps tile. The platform allows that
by default (`allow_idp_initiated: true`); such an assertion is verified exactly
like a solicited one, only without the request-id correlation.

## Step 2: Attributes and claims

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

## Step 3: Map the claims into configuration

Everything lives in `services/web/config/adfs.yaml`. Unknown keys are rejected
at load time, so a typo fails loudly.

| Key | What it does |
|---|---|
| `enabled` | Turns SSO on. Off, the routes still answer, with `?sso=unavailable` |
| `idp_metadata_path` | A committed metadata file, relative to `services/web/config/` |
| `idp_metadata_url` | Alternatively, the App Federation Metadata Url, fetched at boot and refreshed periodically |
| `allowed_email_domains` | Only these email domains may sign in |
| `email_attribute`, `name_attribute`, `groups_attribute` | Claim URIs, if they differ from Step 2 |
| `group_roles` | Entra group name or object ID → platform roles |
| `group_role_patterns` | Group name glob → roles, applied on top of `group_roles` |
| `deny_without_group` | No group claim, no session (default `true`) |
| `auto_provision` | Create an account on a first sign-in |
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

## Step 4: Groups, roles and entitlements

### The group decides whether there is a session at all

With `deny_without_group: true` (the default), an assertion carrying **no
group claim** is refused with `?sso=no_group`. A group the file does not name
is not a refusal: the person signs in as a plain `user`.

That is also what makes `auto_provision` safe. With assignment required in
Entra ID and deny-on here, a first sign-in is one your administrator has
already authorised: the group assignment *is* the invitation. With
auto-provisioning off, an unknown identity is refused with
`?sso=not_provisioned`.

### The group decides the platform role

A user's roles are the union of the roles of every mapped group in their
assertion. **Directory-sourced roles are re-projected on every sign-in;
manually granted roles survive.** Removing someone from
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

## Step 5: The signing certificate

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

If `enabled: true` and the metadata cannot be read, the platform treats it as
a configuration error rather than quietly serving sign-in as unavailable.

### Per-profile overrides

The loader prefers `adfs.<profile>.yaml` (the active deployment profile's
name) and falls back to `adfs.yaml`. A misspelt override is silently absent,
so confirm which file loaded:

```bash
systemprompt infra logs view --since 5m | grep "Loaded ADFS SSO config"
```

## Verifying a live sign-in

1. Open `/admin/login` and use **Sign in with Dynapps SSO**, or start from the
   My Apps tile. Both reach `/admin/auth/adfs/start`.
2. Confirm the roles the directory granted: `systemprompt admin users list`.
3. Check for errors: `systemprompt infra logs view --level error --since 10m`.

Every failure returns to `/admin/login?sso=<reason>`. The browser sees only
the reason; the detail is in the server log.

| Reason | What to fix |
|---|---|
| `unavailable` | SSO is off, or the metadata is unreadable |
| `correlation_lost` | The sign-in state cookie was missing or expired; start again from the platform |
| `no_subject` | No Name ID claim |
| `invalid_assertion` | Signature, audience, `Destination`, validity or replay. Check the certificate, the Identifier and the Reply URL, and that token encryption is off |
| `no_email` | No email claim and no email-format Name ID |
| `forbidden` | The email's domain is not in `allowed_email_domains` |
| `no_group` | The assertion carried no group. Check the group claim and the app assignment |
| `group_overage` | Entra ID sent an overage link instead of groups. Restrict the claim to groups assigned to the application |
| `not_provisioned` | No account, and `auto_provision` is off |
| `error` | Anything else; the server log has the detail |
