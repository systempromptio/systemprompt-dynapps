# Agent D: hosted documentation (services/content/documentation/**)

## Renamed / deleted (for W: navigation)
- RENAMED (git mv): adfs-sso-setup.md -> entra-id-sso-setup.md (slug `entra-id-sso-setup`,
  title "Set Up Dynapps SSO (Microsoft Entra ID)"). Fully rewritten for Entra ID SAML.
- DELETED (git rm): salesforce.md (Astound-org specific: Astound US source org, packaged ECA,
  astound callbacks). Slug `salesforce` no longer exists.
- No India/sfnext docs existed as separate pages; references inside other docs were removed.

## Cross-links fixed
All `/documentation/adfs-sso-setup` -> `/documentation/entra-id-sso-setup`; links to
`/documentation/salesforce` removed (index.md, connect-cowork.md).

## Content notes
- Bulk identifier swap per BRIEF table on all 43 files; prose hand-edited in authentication,
  access-control (examples now group `staff`, project `odoo-delivery`, matching rules.yaml /
  groups.yaml), services-sync + update-a-skill (no kit pinned; Odoo example skill),
  enterprise-roadmap, enterprise-user-access, admin-user-management, deploy*, connect-*.
- Google Workspace connector references removed (connect-cowork, enterprise-roadmap,
  enterprise-tool-governance, access-control).
- `?sso=` reason table uses current strings from adfs_auth/ (unavailable, correlation_lost,
  no_subject, invalid_assertion, no_email, forbidden, no_group, not_provisioned, error) plus
  `group_overage` (agent E). The old `denied` reason was dropped: it is not in the code.
- entra-id-sso-setup.md documents both `idp_metadata_path` (entra-federation-metadata.xml)
  and `idp_metadata_url` (agent E) and the placeholder groups
  Dynapps-Systemprompt-Admins / Dynapps-Systemprompt-Users.
- Image-access tokens now "issued by systemprompt.io" (image is ghcr.io/systempromptio/...).
- Every @RELEASE_VERSION@ token preserved (count per file unchanged vs HEAD).

## Handoffs / blockers
None. No builds run, nothing committed.
