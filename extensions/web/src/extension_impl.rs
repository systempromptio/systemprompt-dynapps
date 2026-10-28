//! `Extension` trait implementation for `WebExtension`.
//!
//! The wiring seam: every provider, prerenderer, renderer, schema, migration
//! and job the five web sibling crates expose is advertised to the host runtime
//! here.

use std::sync::Arc;

use systemprompt::extension::prelude::*;
use systemprompt::traits::{Job, RunnerService};

use crate::SkillsPagePrerenderer;
use crate::assets::web_assets;
use crate::extenders::OrgUrlExtender;
use crate::homepage::{HomepagePageDataProvider, HomepagePrerenderer};
use crate::navigation::NavigationPageDataProvider;
use crate::schemas::{migrations, schema_definitions};

use crate::extension::WebExtension;
use systemprompt_web_site::config_loader::SiteConfig;

fn site_config(ctx: &dyn ExtensionContext) -> SiteConfig {
    SiteConfig::load_from(&ctx.services_root().path)
}

impl Extension for WebExtension {
    fn metadata(&self) -> ExtensionMetadata {
        ExtensionMetadata {
            id: "web",
            name: "Web Content & Navigation",
            version: env!("CARGO_PKG_VERSION"),
        }
    }

    fn page_data_providers(&self, ctx: &dyn ExtensionContext) -> Vec<Arc<dyn PageDataProvider>> {
        let site = site_config(ctx);
        let mut providers: Vec<Arc<dyn PageDataProvider>> = vec![];

        if let Some(nav_config) = &site.navigation {
            providers.push(Arc::new(
                NavigationPageDataProvider::new(Arc::clone(nav_config))
                    .with_branding(site.branding.clone()),
            ));
        }

        if let Some(homepage_config) = &site.homepage {
            providers.push(Arc::new(HomepagePageDataProvider::new(Arc::clone(
                homepage_config,
            ))));
        }

        providers.extend(crate::shared::registry::page_data_providers());
        providers
    }

    fn content_data_providers(
        &self,
        _ctx: &dyn ExtensionContext,
    ) -> Vec<Arc<dyn ContentDataProvider>> {
        crate::shared::registry::content_data_providers()
    }

    fn page_prerenderers(&self, ctx: &dyn ExtensionContext) -> Vec<Arc<dyn PagePrerenderer>> {
        let site = site_config(ctx);
        let mut prerenderers: Vec<Arc<dyn PagePrerenderer>> = vec![];

        if let Some(config) = &site.homepage {
            prerenderers.push(Arc::new(HomepagePrerenderer::new(Arc::clone(config))));
        }

        if let Some(config) = &site.skills_page {
            prerenderers.push(Arc::new(SkillsPagePrerenderer::new(Arc::clone(config))));
        }

        prerenderers
    }

    fn component_renderers(&self, _ctx: &dyn ExtensionContext) -> Vec<Arc<dyn ComponentRenderer>> {
        crate::shared::registry::component_renderers()
    }

    fn template_data_extenders(
        &self,
        ctx: &dyn ExtensionContext,
    ) -> Vec<Arc<dyn TemplateDataExtender>> {
        let mut extenders = crate::shared::registry::template_data_extenders();
        extenders.push(Arc::new(OrgUrlExtender::new(
            ctx.app_config().api_external_url.clone(),
        )));
        extenders
    }

    fn schemas(&self) -> Vec<SchemaDefinition> {
        schema_definitions()
    }

    fn migrations(&self) -> Vec<Migration> {
        migrations()
    }

    fn seeds(&self) -> Vec<Seed> {
        vec![
            Seed::new(
                "admin_oauth_client",
                include_str!("../schema/seeds/admin_oauth_client.sql"),
            ),
            Seed::new(
                "groups_projects",
                include_str!("../schema/seeds/groups_projects.sql"),
            ),
        ]
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec![
            "content",
            "users",
            "authz",
            "managed_resources",
            "oauth",
            "files",
        ]
    }

    fn cross_extension_tables(&self) -> Vec<&'static str> {
        vec![
            "agent_run_events",
            "agent_runs",
            "ai_requests",
            "document_folders",
            "document_grants",
            "document_revisions",
            "documents",
            "managed_installation_receipts",
            "managed_publications",
            "managed_resources",
            "markdown_content",
            "mcp_tool_executions",
            "users",
        ]
    }

    fn router(&self, ctx: &dyn ExtensionContext) -> Option<ExtensionRouter> {
        crate::router::build(ctx, self)
    }

    fn site_auth(&self) -> Option<SiteAuthConfig> {
        Some(SiteAuthConfig {
            login_path: "/admin/login",
            protected_prefixes: &["/admin", "/bridge-auth"],
            public_prefixes: &["/admin/login", "/admin/auth/entra"],
            required_scope: "user",
        })
    }

    fn jobs(&self) -> Vec<Arc<dyn Job>> {
        crate::jobs::extension_jobs()
    }

    fn runner_services(&self) -> Vec<Arc<dyn RunnerService>> {
        vec![Arc::new(crate::router::farm_public::FarmRunner {
            runs: Arc::clone(&self.farm_runs),
        })]
    }

    fn priority(&self) -> u32 {
        100
    }

    fn config_prefix(&self) -> Option<&str> {
        Some(Self::PREFIX)
    }

    fn declares_assets(&self) -> bool {
        true
    }

    fn required_assets(&self, paths: &dyn AssetPaths) -> Vec<AssetDefinition> {
        let mut assets = web_assets(paths);
        assets.extend(systemprompt_web_admin_core::assets::admin_assets(paths));
        assets
    }
}
