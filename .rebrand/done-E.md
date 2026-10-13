# done-E: Entra ID SSO on the SAML relying party

## Files changed
- extensions/web/admin/identity/src/handlers/adfs_auth/{config,identity,mod,provider,callback,start,session}.rs
- NEW extensions/web/admin/identity/src/handlers/adfs_auth/metadata.rs (fetch + 6h refresh of federation metadata)
- extensions/web/admin/identity/src/lib.rs (re-exports GROUPS_OVERAGE_ATTRIBUTE, IDP_METADATA_REFRESH_INTERVAL, spawn_idp_metadata_refresh)
- extensions/web/admin/identity/Cargo.toml (adds `reqwest = { workspace = true }`; description prose). Cargo.lock needs R's regen (reqwest already in the lock via platform/analytics).
- extensions/web/admin/identity/src/handlers/users_bootstrap.rs (sign_in_note + module doc)
- extensions/web/site/src/config_loader/adfs.rs (metadata path optional when URL set; disabled reads no file; spawns the refresher once at load — the loader result is OnceLock-cached)
- extensions/web/shared/admin-core/src/repositories/sync/inventory/kinds.rs (labels + `web/config/entra-federation-metadata.xml`; S committed that file, so tests/unit/web/src/sync_inventory.rs stays green. At runtime a missing file renders as an "absent" row, never fatal.)
- services/web/templates/skills.html line 85 only
- tests/unit/web/src/adfs_claims.rs (Entra defaults, GUID group mapping, overage detection, URL-only config parse)
- tests/contract/admin/src/adfs_auth_contract.rs + NEW tests/contract/admin/fixtures/entra-federation-metadata.xml (placeholder tenant 00000000-0000-4000-8000-000000000e17, self-signed cert, key discarded). baseline.txt untouched.
- extensions/web/src/router/admin/login.rs: no change needed (button label lives in login.hbs, already Dynapps).

## Config keys (adfs.yaml)
- NEW optional `idp_metadata_url: <App Federation Metadata Url>` — fetched at boot and every 6h (5 min retry after failure); the last good document is kept on failure, then the pinned file.
- `idp_metadata_path` is now optional (`#[serde(default)]`). enabled + neither key = config error. enabled + URL + unreadable file = warning only.
- Defaults changed: name_attribute = http://schemas.microsoft.com/identity/claims/displayname, groups_attribute = http://schemas.microsoft.com/ws/2008/06/identity/claims/groups, allowed_email_domains = [dynapps.es, wearedynapps.com]. S's current adfs.yaml parses unchanged; S should ADD `idp_metadata_url` once the tenant exists (recommended over a committed file).

## Denial reasons (`/admin/login?sso=<reason>`)
Existing: unavailable, error, no_email, forbidden, no_group, not_provisioned, no_subject, correlation_lost, invalid_assertion.
NEW: `group_overage` — refused when the assertion carries http://schemas.microsoft.com/claims/groups.link; it revokes sessions like forbidden/no_group. Log line tells the operator to set the groups claim to "Groups assigned to the application". login.hbs does not switch on the reason today, so no template string is needed (no handoff).

## Entra Enterprise App values operators enter
- Identifier (Entity ID): https://dynapps.systemprompt.io/saml/metadata
- Reply URL (ACS): https://dynapps.systemprompt.io/admin/auth/adfs/acs
- Sign on URL (optional): https://dynapps.systemprompt.io/admin/auth/adfs/start
- Claims: emailaddress (user.mail), displayname, groups claim = "Groups assigned to the application", source attribute Group ID (GUIDs go in group_roles), or sAMAccountName/cloud-only display name if synced.
- SAML signing option: "Sign SAML assertion" (default). Signing algorithm SHA-256.
- Token encryption: leave OFF.
- Metadata: copy "App Federation Metadata Url" into idp_metadata_url.

## Blockers / caveats for requirements.txt
- SAML token encryption is unsupported (no SP decryption key configured).
- Groups overage (>150 groups) is refused, not resolved via Microsoft Graph; requires "Groups assigned to the application".
- Single logout (SLO) is not implemented.
- Group values are object-id GUIDs unless the app emits names; group_roles and groups.yaml must name what the app emits.

## Handoffs
- Optional: extensions/web/site/src/config_loader/mod.rs:157 log label "ADFS config error" → "SSO config error" (prose, not mine).
