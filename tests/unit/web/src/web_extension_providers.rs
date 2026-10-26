//! The provider getters are how the runtime discovers what the web extension
//! contributes to a render. They are inventory-backed, and inventory failures
//! are silent — a dropped `submit_*!` or an LTO-stripped registration shows up
//! as an empty list, not a compile error. Each getter is therefore asserted to
//! be non-empty with unique ids: a duplicate id means one registration shadows
//! another, which is equally silent.
//!
//! The hooks take the extension context; the fixture context below boots a
//! profile over the repository's `services/` tree, so the config-derived
//! providers (navigation, homepage, skills) are built from the shipped config.

use std::any::Any;
use std::collections::HashSet;
use std::sync::Arc;

use systemprompt::extension::prelude::{Extension, ExtensionContext};
use systemprompt::manifest::services::root::ActiveServicesRoot;
use systemprompt::manifest::{Config, Profile};
use systemprompt::models::auth::TokenAuthority;
use systemprompt::storage::LocalFileStorage;
use systemprompt::traits::{
    BackgroundTasks, ConfigProvider, DatabaseHandle, FileStorage, SecretStore,
};
use systemprompt_web_admin_core::extension_boot::ExtensionBoot;
use systemprompt_web_extension::WebExtension;

#[derive(Debug)]
struct StubConfig;

impl ConfigProvider for StubConfig {
    fn get(&self, _key: &str) -> Option<String> {
        None
    }
    fn database_url(&self) -> &'static str {
        "postgres://unused/unused"
    }
    fn system_path(&self) -> &'static str {
        "/tmp"
    }
    fn api_port(&self) -> u16 {
        0
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
struct NoDb;

impl DatabaseHandle for NoDb {
    fn is_connected(&self) -> bool {
        false
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

struct FixtureCtx {
    boot: ExtensionBoot,
    _fixture: crate::boot_fixture::Fixture,
}

impl FixtureCtx {
    fn new() -> Self {
        let fixture = crate::boot_fixture::Fixture::new();
        Self {
            boot: fixture.extension_boot(),
            _fixture: fixture,
        }
    }
}

impl ExtensionContext for FixtureCtx {
    fn system_owner_id(&self) -> systemprompt::identifiers::UserId {
        systemprompt::identifiers::UserId::new("providers-test-owner")
    }
    fn config(&self) -> Arc<dyn ConfigProvider> {
        Arc::new(StubConfig)
    }
    fn profile(&self) -> Arc<Profile> {
        Arc::new(self.boot.profile().clone())
    }
    fn app_config(&self) -> Arc<Config> {
        Arc::clone(self.boot.config_arc())
    }
    fn services_root(&self) -> ActiveServicesRoot {
        self.boot.active_services_root().clone()
    }
    fn token_authority(&self) -> Arc<dyn TokenAuthority> {
        Arc::clone(self.boot.token_authority())
    }
    fn secrets(&self) -> Arc<dyn SecretStore> {
        Arc::clone(self.boot.secrets())
    }
    fn dependencies(&self) -> systemprompt::traits::Dependencies {
        systemprompt::traits::Dependencies::new()
    }
    fn database(&self) -> Arc<dyn DatabaseHandle> {
        Arc::new(NoDb)
    }
    fn get_extension(&self, _id: &str) -> Option<Arc<dyn Extension>> {
        None
    }
    fn file_storage(&self) -> Arc<dyn FileStorage> {
        Arc::new(LocalFileStorage::new(std::env::temp_dir()))
    }
    fn instance_id(&self) -> systemprompt::identifiers::InstanceId {
        systemprompt::identifiers::InstanceId::new("providers-test")
    }
    fn background_tasks(&self) -> BackgroundTasks {
        BackgroundTasks::new()
    }
    fn services(&self) -> systemprompt::manifest::services::ServicesView {
        crate::boot_fixture::empty_services_view()
    }
    fn provider_tokens(&self) -> Arc<dyn systemprompt::traits::ProviderTokenCache> {
        Arc::new(systemprompt::security::credential::TokenCache::default())
    }
    fn subject_attributes(&self) -> Arc<dyn systemprompt::models::authz::SubjectAttributeSource> {
        Arc::new(systemprompt::governance::authz::SubjectProviderSet::from_providers(Vec::new()))
    }
}

#[test]
fn page_data_providers_are_registered_with_unique_ids() {
    let ctx = FixtureCtx::new();
    let providers = WebExtension::new().page_data_providers(&ctx);
    assert!(!providers.is_empty(), "no page data provider registered");

    let ids: HashSet<&str> = providers.iter().map(|p| p.provider_id()).collect();
    assert_eq!(ids.len(), providers.len(), "two providers share an id");
    for provider in &providers {
        assert!(!provider.provider_id().is_empty());
    }
    assert!(
        ids.contains("docs-metadata"),
        "the docs provider registration was dropped: {ids:?}"
    );
}

#[test]
fn content_data_providers_are_registered_with_unique_ids() {
    let ctx = FixtureCtx::new();
    let providers = WebExtension::new().content_data_providers(&ctx);
    assert!(!providers.is_empty(), "no content data provider registered");

    let ids: HashSet<&str> = providers.iter().map(|p| p.provider_id()).collect();
    assert_eq!(ids.len(), providers.len(), "two providers share an id");
}

#[test]
fn component_renderers_are_registered_with_unique_ids_and_variables() {
    let ctx = FixtureCtx::new();
    let renderers = WebExtension::new().component_renderers(&ctx);
    assert!(!renderers.is_empty(), "no component renderer registered");

    let ids: HashSet<&str> = renderers.iter().map(|r| r.component_id()).collect();
    assert_eq!(ids.len(), renderers.len(), "two renderers share an id");

    let variables: HashSet<&str> = renderers.iter().map(|r| r.variable_name()).collect();
    assert_eq!(
        variables.len(),
        renderers.len(),
        "two renderers write to the same template variable"
    );
}

#[test]
fn template_data_extenders_are_registered() {
    let ctx = FixtureCtx::new();
    let extenders = WebExtension::new().template_data_extenders(&ctx);
    assert!(
        !extenders.is_empty(),
        "no template data extender registered"
    );

    let ids: HashSet<&str> = extenders.iter().map(|e| e.extender_id()).collect();
    assert_eq!(ids.len(), extenders.len(), "two extenders share an id");
}

#[test]
fn jobs_are_registered_with_unique_names() {
    let jobs = WebExtension::new().jobs();
    assert!(!jobs.is_empty(), "no job registered");

    let names: HashSet<&str> = jobs.iter().map(|j| j.name()).collect();
    assert_eq!(names.len(), jobs.len(), "two jobs share a name");
    for job in &jobs {
        assert!(!job.name().is_empty());
    }
    assert!(
        names.contains("publish_pipeline"),
        "the publish pipeline must be reachable from the extension: {names:?}"
    );
}

#[test]
fn prerenderers_claim_distinct_page_types_and_seeds_are_fixed() {
    let ctx = FixtureCtx::new();
    let extension = WebExtension::new();

    let prerenderers = extension.page_prerenderers(&ctx);
    let page_types: HashSet<&str> = prerenderers.iter().map(|p| p.page_type()).collect();
    assert_eq!(
        page_types.len(),
        prerenderers.len(),
        "two prerenderers claim the same page type"
    );

    let seeds = extension.seeds();
    let seed_ids: Vec<&str> = seeds.iter().map(|s| s.id).collect();
    assert_eq!(
        seed_ids,
        ["admin_oauth_client", "groups_projects"],
        "the boot-seed manifest is a contract: add or remove seeds deliberately"
    );
    for seed in &seeds {
        assert!(!seed.sql.trim().is_empty(), "seed {} is empty", seed.id);
    }
}
