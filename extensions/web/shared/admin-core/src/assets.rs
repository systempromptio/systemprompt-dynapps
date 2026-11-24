//! Static asset manifest for the admin dashboard.
//!
//! The admin crate owns the front-end it serves: per-page scripts under
//! `storage/files/js/pages/`, the live-page viewer shell under
//! `storage/files/js/workspace/live/` (its page runtime is also inlined by the
//! frame route) and the generated `admin-bundle.css`. The facade
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
    assets.extend(live_js_assets(&storage_js.join("workspace/live")));
    assets.extend(crate::vendor::assets::vendor_assets(&storage_js));
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
        page_js!(&pages, "admin-drive-document.js"),
        page_js!(&pages, "drive.js"),
        page_js!(&pages, "admin-entity-access.js"),
        page_js!(&pages, "admin-farm.js"),
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

fn live_js_assets(live: &Path) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::js(live.join("bridge.js"), "js/workspace/live/bridge.js"),
        AssetDefinition::js(live.join("labels.js"), "js/workspace/live/labels.js"),
        AssetDefinition::js(live.join("page.js"), "js/workspace/live/page.js"),
        AssetDefinition::js(live.join("prompts.js"), "js/workspace/live/prompts.js"),
        AssetDefinition::js(live.join("router.js"), "js/workspace/live/router.js"),
        AssetDefinition::js(
            live.join("runtime-boot.js"),
            "js/workspace/live/runtime-boot.js",
        ),
        AssetDefinition::js(
            live.join("runtime-core.js"),
            "js/workspace/live/runtime-core.js",
        ),
        AssetDefinition::js(
            live.join("runtime-mcp.js"),
            "js/workspace/live/runtime-mcp.js",
        ),
        AssetDefinition::js(
            live.join("runtime-use.js"),
            "js/workspace/live/runtime-use.js",
        ),
        AssetDefinition::js(live.join("shell.js"), "js/workspace/live/shell.js"),
        AssetDefinition::js(live.join("viewer.js"), "js/workspace/live/viewer.js"),
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
