# done-S

## Final services/web/config/adfs.yaml keys
enabled: false
idp_metadata_path: entra-federation-metadata.xml   (placeholder XML shipped so the sync-inventory test finds the path; the
  loader in extensions/web/site/src/config_loader/adfs.rs reads it only when
  enabled: true, so no placeholder XML was shipped)
allowed_email_domains: [dynapps.es, wearedynapps.com]
email_attribute: http://schemas.xmlsoap.org/ws/2005/05/identity/claims/emailaddress
name_attribute: http://schemas.microsoft.com/identity/claims/displayname
groups_attribute: http://schemas.microsoft.com/ws/2008/06/identity/claims/groups
deny_without_group: true
allow_idp_initiated: true
clock_skew_seconds: 120
group_roles:
  Dynapps-Systemprompt-Admins: [platform_admin, admin, user]
  Dynapps-Systemprompt-Users: [user]
auto_provision: true
(group_role_patterns removed; entity_id/acs_url still derived from api_external_url)

## Groups / projects (services/web/config/groups.yaml)
group `staff` <- AD/Entra groups [Dynapps-Systemprompt-Users, Dynapps-Systemprompt-Admins]
project `odoo-delivery` (no directory mapping)
Removed: europe-devs, india-devs, business-users, uk; projects commerce, core,
storefront, business-implementation.

## Files changed
- services/config/config.yaml (dropped google_workspace + sfcore_dev includes; default_marketplace_id: dynapps-commons)
- services/config/salesforce-orgs.yaml (orgs: {} + how-to-add comment; loader accepts empty map)
- services/mcp/systemprompt.yaml (description)
- services/access-control/rules.yaml (dynapps-* ids; removed astound-india-dev, astound-europe-dev, sfcore_dev, google_workspace; groups -> staff / odoo-delivery). There was no domain grant block; the astounddigital.com mention lived in the deleted google_workspace why.
- services/web/config/{adfs,groups,salesforce}.yaml (salesforce: my_domain placeholder, enabled untouched)
- services/marketplaces/dynapps-*/config.yaml, services/plugins/dynapps-*/config.yaml, dynapps-salesforce/SOURCE.md
  (google_workspace removed from salesforce marketplace/plugin; google_workspace_test skill include removed: the skill dir does not exist). governance hooks only on dynapps-commons. LICENSE is Salesforce's Apache text, unchanged.
- services/skills/*/config.yaml tags (astound -> dynapps), systemprompt_cli SKILL.md + config.yaml
- services/artifacts/admin-ai-usage/config.yaml, services/governance/config.yaml, services/external_agents/claude_desktop.yaml
- services/scheduler/config.yaml: no brand text, unchanged
- deploy/kit/** (known-kits.json kits: []; template + workflows + README rebranded, DYNAPPS_* secrets)
- scripts/generate-cowork-setup-assets.py, scripts/eval/** (ids, gateway URL, bridge paths, India eval cases -> Belgium/Odoo)
- scripts/services-pin.py: no brand text, unchanged
- extensions/cli/{kit-export,dev-login}/Cargo.toml descriptions, kit-export/src/main.rs help text

## Notes
- Skill SKILL.md bodies still mention `google_workspace` only as "never call it" guards; harmless, left.
- calendar_events, inbox_sweep, voice_profile skills are in no plugin (Google-only); left in place, tags rebranded.

## Validation
bash scripts/validate-services.sh -> "services validation OK"; all services YAML parses; kit JSON parses.
git grep astound over my set: only W-owned files remain.

## Handoffs
See .rebrand/handoff-S.md (gateway.yaml project chains, adfs contract test fixture, sync inventory metadata filename, adfs docs, salesforce unit fixture).
