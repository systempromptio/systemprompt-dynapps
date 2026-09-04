---
title: "User & Access Management"
description: "Manage enterprise users from the admin UI: search, roles, invites, session and PAT revocation, closed registration, and Astound SSO with AD groups as entitlements."
author: "Astound Digital"
slug: "enterprise-user-access"
keywords: "users, access, roles, invites, sso, adfs, active directory, pat, sessions, registration, deprovisioning"
kind: "guide"
public: true
tags: ["enterprise", "admin", "access"]
published_at: "2026-08-25"
updated_at: "2026-08-31"
after_reading_this:
  - "Manage users end to end from /admin/access/users without touching the CLI or database"
  - "Invite new users through the hashed, single-use, 7-day invite flow"
  - "Verify all three self-registration doors are closed on your instance"
  - "Revoke sessions and personal access tokens for any user immediately"
  - "Understand how Astound SSO provisions accounts and how AD groups revoke access"
related_docs:
  - title: "Authentication"
    url: "/documentation/authentication"
  - title: "Enterprise Roadmap & Known Limitations"
    url: "/documentation/enterprise-roadmap"
---

# User & Access Management

**TL;DR:** Every user-lifecycle operation — search, create, role and status changes, disable or delete, session and personal-access-token revocation — lives in the admin UI at `/admin/access/users`. Self-registration is closed on all three doors; the ways in are an admin invite followed by passkey enrolment, or Astound SSO carrying a mapped `Systemprompt-*` Active Directory group. Groups are re-read at every sign-in, so leaving a group revokes the role it granted.

## Managing users from the admin UI

Open `/admin/access/users` as an admin. From this one page you can:

- **Search and list** users by name or email, with pagination.
- **Create** a user directly, or send an invite (see below).
- **Edit roles and status** — promote or demote roles (`user`, `project_manager`, `admin`), activate or suspend an account.
- **Disable or delete** an account. Disabling keeps history; deleting removes the account.
- **Revoke sessions** — force sign-out everywhere with one action.
- **Revoke personal access tokens** — every PAT the user holds can be revoked from their detail page, or from `/admin/devices/pats` which lists tokens across the instance.
- **See last activity** — each user row shows last-active time, so stale accounts are visible at a glance.

No CLI or database access is required for any of these operations. The CLI equivalent for scripting exists under `systemprompt admin users` (see `systemprompt admin users --help`).

## Inviting users

The invite flow is the standard way in:

1. An admin creates an invite from `/admin/access/users`.
2. The invite token is **hashed at rest**, **single-use**, and expires after a **7-day TTL**.
3. The recipient follows the link, sets up their credential, and lands in the account with the roles the admin assigned.

An expired or already-used invite is rejected; issue a fresh one.

## Closed registration: the three doors

Enterprise instances refuse unrestricted self-registration. Three independent doors exist, and all three ship closed:

| Door | Control | State |
|---|---|---|
| Core self-registration | `allow_registration` in core config | Off |
| Passkey self-registration | Passkey sign-up flow | Off |
| SSO auto-provisioning | Group-gated JIT | On, and gated on a mapped AD group (see below) |

An unapproved visitor cannot create an account or connect a Bridge. The only enrolment paths are an admin invite or a gated SSO provision.

## Astound SSO and deprovisioning

The login page offers **Sign in with Astound SSO** — SAML 2.0 against the corporate AD FS farm — and a passkey form for platform operators. A sign-in is accepted only when the signed assertion carries an allow-listed email and at least one `Systemprompt-*` Active Directory group the instance maps; an assertion with no mapped group gets no session. Where SSO-initiated provisioning is enabled (`auto_provision` in `services/web/config/adfs.yaml`), the mapped group is the authorisation: an account is created on first sign-in. Every person is an **admin** (`Systemprompt-Admins`), a **project manager** (any `Systemprompt-ProjectManagers-*` group), or a plain user, and every plain user belongs to exactly one **project** — `Systemprompt-Commerce` or `Systemprompt-Core` — which is what every dashboard, listing and report is split by. The full setup procedure is in [Set Up Astound SSO (AD FS)](/documentation/adfs-sso-setup).

Deprovisioning follows the directory: disabling the AD account stops sign-in immediately, and removing someone from a `Systemprompt-*` group strips the role it granted at their next sign-in. Mid-session revocation is bounded by the session TTL; an admin can revoke sessions and PATs immediately from `/admin/access/users`.

SCIM provisioning is not offered — AD FS does not push standards-based SCIM, so an endpoint would have no caller. See the [Enterprise Roadmap](/documentation/enterprise-roadmap) for when this changes.

## The project-manager role

`project_manager` is a read-only admin. It reaches every page of the dashboard — users, access, analytics, conversations, traces, reports, the catalog — across every project rather than one, and it changes nothing: user and role edits, access-control rules, gateway configuration and device enrolment all stay `admin`-only. It is also outside the admin control plane, so a project manager is not granted the `systemprompt` MCP server, the `astound-admin` plugin, the `admin_console` agent or the admin skills, and an admin tool call from one is denied by governance like any other non-admin request.

The role is granted by Active Directory. Any group matching `Systemprompt-ProjectManagers-*` — one per region — maps to it, so IT adds a region by creating the group, with no configuration change here. Because project managers span projects, they are exempt from the "exactly one project group" rule that applies to plain users.

## Personal access tokens and devices

`/admin/devices/pats` lists every personal access token on the instance: owner, prefix (so a leaked token can be identified without exposing it), optional expiry, and creation time. Any token can be revoked immediately, and revocation takes effect on the next request.

## Time-bound sharing

Share tokens and access grants carry expiry: share tokens honour an expiry timestamp, and grants respect a `valid_until` boundary. Invites, setup tokens, and JWTs are already time-bound, so no standing credential lives forever by default. There is currently no forced maximum PAT lifetime policy — for contractors, lean on account-level disable plus PAT revocation (see the [roadmap](/documentation/enterprise-roadmap)).

## Context-aware access control

What a session can reach — MCP servers, gateway routes and models, plugins — is decided per request from the user's **current context**: role, and the Active Directory groups (including the project group) carried by the last SSO sign-in. Rules are evaluated deny-overrides with narrowest-band-wins, and an entity that has rules **defaults to deny** for anyone the rules don't name. Two users with different contexts get different authorized sets from the same instance, and unlinking or demotion narrows access on the next request.

## Verified evidence

Every capability on this page is proven by tagged end-to-end tests run against a seeded instance. To replicate: `just start`, then `just e2e-seed --reset`, then the command in the table. Screenshots regenerate with `just e2e-screens`.

| Ref | Verified behaviour | Replicate with |
|---|---|---|
| REQ-001 | An admin can search, create, edit roles/status, disable, and revoke sessions and PATs for a user entirely from the admin UI | `just e2e-req REQ-001` |
| REQ-002 | An unapproved visitor cannot register on any of the three doors; an admin invite is the only working path in | `just e2e-req REQ-002` |
| REQ-023 | The login page offers Astound SSO and the operator passkey form, and the SSO routes refuse a malformed return by redirecting to login rather than erroring | `just e2e-req REQ-023` |
| REQ-025 | Share tokens and grants stop working after their expiry / `valid_until` boundary | `just e2e-req REQ-025` |
| REQ-044 | Role and AD-group context each change the authorized entity set; deny overrides allow; ruled entities default to deny | `just test-integration` |

![The user roster at /admin/access/users with search and role columns](/files/images/evidence/req-001-users-roster.png)
*The user roster: search, roles, status, and last-active at a glance.*

![A user detail page showing role editing and session/PAT revocation](/files/images/evidence/req-001-user-detail.png)
*A user's detail page with role edit, disable, and session and PAT revocation.*

![The login page offering a single Astound SSO button and no other way in](/files/images/evidence/req-002-login-sso-only.png)
*The login surface: one door. No credential field, no registration path — an account exists only once a signed assertion carries a mapped `Systemprompt-*` AD group.*

Deeper gateway-level checks for these behaviours also run in the platform's integration suite (`just test-integration`).
