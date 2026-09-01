# REQ-053 — ADFS SSO

Status: **live — SSO is the platform's only login.** Owner ref: Sergey Kubrakov (stakeholder), register row 54.
Decision record: `docs/decisions/2026-08-28-adfs-sso-replaces-salesforce.md`;
Slack reply: `docs/decisions/2026-08-28-adfs-sso-slack-reply.md`.

## What was answered (2026-08-28)

Astound IT sent the IdP details. Probed read-only:

- Federation metadata resolves and is signed; token-signing cert valid to 2028-06-08.
- SAML SSO/SLO at `https://login.astoundcommerce.com/adfs/ls/`.
- **OIDC is enabled** on the farm (`/adfs/.well-known/openid-configuration`
  → authorization-code + PKCE, RS256 JWKS at `/adfs/discovery/keys`,
  `allatclaims` scope, `userinfo`). This answered Q1 of the original ticket:
  the OIDC path is viable, no SAML stack needed.
- Claims IT emits: email, name, Groups filtered to `Systemprompt-*`.

Decision (revised same afternoon, see the decision record's addendum):
**SAML** — IT had already created the relying party "Systemprompt DEV" and no
OIDC client exists; the `saml` crate gives a pure-Rust verifier. Salesforce is retired from `next` (preserved on the `salesforce`
branches of `systempromptio/systemprompt-astound` and
`systempromptio/systemprompt-template`).

## What is built

- `extensions/web/admin/src/handlers/adfs_auth/` — `GET /admin/auth/adfs/start`
  (AuthnRequest, HTTP-Redirect, RelayState nonce), `POST /admin/auth/adfs/acs`
  (assertion consumer: XML-DSig against the pinned metadata cert, audience,
  destination, validity, replay cache, request correlation; claim gate,
  group→role map, federated resolution, session mint). IdP-initiated sign-in
  from the AD FS portal is accepted.
- `services/web/config/adfs.yaml` + `adfs-federation-metadata.xml` (pinned
  IdP cert). No client secret.
- Roles re-projected onto `users.roles` on every sign-in (replace, not union);
  an empty map is not a demotion (`repositories/users/federated.rs`).
- `adfs_group` subject dimension (precedence 150) over the new
  `adfs_group_memberships` table, written at sign-in; entity grants in
  `services/access-control/adfs.yaml` projected by
  `repositories/config/adfs_yaml_loader.rs`.
- Login page: "Sign in with Astound SSO" (rendered only when the config is
  usable) above the operator passkey form; `?sso=<reason>` messages.
- Tests: `tests/unit/web/src/adfs_claims.rs` (claim shapes, group map,
  config defaults), `tests/unit/web/src/adfs_config_files.rs` (the committed
  `adfs*.yaml` parse, pin a metadata file that exists, and agree on host/ACS), `tests/integration/admin-config/src/config_adfs.rs`
  (loader), `tests/integration/admin-core/src/{users_adfs_groups,users_federated_sso}.rs`
  (side table, dimension, role projection), `tests/contract/admin/src/adfs_auth_contract.rs`
  (unavailable, start, callback ladder), Playwright `@REQ-023`.

## Closed on the IdP (2026-08-31)

IT applied all three issuance fixes; verified against a live assertion the same
day. Recorded here so the history stays readable:

1. ~~Add a Name ID claim (email format)~~ — done; `emailAddress` format.
2. ~~Add the E-Mail-Address claim~~ — done.
3. ~~Emit groups as `…/claims/Group`, filtered to `Systemprompt-*`; fix the
   `Sytemprompt-Admins` typo~~ — done; the misspelt fallback mapping has been
   dropped from `group_roles`. The legacy unfiltered `http://temp/groups` is
   still emitted alongside the filtered claim; we read the filtered one.
4. ~~Confirm the final `Systemprompt-*` group names~~ — `Systemprompt-Admins`
   and `Systemprompt-Users` (the latter retired 2026-09-01, see *Projects*).

`auto_provision` was turned on the same day: with `deny_without_group: true` and
a group claim filtered to `Systemprompt-*`, a first sign-in carrying a mapped
group is one IT has already authorised, so the AD group is the invitation.

## Projects (2026-09-01)

IT created `Systemprompt-Commerce` and `Systemprompt-Core`; these plus
`Systemprompt-Admins` are the groups the farm sends. The multi-organization
model (plans, seats, budgets, departments) was deleted outright; every person
is admin or user, and belongs to exactly one **project**:

- `group_projects` in `adfs.yaml` maps the two groups to `commerce` / `core`;
  both also grant `[user]` in `group_roles`. `Systemprompt-Users` is gone.
- `gate_claims`: a non-admin in no project group → `?sso=no_project`; anyone
  in both → `?sso=ambiguous_project` (fail closed); an admin in none signs in
  with no project. Both refusals revoke like `no_group`.
- The project is written to `user_profile_ext.project` on every sign-in
  (replace) and read per request into `UserContext.project`. No console
  write path — move people in AD.
- `util/project_scope.rs`: admins see every project and narrow with a
  Project selector (`?project=`); non-admins are pinned to their own. Every
  analytics, requests, traces, contexts, users and report query takes it.
- No `project` authz dimension: the group name is already bound by
  `adfs_group`, so per-project entitlements are grants to the group in
  `services/access-control/adfs.yaml`.

## Configuring another farm (2026-08-31)

The loader prefers `services/web/config/adfs.<profile>.yaml` and falls back to
`adfs.yaml`, so a deployment points at its own AD FS farm by adding one file
plus its own pinned metadata — the committed DEV config stays the default and
our staging deploy is unaffected. The startup log names the file it read.

Customer-facing procedure: `services/content/documentation/adfs-sso-setup.md`
(`/documentation/adfs-sso-setup`) — the relying-party trust, the three issuance
rules, every config key, the group→role→entitlement chain, and certificate
rotation.

## Still open

1. **Sync policy** — next-login re-projection is what is built; a hard TTL still
   needs a number from the stakeholder.
2. **Deprovision policy** — *closed 2026-08-31.* A refused sign-in now reaches
   backwards: `repositories/users/revocation.rs::revoke_user_access` revokes the
   user's sessions, bridge PATs, device certificates, and any unredeemed
   device-link code, in one transaction. It runs on a `no_group` / `forbidden`
   denial (matched by email — the gate fails before an identity is resolved) and
   when a returning login's role projection takes `admin` away, since
   `session.rs::permissions_for_roles` baked that permission into every token
   minted while the user still held it. Bounded residual: an already-minted
   bridge JWT stays valid for its TTL (core's `DEFAULT_ACCESS_TTL_SECONDS`, one
   hour), so revocation bites within the hour rather than instantly. Whether to
   auto-*close* the account — as opposed to stripping its credentials — is still
   undecided, and is a smaller question now that the credentials go.
3. **`astound.local` in `allowed_email_domains`** — the farm's `emailaddress`
   rule sources the internal AD namespace rather than the user's `mail`
   attribute, so a real sign-in asserts `e.burton@astound.local`. The domain is
   listed to unblock testing and, while listed, the domain gate means little.
   Remove it once IT emits `mail`. Not ours to close.
4. **Replay cache is in-process** (`handlers/adfs_auth/callback.rs`). Correct for
   a single instance; a multi-instance deploy must move it to Postgres before
   scaling, or the same assertion replays against a second machine.

## Single-login consolidation (2026-08-31)

SSO is now the *only* browser door, not merely the preferred one:

- `POST /admin/auth/passkey/register` (self-registration) — route and handler
  removed. Enrolment is closed by construction, not by a config flag; the
  `allow_self_registration` field is gone from `AdfsConfig` and `adfs.yaml`.
- `GET /admin/invite/{token}` and `POST /admin/auth/invite/accept` — removed.
  Redeeming an invite provisioned an account and enrolled a credential outside
  SSO, which was a second way in. Admin-side invite *management* remains as a
  record, but see the note below.
- The login page's passkey sign-in form and "Create an account" form are gone;
  the page offers one button.
- The SSO JWT now derives its permissions from the mapped roles
  (`permissions_for_roles`), so a `Systemprompt-Admins` member holds
  `Permission::Admin`. Previously every SSO session was minted as `User`
  regardless of group, and the governance scope check reads that permission.

`handlers/users_bootstrap.rs` no longer mints a bootstrap invite: it used to
return an `/admin/invite/{token}` URL, which would now 404. An admin-created row
is adopted by the first assertion carrying a mapped group and the matching
email, so the API returns a note instead of a link (the admin UI already
rendered the note when no link was present).
