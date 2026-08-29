# REQ-053 — ADFS SSO

Status: **built (SAML), awaiting issuance-rule fixes on the IdP**. Owner ref: Sergey Kubrakov (stakeholder), register row 54.
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
  (AuthnRequest, HTTP-Redirect, RelayState nonce), `POST /admin/auth/adfs/callback`
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
- Tests: `extensions/web/admin/tests/adfs_claims.rs` (claim shapes, group map,
  config defaults), `tests/integration/admin-config/src/config_adfs.rs`
  (loader), `tests/integration/admin-core/src/{users_adfs_groups,users_federated_sso}.rs`
  (side table, dimension, role projection), `tests/contract/admin/src/adfs_auth_contract.rs`
  (unavailable, start, callback ladder), Playwright `@REQ-023`.

## Blocked on Astound (issuance rules on the relying-party trust)

1. Add a Name ID claim (email format) — today's assertion has none → `no_subject`.
2. Add the E-Mail-Address claim.
3. Emit groups as `…/claims/Group`, filtered to `Systemprompt-*`; fix the
   `Sytemprompt-Admins` typo (both spellings mapped meanwhile).
4. Confirm the final `Systemprompt-*` group names.
5. Sync policy (next-login is what is built; a hard TTL needs a number).
6. Deprovision policy (sign-in stops immediately; auto-close the account or not).

## Go-live steps

1. `just deploy-next`; sign in as `e.burton` from `/admin/login` and from the
   AD FS portal; check `systemprompt infra logs view --level error --since 10m`.
2. Once IT ships the claims: switch `groups_attribute` back to
   `…/claims/Group`, drop the misspelt group line.
3. Decide `auto_provision` (recommended `true` — the AD group is the invitation).
4. Rotate the test password.
