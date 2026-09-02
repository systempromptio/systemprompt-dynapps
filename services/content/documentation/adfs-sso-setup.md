---
title: "Set Up Astound SSO (AD FS)"
description: "The two-sided setup for Astound single sign-on: what your IT team registers in AD FS, and how every claim, domain, and Active Directory group maps into platform roles and entitlements. Includes moving an instance from a test farm to production."
author: "Astound Digital"
slug: "adfs-sso-setup"
keywords: "adfs, saml, sso setup, relying party trust, claim rules, active directory groups, roles, entitlements, federation metadata, single sign-on configuration"
kind: "guide"
public: true
tags: ["authentication", "security", "sso", "administration"]
published_at: "2026-08-31"
updated_at: "2026-08-31"
after_reading_this:
  - "Register this platform as a relying party in AD FS with the right identifier and endpoint"
  - "Write the three claim issuance rules the platform needs"
  - "Map Active Directory groups to platform roles and to per-entity entitlements"
  - "Pin and rotate the identity provider's signing certificate"
  - "Point an instance at a different AD FS farm without disturbing the others"
related_playbooks:
  - title: "Authentication"
    url: "/documentation/authentication"
  - title: "User & Access Management"
    url: "/documentation/enterprise-user-access"
  - title: "Create & Manage Users"
    url: "/documentation/admin-user-management"
---

# Set Up Astound SSO (AD FS)

**TL;DR:** SSO is a SAML 2.0 trust with two halves. Your **AD FS team**
registers this platform as a relying party and writes three claim issuance
rules. **You** put the matching values in `services/web/config/adfs.yaml` and
map each `Systemprompt-*` Active Directory group to the roles it grants. There
is no client secret anywhere: trust is anchored on the identity provider's
signing certificate, committed to the repository as a federation metadata file.

This page is the setup procedure. For how sign-in works at runtime — the
callback, sessions, cookies, and route protection — see
[Authentication](/documentation/authentication).

## What you are building

The platform is the **service provider** (relying party). AD FS is the
**identity provider**. A sign-in is standard SAML 2.0 Web Browser SSO: the
platform sends an `AuthnRequest`, your corporate login policies apply
unchanged — Windows authentication, forms, MFA — and AD FS posts a signed
assertion back to the platform's assertion consumer service.

Three consequences worth stating up front:

- **No secret to distribute or rotate.** The assertion's XML signature is
  verified against the certificate in the pinned metadata. Certificate
  rotation is a file refresh and a redeploy, reviewable as a diff.
- **The directory is the authority.** Active Directory group membership is the
  entitlement. The platform re-reads it at every sign-in and stores no
  credential of its own for SSO users.
- **Deny by default.** An assertion carrying no group the platform maps gets
  no session at all.

## Step 1 — Register the relying-party trust in AD FS

Give your AD FS administrator the two values in the middle column, where
`<host>` is the platform's public hostname (for example
`sp-dev.astound.digital`). Both must be `https`.

| AD FS setting | Value | Config key it must equal |
|---|---|---|
| Relying party identifier | `https://<host>/saml/metadata` | `entity_id` (derived) |
| Assertion Consumer Service (POST binding) | `https://<host>/admin/auth/adfs/acs` | `acs_url` (derived) |
| Token signature | Assertion signed, RSA-SHA256 | — |
| Token encryption | **Not** encrypted | — |
| Name ID format | Email address | — |

`<host>` is whatever the profile's `--external-url` names: both SAML URLs are
derived from it at boot, so there is nothing to edit in `adfs.yaml` and the two
values cannot disagree with each other or with the URL the instance serves
itself under. **Each host that signs in needs its own relying-party trust.**
Two deployments on one AD FS farm — a staging host and a production host, say —
are two trusts with identical claim rules; an instance whose identifier has no
trust behind it is refused with `MSIS7007: The requested relying party trust
... is unspecified or unsupported`.

Ask for one value back: the farm's **federation metadata URL**, normally
`https://<farm>/FederationMetadata/2007-06/FederationMetadata.xml`. Step 5
pins it.

> **The ACS URL must match character for character.** The farm writes the
> registered endpoint into the assertion as both `Destination` and
> `Recipient`, and the platform rejects an assertion whose `Destination`
> disagrees with its own `acs_url`. A trust registered for one path while the
> platform serves another does not fail with a clear message — the browser is
> redirected, re-POSTs to a route that does not accept POST, and the user sees
> a bare `405`.

Also decide whether users may start a sign-in from the AD FS portal — the
application tile on `/adfs/ls/idpinitiatedsignon`. The platform allows it by
default (`allow_idp_initiated: true`); such an assertion is verified exactly
like a solicited one, only without the request-id correlation.

## Step 2 — Claim issuance rules

The relying-party trust needs three issuance rules. All three are required;
the platform rejects a sign-in that is missing any of them.

1. **Name ID, in email-address format.** Without it the assertion carries no
   subject and the sign-in fails with `?sso=no_subject`.
2. **E-Mail-Address**, sourced from the user's **`mail`** attribute. Source it
   from `mail`, not from the internal AD namespace — a rule that emits
   `user@company.local` produces an address that is not routable and is not on
   any domain you would sensibly allow-list, which makes the domain gate in
   Step 3 meaningless.
3. **Group**, filtered to the platform's groups only. Emit group membership
   under `http://schemas.xmlsoap.org/claims/Group`, filtered to names matching
   `Systemprompt-*`. Filtering matters: an unfiltered claim ships a user's
   entire directory group list to the platform on every sign-in.

Agree the exact group names at this point, and confirm the spelling in a live
assertion before going live — a misspelt group is a group that maps to nothing
and a user who cannot sign in.

## Step 3 — Map the claims into configuration

Everything lives in one file, `services/web/config/adfs.yaml`. The claim URIs
default to the standard AD FS ones; set them only if your issuance rules name
them differently.

| AD FS claim | Config key | The platform uses it for |
|---|---|---|
| `http://schemas.xmlsoap.org/ws/2005/05/identity/claims/emailaddress` | `email_attribute` | The login identity and the domain gate |
| `http://schemas.xmlsoap.org/ws/2005/05/identity/claims/name` | `name_attribute` | The display name |
| `http://schemas.xmlsoap.org/claims/Group` | `groups_attribute` | Roles and the `adfs_group` access dimension |

The rest of the file:

| Key | Default | What it does |
|---|---|---|
| `enabled` | `false` | Turns SSO on. Off, the routes still answer, with `?sso=unavailable` |
| `entity_id` | *(required)* | The relying-party identifier from Step 1 |
| `acs_url` | *(required)* | The assertion consumer service from Step 1 |
| `idp_metadata_path` | *(required)* | The pinned metadata file, relative to `services/web/config/` |
| `allowed_email_domains` | `astounddigital.com`, `astoundcommerce.com` | Only these email domains may sign in |
| `group_roles` | *(empty)* | Active Directory group → platform roles. See Step 4 |
| `deny_without_group` | `true` | No mapped group, no session |
| `auto_provision` | `false` | Create an account on a first sign-in |
| `allow_idp_initiated` | `true` | Accept sign-ins started from the AD FS portal |
| `clock_skew_seconds` | `120` | Tolerance for drift between the farm and this server |

Unknown keys are rejected at load time, so a typo fails loudly rather than
silently reverting to a default.

A complete file:

```yaml
enabled: true

entity_id: "https://sso.example.com/saml/metadata"
acs_url: "https://sso.example.com/admin/auth/adfs/acs"
idp_metadata_path: adfs-federation-metadata.xml

allowed_email_domains:
  - example.com

email_attribute: "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/emailaddress"
name_attribute: "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/name"
groups_attribute: "http://schemas.xmlsoap.org/claims/Group"

deny_without_group: true
allow_idp_initiated: true
clock_skew_seconds: 120

group_roles:
  Systemprompt-Admins: [admin, user]
  Systemprompt-Commerce: [user]
  Systemprompt-Core: [user]

group_projects:
  Systemprompt-Commerce: commerce
  Systemprompt-Core: core

auto_provision: true
```

## Step 4 — Groups, roles, and entitlements

This is the part worth getting right. An Active Directory group reaches the
platform as a claim and then does four separate jobs.

### The group decides whether there is a session at all

`deny_without_group: true` — the default, and the recommended setting — means
an assertion whose groups map to nothing is refused with `?sso=no_group`. The
directory group is the authorisation; without one there is nothing to grant.

Turn it off only on an evaluation instance where every user on an allow-listed
domain should get in as a plain `user`.

This setting is also what makes `auto_provision` safe. Auto-provisioning
creates an account on a first successful sign-in. With deny-on and a group
claim filtered to `Systemprompt-*`, a first sign-in that carries a mapped
group is one your IT team has already authorised — the AD group *is* the
invitation. With deny **off**, auto-provisioning would create an account for
anyone on an allow-listed domain, which is a different and much weaker gate.
With auto-provisioning off, SSO signs in existing accounts only, and an
unknown identity is rejected with `?sso=not_provisioned`.

### The group decides the platform role

`group_roles` maps each group to the roles it grants:

```yaml
group_roles:
  Systemprompt-Admins: [admin, user]
  Systemprompt-Commerce: [user]
  Systemprompt-Core: [user]
```

A user's roles are the union of the roles of every mapped group in their
assertion, deduplicated. Two rules follow:

- **Roles are replaced, not merged, on every sign-in.** Removing someone from
  `Systemprompt-Admins` in Active Directory revokes `admin` at their next
  sign-in — you do not also have to demote them here.
- **A group the file does not name maps to nothing.** It grants no role and it
  is not recorded, so it cannot be used for entitlements either.

The two roles the platform itself understands:

| Role | Reaches |
|---|---|
| `user` | `/admin/profile`, `/admin/settings`, `/admin/setup`, the desktop bridge, the gateway |
| `admin` | The whole admin console, plus everything `user` reaches |

Roles are read from the user record on every request, so a change takes effect
immediately for an existing session — no sign-out required.

### The group decides the project

`group_projects` maps a group to the **project** the user belongs to —
`commerce` or `core`. The project is the split every dashboard, listing and
report runs on: an admin sees every project and can narrow any page with a
Project selector; a plain user only ever sees their own project's data.

```yaml
group_projects:
  Systemprompt-Commerce: commerce
  Systemprompt-Core: core
```

Three rules follow:

- **Every plain user needs exactly one.** A non-admin whose assertion carries
  no project group is refused with `?sso=no_project`.
- **Two is a mistake, not a merge.** Anyone — admin or not — in both project
  groups is refused with `?sso=ambiguous_project`, so a directory
  misconfiguration is visible rather than silently resolved.
- **Admins may have none.** An admin in neither project group signs in with
  no project and sees everything; an admin in one is also just recorded as
  that project's member.

Like roles, the project is replaced on every sign-in, and every group named
here must also appear in `group_roles`. There is no way to set a project from
the console: move someone by moving them in Active Directory.

### The group decides per-entity entitlements

Roles are coarse. To confine one plugin, MCP server, agent, skill,
marketplace, or gateway route to the members of a named group, list it in
`services/access-control/adfs.yaml`:

```yaml
grants:
  - entity_type: mcp_server
    entity_id: knowledge-bank
    groups:
      - Systemprompt-Admins
```

Unlike the SP configuration above, **this file is not profile-aware**: it is
read from one fixed path, so there is no `adfs.<profile>.yaml` equivalent for
grants and every profile shares these. The list is empty by default, which
costs a multi-environment deployment nothing today — but a grant added here
applies everywhere the image is deployed.

At startup each grant becomes an allow rule at `rule_type=adfs_group`, and the
entity's default is forced **closed** — so a user outside every listed group
cannot reach it whatever their role would otherwise allow. Group rules sit
above role rules in the precedence ladder:

```
user 0  →  adfs_group 150  →  role 200
```

There is no separate `project` dimension: a project *is* an AD group, so
confining an entity to one project is a grant to `Systemprompt-Commerce` or
`Systemprompt-Core` here.

A group used here **must also appear in `group_roles`**, because only mapped
groups are recorded against the account. Naming a group in a grant that
`group_roles` does not map confines that entity to nobody.

### End to end

A user in `Systemprompt-Admins` signing in:

1. AD FS issues the filtered group claim carrying `Systemprompt-Admins`.
2. `group_roles` maps it to `[admin, user]`, which replace whatever roles the
   account held.
3. The group is recorded against the account for the `adfs_group` dimension.
4. The user reaches the whole console, plus any entity granted to
   `Systemprompt-Admins`, and is closed out of entities granted only to other
   groups.
5. IT removes them from the group. At their next sign-in the roles are
   re-projected as empty — and because `deny_without_group` is on, that
   sign-in is refused outright.

Group changes take effect at the **next sign-in**; a session already issued
runs until it expires.

## Step 5 — Pin the identity provider's signing certificate

The platform never fetches federation metadata at sign-in time. It reads the
copy committed beside the config, so the trusted signing certificate is pinned
and any change to it is a reviewable diff:

```bash
curl -sS https://<farm>/FederationMetadata/2007-06/FederationMetadata.xml \
  -o services/web/config/adfs-federation-metadata.xml
```

Commit the result and redeploy. **When the farm rotates its token-signing
certificate, repeat this** — a stale pinned certificate rejects every
assertion with `?sso=invalid_assertion`. Ask your AD FS team for the current
certificate's expiry and diarise the refresh ahead of it.

If `enabled: true` and the file named by `idp_metadata_path` cannot be read,
the platform treats it as a configuration error rather than quietly serving
sign-in as unavailable.

## Pointing an instance at a different farm

Two supported ways.

**One environment — edit the file.** Change `entity_id`, `acs_url`,
`allowed_email_domains`, and `group_roles` in
`services/web/config/adfs.yaml`, re-pin the metadata for the new farm, and
redeploy.

**Several environments — add a per-profile file.** The loader prefers
`adfs.<profile>.yaml` — the active deployment profile's name — and falls back
to `adfs.yaml` when it is absent. Each file names its own metadata, so both
farms' certificates sit side by side:

```
services/web/config/
  adfs.yaml                              # the default every profile falls back to
  adfs-federation-metadata.xml
  adfs.production.yaml                   # used only by the `production` profile
  adfs-federation-metadata.production.xml
```

Add the override; never rename the default.

**The name must match the deployment profile exactly.** The loader builds the
filename from the active profile's name, so profile `production` reads
`adfs.production.yaml` and nothing else. A misspelt or misplaced override is
**not** an error: it is simply absent, and the instance silently falls back to
`adfs.yaml` — pointing at whichever farm the default names, which on a fresh
checkout is somebody else's. The farm then rejects the assertion and the
symptom is `?sso=invalid_assertion`, which reads like a certificate or ACS
problem and sends you to the wrong place entirely.

So confirm the file actually loaded rather than assuming it. The startup log
names it:

```bash
systemprompt infra logs view --since 5m | grep "Loaded ADFS SSO config"
```

Check this first, before anything else in the triage table below. It is one
line, and it rules out the failure that looks like every other failure.

## Verifying a live sign-in

1. Use **Sign in** in the site header, or open `/admin/login` and use **Sign
   in with Astound SSO** — both start the same flow at
   `/admin/auth/adfs/start`. Repeat from the AD FS portal tile if you allow
   IdP-initiated sign-in.
2. Confirm the roles the directory granted: `systemprompt admin users list`.
3. Check for errors: `systemprompt infra logs view --level error --since 10m`.

Every failure returns to `/admin/login?sso=<reason>`. The browser is told only
the reason; the detail is in the server log.

| Reason | What to fix |
|---|---|
| `unavailable` | SSO is off, or the pinned metadata is unreadable |
| `denied` | AD FS refused, or the user cancelled |
| `no_subject` | No Name ID claim — issuance rule 1 in Step 2 |
| `invalid_assertion` | Signature, audience, `Destination`, validity, or replay — check the pinned certificate and the ACS URL |
| `no_email` | No email claim and no email-format Name ID — issuance rules 1 and 2 |
| `forbidden` | The email's domain is not in `allowed_email_domains` |
| `no_group` | No group the platform maps — check the filter and the spelling |
| `not_provisioned` | No account, and `auto_provision` is off |
| `no_project` | A non-admin in neither `Systemprompt-Commerce` nor `Systemprompt-Core` |
| `ambiguous_project` | In both project groups — leave the user in exactly one |
| `error` | Anything else; the server log has the detail |
