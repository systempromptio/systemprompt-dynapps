//! Services views built from a services tree on disk.
//!
//! Production code receives its `ServicesView` from core's boot; a test
//! builds the same view from a fixture tree and hands it to the code under
//! test, so no process-wide services state is installed.

use std::path::Path;

use systemprompt::loader::ServicesState;
use systemprompt::manifest::services::ServicesView;

pub fn services_view(config: &Path) -> ServicesView {
    let state = ServicesState::load_from_path(config).expect("load the services fixture");
    state.view()
}

pub fn empty_services_view() -> ServicesView {
    let directory = tempfile::tempdir().expect("services fixture dir");
    let path = directory.path().join("config.yaml");
    std::fs::write(&path, "{}").expect("write services fixture");
    services_view(&path)
}

pub fn extension_services(
    pool: &std::sync::Arc<sqlx::PgPool>,
    view: ServicesView,
) -> systemprompt_web_admin_core::extension_services::ExtensionServices {
    use systemprompt::governance::authz::{AuthzHookContext, NullAuditSink, SubjectProviderSet};
    use systemprompt::models::authz::SubjectSourceHandle;
    let subjects = SubjectSourceHandle::default();
    let providers = SubjectProviderSet::discover(&AuthzHookContext {
        pool: std::sync::Arc::clone(pool),
        sink: std::sync::Arc::new(NullAuditSink),
        services: view.clone(),
        subjects: subjects.clone(),
    });
    subjects
        .bind(std::sync::Arc::new(providers))
        .expect("a fresh handle binds once");
    systemprompt_web_admin_core::extension_services::ExtensionServices::new(
        view,
        std::sync::Arc::new(subjects),
    )
}

pub fn offline_extension_services(
    view: ServicesView,
) -> systemprompt_web_admin_core::extension_services::ExtensionServices {
    systemprompt_web_admin_core::extension_services::ExtensionServices::new(
        view,
        std::sync::Arc::new(
            systemprompt::governance::authz::SubjectProviderSet::from_providers(Vec::new()),
        ),
    )
}
