# Authentication on the Astound systemprompt instance

Decision record for replacing Salesforce with Astound's ADFS as the single
sign-on. Written 2026-08-28 from a live probe of the IdP and a read of the
codebase. Nothing in here needs Astound IT to have done anything yet.

## 1. Where we are

Today the admin console and the Claude Code bridge authenticate with **email +
passkey** (WebAuthn, core `crates/domain/oauth`), domain-gated self-registration,
and short-lived JWT sessions minted by core's `SessionCreationService`.

Salesforce was wired as an OpenID Connect identity provider ("Sign in with
Salesforce") and later demoted to a profile-linking mechanism that gated the
Salesforce MCP. That entire body of work — SSO handlers, org provisioning CLI,
JWT-bearer minting, deprovision job, docs, tests — is being retired from `next`
and is preserved at commit `ebf9d8fd` on:

- `systempromptio/systemprompt-astound` — branch `salesforce`
  (also `backup/full-marketplace-and-skills`)
- `systempromptio/systemprompt-template` — branch `salesforce`
- `Astound-Digital/systemprompt-astound` — `backup/full-marketplace-and-skills`

The decision on the table: **Astound's ADFS becomes the single sign-on**, and
Active Directory groups (`Systemprompt-*`) decide what a person may do.

## 2. What Astound's IdP offers (verified 2026-08-28)

`login.astoundcommerce.com` is Microsoft AD FS. Its federation metadata
(`/FederationMetadata/2007-06/FederationMetadata.xml`) is public, signed, and
served over valid TLS. The farm advertises two protocols:

**SAML 2.0**

| item | value |
|---|---|
| IdP Entity ID | `http://login.astoundcommerce.com/adfs/services/trust` |
| SSO endpoint | `https://login.astoundcommerce.com/adfs/ls/` (HTTP-Redirect and HTTP-POST) |
| SLO endpoint | same URL, both bindings |
| Requirements on us | `WantAssertionsSigned="true"` |
| NameID formats | `emailAddress` (1.1), `persistent`, `transient` |
| Signing certificate | `CN=ADFS Signing - login.astoundcommerce.com`, valid 2018-06-11 → **2028-06-08**, SHA-256 `5F:FE:5F:44:4D:79:D1:81:16:8F:F0:E4:19:3F:73:B8:23:71:1E:8F:D0:64:42:D8:DA:47:29:AD:D3:3C:F8:64` |
| Claims available | `…/2005/05/identity/claims/emailaddress`, `…/name`, `…/upn`, `givenname`, `surname`, `http://schemas.xmlsoap.org/claims/Group` |

**OpenID Connect / OAuth 2.0** — `https://login.astoundcommerce.com/adfs/.well-known/openid-configuration` resolves:

| item | value |
|---|---|
| Issuer | `https://login.astoundcommerce.com/adfs` |
| Authorize / token | `/adfs/oauth2/authorize/`, `/adfs/oauth2/token/` |
| JWKS | `/adfs/discovery/keys` (RS256) |
| Grants | `authorization_code` (+ PKCE), `refresh_token`, `device_code`, `jwt-bearer`, `client_credentials` |
| Client auth | `client_secret_post`, `client_secret_basic`, `private_key_jwt` |
| Scopes | `openid email profile allatclaims` (and others) |
| Also | `userinfo_endpoint`, `end_session_endpoint`, `response_mode=form_post` |

This is AD FS 2016 or later with OIDC enabled. Both protocols are driven by the
same issuance rules on the ADFS side, so the `Systemprompt-*` group filter
Astound IT has already written applies either way. They asked for SAML
parameters (Entity ID, ACS URL) because that is the default enterprise ask —
not because OIDC is unavailable.

## 3. SAML vs OIDC for this product

| | SAML 2.0 | OIDC |
|---|---|---|
| Message format | XML assertions, XML-DSig, exclusive canonicalisation, enveloped signatures | JSON Web Tokens (JWS RS256) |
| Existing code here | **none** — no SAML, WS-Fed, or XML-DSig anywhere in core or this repo | complete: the Salesforce login, `repositories/users/federated.rs` resolution, JWKS verification via `jsonwebtoken`, PKCE, session mint |
| Rust ecosystem | `samael` (pulls libxml2 + xmlsec1 + openssl native libs into the Docker build) or a hand-written verifier | `jsonwebtoken` / `openidconnect`, pure Rust, already compiled in |
| Security history | XML Signature Wrapping, comment-injection (the 2018 CVE class that hit every major SP), canonicalisation bugs — SP-side verification is the hardest part to get right | well understood: verify `iss`, `aud`, `nonce`, `exp`, `kid`; PKCE defeats code interception |
| Browser flow | POST-binding form to an ACS | 302 redirect + callback with `code` |
| Group claims | `<Attribute Name="http://schemas.xmlsoap.org/claims/Group">` | `groups` array on the id_token (`allatclaims`) |
| Claude Code bridge | `/bridge-auth` device-link reuses the web session — same either way | same |
| Logout | SLO (rarely worth implementing) | `end_session_endpoint` |
| ADFS admin effort | Relying Party Trust, ~10 min | Application Group (server app + web API), ~10 min |
| Our effort | **~4 weeks** (SP, verifier, tests, hardening) plus a permanent native-dependency and crypto-surface tax on the image | **~2 weeks** (handler ~1 wk, group mapping ~1 wk) |

Nothing a SAML relying party gives the business is missing from OIDC on this
farm. The only reason to pick SAML is if Astound's IdP policy refuses to create
OIDC application groups — a one-line question for IT.

**Recommendation: OIDC.** Hand IT both parameter sets so neither side blocks;
build OIDC; keep SAML as a contingency, not a parallel build.

## 4. Parameters for Astound IT

**Preferred — OIDC Application Group** ("Web browser accessing a web
application", confidential client):

- Redirect URIs:
  - `https://astound.systemprompt.io/admin/auth/adfs/callback`
  - `http://localhost:8080/admin/auth/adfs/callback` (development)
- Scopes: `openid email profile allatclaims`
- Claims on the **id_token**: `email`, `name` (or `given_name` + `family_name`),
  `upn`, and `groups` = Token-Groups **unqualified names**, filtered to
  `Systemprompt-*` (issuance transform rule on the application group — the same
  rule already written for the SAML relying party).
- Please send: client id, client secret (out of band, not Slack), and the exact
  `Systemprompt-*` group names with what each is meant to grant.

**Fallback — SAML 2.0 Relying Party Trust**, only if policy forbids the above:

- Entity ID: `https://astound.systemprompt.io/saml/metadata`
- ACS URL (HTTP-POST): `https://astound.systemprompt.io/admin/auth/adfs/acs`
- NameID: `urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress`
- Signed assertions; no assertion encryption; SLO not required for v1.

## 5. How the login will work (OIDC)

1. User clicks **Sign in with Astound SSO** → `GET /admin/auth/adfs/start` sets a
   state + nonce + PKCE cookie and redirects to `/adfs/oauth2/authorize/`.
2. ADFS authenticates (Windows auth, forms, MFA — per corporate policy) and
   returns a `code` to `/admin/auth/adfs/callback`.
3. We exchange the code at `/adfs/oauth2/token/` and verify the id_token:
   signature against the JWKS, `iss`, `aud`, `nonce`, `exp`; require an email
   on an allow-listed Astound domain.
4. **Group mapping** — `services/access-control/adfs.yaml` maps `Systemprompt-*`
   groups to roles (`admin`, `user`), organizations and plans. **No mapped group
   ⇒ no session** (deny by default). Roles are re-projected on every login, so an
   AD removal takes effect at the next login; mid-session revocation is bounded
   by the session TTL.
5. `federated.rs` resolves `(issuer = https://login.astoundcommerce.com/adfs, sub)`
   to a local user — returning user, merge onto an existing account by verified
   email, or just-in-time provision (seat-limited, org attached by email domain)
   — and core mints the session JWT cookie.
6. Claude Code / the bridge: `/bridge-auth` device-link sits behind the same
   session cookie, so it works unchanged. PATs and bridge JWTs are untouched.

Passkeys remain as a break-glass path for platform operators (or are removed
entirely — a build-time decision).

## 6. Open questions for Astound

1. Is an OIDC application group acceptable? (If not → SAML.)
2. The exact `Systemprompt-*` group names and their intended meaning.
3. Is next-login sufficient for group changes, or is a hard session TTL
   (e.g. 8 h) required?
4. Should disabling the AD account also close the systemprompt account
   (automated deprovision), or is "can no longer sign in" enough?


---

## Addendum — 2026-08-28 afternoon: live test, and the decision flips to SAML

Tested against the farm with the `e.burton` account (forms auth, domain-
qualified `ASTOUND\e.burton`; the bare and UPN forms are rejected):

- IT had **already created a SAML relying party** — "Systemprompt DEV",
  Entity ID `https://astound.systemprompt.io/saml/metadata`, ACS
  `https://astound.systemprompt.io/admin/auth/adfs/callback` (HTTP-POST) —
  reachable from the AD FS portal's IdP-initiated sign-on page.
- **No OIDC client exists** (every candidate id → `invalid_client`,
  MSIS9607). The OIDC path could not be exercised.
- The assertion for the test account is signed RSA-SHA256 by the metadata
  signing certificate (fingerprint matches), enveloped on the Assertion,
  not encrypted, 1-hour validity. Claims as actually emitted:
  `…/claims/name = Edward Burton`; `http://temp/groups = [Domain Users,
  ADFS-FS-Users, Sytemprompt-Admins]`. **No NameID, no email claim**; the
  group attribute is not the announced `…/claims/Group`, is unfiltered, and
  the group name is misspelt.
- Pure-Rust SAML tooling exists (`saml` crate: XML-DSig, exclusive C14N,
  XSW-resistant extraction, no libxml2/xmlsec). Prototyped against the
  captured assertion: a tampered group name is rejected with a digest
  mismatch; the genuine assertion is rejected only for the missing NameID.

**Decision (Ed, same day): switch to SAML.** The relying party is live, the
verifier is pure Rust, and the captured assertion is a test fixture. The
OIDC handlers were replaced; nothing OIDC remains. Entity ID and ACS above
are the parameters of record — the SAML fallback list in §4 is now the
primary and the OIDC list is moot.

What IT still has to change on the relying-party trust before a sign-in
completes:

1. Add a **Name ID** claim (email-address format) — without a `<NameID>` the
   assertion is rejected as `no_subject`.
2. Add the **email** claim (`…/2005/05/identity/claims/emailaddress`).
3. Emit groups as `http://schemas.xmlsoap.org/claims/Group`, **filtered to
   `Systemprompt-*`** (today: `http://temp/groups`, unfiltered), and fix the
   `Sytemprompt-Admins` typo. Until then `adfs.yaml` maps both spellings and
   reads `http://temp/groups`.

## Addendum (2026-09-01): projects from AD groups

IT provisioned `Systemprompt-Commerce` and `Systemprompt-Core`. With the
directory able to say which of the two workstreams a person is in, the pooled
multi-organization model this instance carried (organizations, plans, seats,
budgets, departments) was deleted rather than repointed: every user is admin
or user, and belongs to exactly one project, replaced on every sign-in from
`group_projects` in `adfs.yaml`. Dashboards, listings and reports split on it;
entitlements keep using the `adfs_group` dimension, since a project is a group.
Detail: `docs/tickets/req-053-adfs-sso.md`, section *Projects*.
