# done-T — Rust extensions + id tests

## Extensions changed
- mcp/systemprompt/src/reports/renderer.rs: renderer name `dynapps-admin-reports`
- web/admin/analytics/src/services/analysis_report/mod.rs: seed `dynapps:analysis_report`
- web/jobs/src/conversation_judge/judge/mod.rs: seed `dynapps:conversation_judge`
- web/admin/platform/src/services/connector_oauth/{generic,transport}.rs: client_name "Dynapps systemprompt"
- web/admin/platform/src/services/connector_oauth/verify.rs: User-Agent "Dynapps-systemprompt", clientInfo "Dynapps connection verification"
- web/admin/platform/src/handlers/bridge_whoami.rs, services/salesforce_orgs.rs, shared/admin-core authz/cached.rs, repositories/scope/participation.rs: doc text
- web/admin/platform/src/handlers/salesforce_auth/config.rs: example audience `https://<my-domain>.my.salesforce.com` (no org hard-coded; value comes from config)
- web/admin/platform/src/handlers/ssr/ssr_connectors_cards.rs: "The Dynapps control plane"
No producer-key constants (`astound.feedback*` etc.) existed in my Rust set; none touched.

## Salesforce registry
`orgs: {}` (as S wrote) loads: `orgs` is `#[serde(default)]` BTreeMap and empty input returns default. No handoff needed.

## Tests changed
Contract: catalog, ssr_access_control_audience, ssr_people_containers, pat_export.
Integration: admin-config executive_marketplace (dynapps-* ids; removed `google_workspace_test` from EXPECTED skills — now matches services/plugins/dynapps-salesforce exactly; removed the google_workspace MCP access tuple and the then-unused `operator` binding), salesforce_identity (`@dynapps.es`), admin-core dashboard_usage_daily (`dynapps-commons`), groups_marketplaces (dynapps-developers / dynapps-cowork / dynapps-consultants), mcp_servers_yaml (`@dynapps/crm-mcp`), web content_api + jobs_context (`dynapps.test`), schema-upgrade feedback.rs doc.
Unit: admin_reports (asserts "Dynapps"), authz_marketplace_pure, analysis_skills (`dynapps-odoo-ba`), builders, connector_oauth, content_config_paths, conversation_view, jobs_governance_config, kit_export, marketplace_hash (`bundle:dynapps-ba`), mcp_tool_name, registry_completeness, salesforce_orgs (fixture org `example-co.my.salesforce.com`), services_tree_declarations, site_docs_page_data, site_org_url_extender, site_page_providers.
Also tests/fixtures/oauth/server.py: mock email `sales.tester@dynapps.example` (connector_oauth.rs asserts it; must change together).

## Flags for the validation pass
- mcp_tool_name.rs: underscored-server fixture is now `odoo_erp` (local known() list, not a real server) so the "underscored id kept whole" test keeps its purpose; no real underscored server remains.
- site_page_providers / site_docs_page_data build their own data ("Dynapps"); they do not read W's files.
- kit_export reads services/skills/who_am_i and marketplace dynapps-commons: both present.
- executive_marketplace runtime-policy assertions depend on S's rules.yaml for dynapps-super-admin/dynapps-salesforce/salesforce.
- Generic fixture group names (`india-devs`, `europe-devs`, `bundle:sfnext`) in access-control/scope unit tests left as test data (no brand string).
- lint-inline-comments currently red only on adfs_auth/{metadata,provider}.rs (not my set). headers/size/comments gates green.
