//! Static asset manifest for the admin dashboard.
//!
//! The admin crate owns the front-end it serves: per-page scripts under
//! `storage/files/js/pages/` and the generated `admin-bundle.css`. The facade
//! merges this with the public-site manifest in `required_assets`.
//! `admin-bundle.css` registers as optional: `bundle_admin_css` generates it,
//! and a fresh checkout has not run that job yet.

use std::path::Path;

use systemprompt::extension::{AssetDefinition, AssetPaths, AssetType};

macro_rules! page_js {
    ($p:expr, $name:literal) => {
        AssetDefinition::js($p.join($name), concat!("js/pages/", $name))
    };
}

#[must_use]
pub fn admin_assets(paths: &dyn AssetPaths) -> Vec<AssetDefinition> {
    let storage_css = paths.storage_files().join("css");
    let storage_js = paths.storage_files().join("js");

    let mut assets = page_js_assets(&storage_js);
    assets.push(admin_bundle_css(&storage_css));
    assets.push(AssetDefinition::css(
        storage_css.join("requirements-review.css"),
        "css/requirements-review.css",
    ));
    assets.push(AssetDefinition::js(
        storage_js.join("admin-report-bridge.js"),
        "js/admin-report-bridge.js",
    ));
    assets.push(AssetDefinition::js(
        storage_js.join("admin-report-shell.js"),
        "js/admin-report-shell.js",
    ));
    assets
}

fn page_js_assets(storage_js: &Path) -> Vec<AssetDefinition> {
    let pages = storage_js.join("pages");
    vec![
        page_js!(&pages, "admin-access-control.js"),
        page_js!(&pages, "admin-overview.js"),
        page_js!(&pages, "admin-overview-fetch.js"),
        page_js!(&pages, "admin-overview-morph.js"),
        page_js!(&pages, "admin-sync.js"),
        page_js!(&pages, "admin-approvals.js"),
        page_js!(&pages, "admin-contexts.js"),
        page_js!(&pages, "admin-devices.js"),
        page_js!(&pages, "admin-drive.js"),
        page_js!(&pages, "admin-drive-document.js"),
        page_js!(&pages, "my-drive.js"),
        page_js!(&pages, "my-drive-document.js"),
        page_js!(&pages, "admin-entity-access.js"),
        page_js!(&pages, "admin-farm-agent.js"),
        page_js!(&pages, "admin-farm-evals.js"),
        page_js!(&pages, "admin-farm-live-list.js"),
        page_js!(&pages, "admin-farm-run.js"),
        page_js!(&pages, "admin-farm-run-fetch.js"),
        page_js!(&pages, "admin-gateway.js"),
        page_js!(&pages, "admin-gateway-policies.js"),
        page_js!(&pages, "admin-governance.js"),
        page_js!(&pages, "admin-group-access.js"),
        page_js!(&pages, "admin-group-assign.js"),
        page_js!(&pages, "admin-group-detail.js"),
        page_js!(&pages, "admin-groups.js"),
        page_js!(&pages, "admin-login.js"),
        page_js!(&pages, "admin-project-detail.js"),
        page_js!(&pages, "admin-project-report.js"),
        page_js!(&pages, "admin-projects.js"),
        page_js!(&pages, "admin-requests.js"),
        page_js!(&pages, "admin-roles.js"),
        page_js!(&pages, "admin-service-accounts.js"),
        page_js!(&pages, "admin-settings.js"),
        page_js!(&pages, "admin-subscriptions.js"),
        page_js!(&pages, "admin-user-access.js"),
        page_js!(&pages, "admin-user-detail.js"),
        page_js!(&pages, "admin-users.js"),
        page_js!(&pages, "connect-code.js"),
        page_js!(&pages, "connectors.js"),
    ]
}

fn admin_bundle_css(storage_css: &Path) -> AssetDefinition {
    AssetDefinition::builder(
        storage_css.join("admin-bundle.css"),
        "css/admin-bundle.css",
        AssetType::Css,
    )
    .optional()
    .build()
}
