---
title: "User & Access Management"
description: "Manage enterprise users from the admin UI: search, roles, session and PAT revocation, closed registration, and Astound SSO with AD groups as entitlements."
author: "Astound Digital"
slug: "enterprise-user-access"
keywords: "users, access, roles, invites, sso, adfs, active directory, pat, sessions, registration, deprovisioning"
kind: "guide"
public: true
tags: ["enterprise", "admin", "access"]
published_at: "2026-08-25"
updated_at: "2026-09-16"
after_reading_this:
  - "Manage users end to end from /admin/access/users without touching the CLI or database"
  - "Provision users through SSO or operator administration"
  - "Understand the closed browser registration and recovery paths"
  - "Revoke sessions and personal access tokens for any user immediately"
  - "Understand how Astound SSO provisions accounts and how AD groups revoke access"
related_docs:
  - title: "Access Control: Who Reaches What, and Why"
    url: "/documentation/access-control"
  - title: "Authentication"
    url: "/documentation/authentication"
  - title: "Enterprise Roadmap & Known Limitations"
    url: "/documentation/enterprise-roadmap"
---

# User & Access Management

The admin UI at `/admin/access/users` manages accounts, roles, status, sessions, and personal access tokens. Browser sign-in uses Astound SSO; operators can provision and recover access through the CLI. Public registration, browser invite redemption, and passkey login are not offered by this installation.

## Managing users from the admin UI

Open `/admin/access/users` as an admin. From this one page you can:

- **Search and list** users by name or email, with pagination.
- **Create** a user directly.
- **Edit roles and status** — promote or demote roles (`user`, `developer`, `knowledge_worker`, `project_manager`, `admin`), activate or suspend an account.
- **Disable or delete** an account. Disabling keeps history; deleting removes the account.
- **Revoke sessions** — force sign-out everywhere with one action.
- **Revoke personal access tokens** — every PAT the user holds can be revoked from their detail page, or from `/admin/devices/pats` which lists tokens across the instance.
- **See last activity** — each user row shows last-active time, so stale accounts are visible at a glance.

No CLI or database access is required for any of these operations. The CLI equivalent for scripting exists under `systemprompt admin users` (see `systemprompt admin users --help`).

## Provisioning users

Users enter through group-gated SSO when `auto_provision` is enabled, or an operator creates their account. Existing invite and WebAuthn administration primitives do not provide a browser onboarding flow here. Follow [Authentication](/documentation/authentication) and [Create & Manage Users](/documentation/admin-user-management).

## Astound SSO and deprovisioning

The login page offers **Sign in with Astound SSO**. A verified assertion must carry an allowed email domain and a group claim when `deny_without_group` is enabled. Mapped groups grant their configured roles; an unmapped group receives `user`. Directory role grants are refreshed at sign-in, while manual grants survive. Users may belong to zero, one, or several projects. See [Set Up Astound SSO (AD FS)](/documentation/adfs-sso-setup).

Deprovisioning follows the directory: disabling the AD account stops sign-in immediately, and removing someone from a `Systemprompt-*` group strips the role it granted at their next sign-in. Mid-session revocation is bounded by the session TTL; an admin can revoke sessions and PATs immediately from `/admin/access/users`.

SCIM provisioning is not offered — AD FS does not push standards-based SCIM, so an endpoint would have no caller. See the [Enterprise Roadmap](/documentation/enterprise-roadmap) for when this changes.

## The project-manager role

`project_manager` is a read-only admin. It reaches every page of the dashboard — users, access, analytics, conversations, traces, reports, the catalog — across every project rather than one, and it changes nothing: user and role edits, access-control rules, gateway configuration and device enrolment all stay `admin`-only. It is also outside the admin control plane, so a project manager is not granted the `systemprompt` MCP server, the `astound-admin` plugin or the admin skills, and an admin tool call from one is denied by governance like any other non-admin request.

The role is granted by Active Directory. Any group matching `Systemprompt-ProjectManagers-*` — one per region — maps to it, so IT adds a region by creating the group, with no configuration change here. Project membership is independent of this role; users are not restricted to exactly one project.

## Personal access tokens and devices

`/admin/devices/pats` lists every personal access token on the instance: owner, prefix (so a leaked token can be identified without exposing it), optional expiry, and creation time. Any token can be revoked immediately, and revocation takes effect on the next request.

## Time-bound sharing

Share tokens honour an expiry timestamp, and time-bound group membership and access rules are being added in this release as `valid_from`/`valid_until` columns. Invites, setup tokens, and JWTs are already time-bound, so no standing credential lives forever by default. There is currently no forced maximum PAT lifetime policy — for contractors, lean on account-level disable plus PAT revocation (see the [roadmap](/documentation/enterprise-roadmap)).

## Context-aware access control

What a session can reach — marketplaces, plugins, skills, MCP servers, gateway routes — is decided per request from the person's **current context**: their role, the groups and projects the last SSO sign-in placed them in, and the accounts they have linked. The narrowest band that names them decides, a deny beats an allow inside it, and an entity with rules is closed to everyone they do not name. Every rule is declared once in `services/access-control/rules.yaml` with a stated reason and enforced from the database; `/admin/access-control` shows who reaches what and why, and its Sync page reconciles code and console in either direction. The full model, worked examples and how-tos are in [Access Control](/documentation/access-control).

## Verification

Use the commands and UI checks above against the configured instance. Repository maintainers can run `just test-unit` and `just test-integration`; a passing fixture test does not establish that a deployed integration is configured. Screenshots below are illustrative and may show an earlier configuration.

![The user roster at /admin/access/users with search and role columns](/files/images/evidence/req-001-users-roster.png)
*The user roster: search, roles, status, and last-active at a glance.*

![A user detail page showing role editing and session/PAT revocation](/files/images/evidence/req-001-user-detail.png)
*A user's detail page with role edit, disable, and session and PAT revocation.*

![The login page offering a single Astound SSO button and no other way in](/files/images/evidence/req-002-login-sso-only.png)
*The login surface: one door. No credential field, no registration path — operators can provision accounts, and successful SSO can provision them when enabled.*
