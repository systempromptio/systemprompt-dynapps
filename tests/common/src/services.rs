//! Services views built from a services tree on disk.
//!
//! Production code receives its `ServicesView` from core's boot; a test
//! builds the same view from a fixture tree and hands it to the code under
//! test, so no process-wide services state is installed.

use std::path::Path;
use std::sync::Arc;

use systemprompt::config::{LoadedProfile, SecretsService};
use systemprompt::loader::ServicesState;
use systemprompt::manifest::services::ServicesView;
use systemprompt::traits::SecretStore;

fn fixture_profile() -> (tempfile::TempDir, Arc<LoadedProfile>) {
    let dir = tempfile::tempdir().expect("profile directory");
    let yaml = crate::secrets_fixture::FIXTURE_PROFILE
        .replace("__PROFILE_DIR__", &dir.path().to_string_lossy())
        .replace("__REPO__", &crate::repo_root().to_string_lossy());
    let path = dir.path().join("profile.yaml");
    std::fs::write(&path, yaml).expect("fixture profile");
    let profile = LoadedProfile::from_path(&path).expect("load the fixture profile");
    (dir, profile)
}

pub fn services_view(config: &Path) -> ServicesView {
    let (_dir, profile) = fixture_profile();
    let state = ServicesState::load_from_path(config, &profile).expect("load the services fixture");
    state.view()
}

pub fn fixture_secret_store() -> Arc<dyn SecretStore> {
    secret_store_from(&crate::secrets_fixture::secrets_document(None, &[]))
}

pub fn secret_store_without_master_key() -> Arc<dyn SecretStore> {
    let mut doc: serde_json::Value =
        serde_json::from_str(&crate::secrets_fixture::secrets_document(None, &[]))
            .expect("fixture secrets document");
    doc.as_object_mut()
        .expect("a secrets object")
        .remove("encryption_master_key");
    secret_store_from(&doc.to_string())
}

fn secret_store_from(document: &str) -> Arc<dyn SecretStore> {
    let (_dir, profile) = fixture_profile();
    let secrets = serde_json::from_str(document).expect("fixture secrets document");
    SecretsService::from_secrets(profile, secrets)
}

pub fn empty_services_view() -> ServicesView {
    let directory = tempfile::tempdir().expect("services fixture dir");
    let path = directory.path().join("config.yaml");
    std::fs::write(&path, "{}").expect("write services fixture");
    services_view(&path)
}

pub fn extension_services(
    pool: &Arc<sqlx::PgPool>,
    view: ServicesView,
) -> systemprompt_web_admin_core::extension_services::ExtensionServices {
    use systemprompt::governance::authz::{AuthzHookContext, NullAuditSink, SubjectProviderSet};
    use systemprompt::models::authz::SubjectSourceHandle;
    let subjects = SubjectSourceHandle::default();
    let providers = SubjectProviderSet::discover(&AuthzHookContext {
        pool: Arc::clone(pool),
        sink: Arc::new(NullAuditSink),
        services: view.clone(),
        subjects: subjects.clone(),
        secrets: fixture_secret_store(),
    });
    subjects
        .bind(Arc::new(providers))
        .expect("a fresh handle binds once");
    systemprompt_web_admin_core::extension_services::ExtensionServices::new(
        view,
        Arc::new(subjects),
    )
}

pub fn offline_extension_services(
    view: ServicesView,
) -> systemprompt_web_admin_core::extension_services::ExtensionServices {
    systemprompt_web_admin_core::extension_services::ExtensionServices::new(
        view,
        Arc::new(systemprompt::governance::authz::SubjectProviderSet::from_providers(Vec::new())),
    )
}

pub fn subject_source(
    pool: Arc<sqlx::PgPool>,
    services: ServicesView,
    secrets: Arc<dyn SecretStore>,
) -> Arc<dyn systemprompt::models::authz::SubjectAttributeSource> {
    use systemprompt::governance::authz::{AuthzHookContext, NullAuditSink, SubjectProviderSet};
    let handle = systemprompt::models::authz::SubjectSourceHandle::default();
    let set = SubjectProviderSet::discover(&AuthzHookContext {
        pool,
        sink: Arc::new(NullAuditSink),
        services,
        subjects: handle.clone(),
        secrets,
    });
    handle
        .bind(Arc::new(set))
        .expect("a fresh subject handle is unbound");
    Arc::new(handle)
}
