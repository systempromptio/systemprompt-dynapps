//! Shared JavaScript service module definitions.

use std::path::Path;
use systemprompt::extension::AssetDefinition;

macro_rules! svc_js {
    ($p:expr, $name:literal) => {
        AssetDefinition::js($p.join($name), concat!("js/services/", $name))
    };
}

macro_rules! ws_js {
    ($p:expr, $name:literal) => {
        AssetDefinition::js($p.join($name), concat!("js/workspace/", $name))
    };
}

macro_rules! site_js {
    ($p:expr, $name:literal) => {
        AssetDefinition::js($p.join($name), concat!("js/site/", $name))
    };
}

#[doc(hidden)]
pub fn public_js_assets(storage_js: &Path) -> Vec<AssetDefinition> {
    let site = storage_js.join("site");
    vec![
        AssetDefinition::js(storage_js.join("analytics.js"), "js/analytics.js"),
        AssetDefinition::js(storage_js.join("docs.js"), "js/docs.js"),
        AssetDefinition::js(storage_js.join("mobile-menu.js"), "js/mobile-menu.js"),
        AssetDefinition::js(storage_js.join("homepage.js"), "js/homepage.js"),
        AssetDefinition::js(storage_js.join("motion-flag.js"), "js/motion-flag.js"),
        site_js!(&site, "analytics-handlers.js"),
        site_js!(&site, "analytics-metrics.js"),
        site_js!(&site, "analytics-state.js"),
        site_js!(&site, "analytics-transport.js"),
        site_js!(&site, "copy-buttons.js"),
        site_js!(&site, "docs-evidence-gallery.js"),
        site_js!(&site, "docs-export.js"),
        site_js!(&site, "docs-nav.js"),
        site_js!(&site, "docs-pagination.js"),
        site_js!(&site, "docs-toc.js"),
        site_js!(&site, "dom-throttle.js"),
        site_js!(&site, "mcp-connect-modal.js"),
        site_js!(&site, "status-api.js"),
        site_js!(&site, "status-card.js"),
        site_js!(&site, "status-render.js"),
    ]
}

#[doc(hidden)]
pub fn service_js_assets(storage_js: &Path) -> Vec<AssetDefinition> {
    let p = storage_js.join("services");
    let mut v = service_core_js(&p);
    v.extend(service_utils_js(storage_js));
    v.extend(workspace_js(&storage_js.join("workspace")));
    v.extend(documents_js(&storage_js.join("documents")));
    v
}

fn documents_js(p: &Path) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::js(p.join("documents-api.js"), "js/documents/documents-api.js"),
        AssetDefinition::js(
            p.join("documents-events.js"),
            "js/documents/documents-events.js",
        ),
    ]
}

fn workspace_js(p: &Path) -> Vec<AssetDefinition> {
    let mut v = editor_js(p);
    v.extend([
        ws_js!(p, "upload/complete.js"),
        ws_js!(p, "upload/errors.js"),
        ws_js!(p, "upload/hash.js"),
        ws_js!(p, "upload/import.js"),
        ws_js!(p, "upload/progress.js"),
        ws_js!(p, "upload/queue.js"),
        ws_js!(p, "upload/resume.js"),
        ws_js!(p, "upload/session.js"),
        ws_js!(p, "upload/sha256.js"),
        ws_js!(p, "upload/transport.js"),
        ws_js!(p, "commands.js"),
        ws_js!(p, "drawer-access.js"),
        ws_js!(p, "drawer-access-requests.js"),
        ws_js!(p, "drawer-comments.js"),
        ws_js!(p, "drawer-details.js"),
        ws_js!(p, "drawer-owner.js"),
        ws_js!(p, "drawer-proposals.js"),
        ws_js!(p, "drawer-request-access.js"),
        ws_js!(p, "drawer-sharing.js"),
        ws_js!(p, "menu.js"),
        ws_js!(p, "pane-dom.js"),
        ws_js!(p, "panes.js"),
        ws_js!(p, "presence.js"),
        ws_js!(p, "registry.js"),
        ws_js!(p, "drawer-class.js"),
        ws_js!(p, "drawer-share.js"),
        ws_js!(p, "drawer-versions.js"),
        ws_js!(p, "drawer.js"),
        ws_js!(p, "editors.js"),
        ws_js!(p, "palette.js"),
        ws_js!(p, "people-picker.js"),
        ws_js!(p, "people.js"),
        ws_js!(p, "rail.js"),
        ws_js!(p, "shell.js"),
    ]);
    v
}

fn editor_js(p: &Path) -> Vec<AssetDefinition> {
    vec![
        ws_js!(p, "doc/actions.js"),
        ws_js!(p, "doc/awareness.js"),
        ws_js!(p, "doc/opaque.js"),
        ws_js!(p, "doc/remote.js"),
        ws_js!(p, "sheet/editor.js"),
        ws_js!(p, "sheet/cell-editor.js"),
        ws_js!(p, "sheet/clipboard.js"),
        ws_js!(p, "sheet/formula.js"),
        ws_js!(p, "sheet/grid.js"),
        ws_js!(p, "sheet/keys.js"),
        ws_js!(p, "sheet/layout.js"),
        ws_js!(p, "sheet/mount.js"),
        ws_js!(p, "sheet/presence.js"),
        ws_js!(p, "sheet/projection.js"),
        ws_js!(p, "sheet/replica.js"),
        ws_js!(p, "sheet/selection.js"),
        ws_js!(p, "slides/editor.js"),
        ws_js!(p, "slides/cursors.js"),
        ws_js!(p, "slides/deck.js"),
        ws_js!(p, "slides/filmstrip.js"),
        ws_js!(p, "slides/frames.js"),
        ws_js!(p, "slides/mount.js"),
        ws_js!(p, "slides/paragraphs.js"),
        ws_js!(p, "slides/present.js"),
        ws_js!(p, "slides/shape-text.js"),
        ws_js!(p, "slides/stage.js"),
        ws_js!(p, "doc/comments.js"),
        ws_js!(p, "doc/composer.js"),
        ws_js!(p, "doc/contended.js"),
        ws_js!(p, "doc/editor.js"),
        ws_js!(p, "doc/ids.js"),
        ws_js!(p, "doc/live.js"),
        ws_js!(p, "doc/loro.js"),
        ws_js!(p, "doc/mentions.js"),
        ws_js!(p, "doc/menus.js"),
        ws_js!(p, "doc/model.js"),
        ws_js!(p, "doc/mount.js"),
        ws_js!(p, "doc/schema.js"),
        ws_js!(p, "doc/session.js"),
        ws_js!(p, "doc/source.js"),
        ws_js!(p, "doc/table-model.js"),
        ws_js!(p, "doc/toolbar.js"),
        ws_js!(p, "doc/write.js"),
    ]
}

fn service_core_js(p: &Path) -> Vec<AssetDefinition> {
    vec![
        svc_js!(p, "api.js"),
        svc_js!(p, "attention.js"),
        svc_js!(p, "auth.js"),
        svc_js!(p, "bootstrap.js"),
        svc_js!(p, "confirm.js"),
        svc_js!(p, "doc-events.js"),
        svc_js!(p, "document-class.js"),
        svc_js!(p, "connector-labels.js"),
        svc_js!(p, "drive-class-badges.js"),
        svc_js!(p, "drive-folder-class.js"),
        svc_js!(p, "drive-listing-create.js"),
        svc_js!(p, "drive-listing-menu.js"),
        svc_js!(p, "drive-listing-registry.js"),
        svc_js!(p, "drive-listing-view.js"),
        svc_js!(p, "drive-paper.js"),
        svc_js!(p, "drive-rail.js"),
        svc_js!(p, "drive-rows.js"),
        svc_js!(p, "dropdown.js"),
        svc_js!(p, "events.js"),
        svc_js!(p, "export-columns.js"),
        svc_js!(p, "export-formats.js"),
        svc_js!(p, "export-preview.js"),
        svc_js!(p, "export-scope.js"),
        svc_js!(p, "export-url.js"),
        svc_js!(p, "live-number.js"),
        svc_js!(p, "live-poll.js"),
        svc_js!(p, "live-status.js"),
        svc_js!(p, "export.js"),
        svc_js!(p, "filter-ribbon.js"),
        svc_js!(p, "flash.js"),
        svc_js!(p, "form-confirm.js"),
        svc_js!(p, "header-actions.js"),
        svc_js!(p, "header-search-list.js"),
        svc_js!(p, "header-search.js"),
        svc_js!(p, "nav-groups.js"),
        svc_js!(p, "notification-bell.js"),
        svc_js!(p, "scope.js"),
        svc_js!(p, "sidebar.js"),
        svc_js!(p, "table-cards.js"),
        svc_js!(p, "table-expand.js"),
        svc_js!(p, "toast.js"),
        svc_js!(p, "validity.js"),
    ]
}

fn service_utils_js(storage_js: &Path) -> Vec<AssetDefinition> {
    vec![
        AssetDefinition::js(
            storage_js.join("components/sp-toast.js"),
            "js/components/sp-toast.js",
        ),
        AssetDefinition::js(
            storage_js.join("components/sp-confirm-dialog.js"),
            "js/components/sp-confirm-dialog.js",
        ),
        AssetDefinition::js(
            storage_js.join("components/sp-confirm-dialog-view.js"),
            "js/components/sp-confirm-dialog-view.js",
        ),
        AssetDefinition::js(
            storage_js.join("components/sp-copy.js"),
            "js/components/sp-copy.js",
        ),
        AssetDefinition::js(
            storage_js.join("components/sp-tabs.js"),
            "js/components/sp-tabs.js",
        ),
        AssetDefinition::js(
            storage_js.join("components/sp-chart.js"),
            "js/components/sp-chart.js",
        ),
        AssetDefinition::js(
            storage_js.join("components/sp-evidence-lightbox.js"),
            "js/components/sp-evidence-lightbox.js",
        ),
        AssetDefinition::js(
            storage_js.join("components/sp-transcript.js"),
            "js/components/sp-transcript.js",
        ),
        AssetDefinition::js(
            storage_js.join("components/sp-sync-plane.js"),
            "js/components/sp-sync-plane.js",
        ),
        AssetDefinition::js(
            storage_js.join("components/sp-access-review.js"),
            "js/components/sp-access-review.js",
        ),
        AssetDefinition::js(
            storage_js.join("components/sp-help.js"),
            "js/components/sp-help.js",
        ),
        AssetDefinition::js(
            storage_js.join("components/sp-table-select.js"),
            "js/components/sp-table-select.js",
        ),
        AssetDefinition::js(
            storage_js.join("components/sp-report-poll.js"),
            "js/components/sp-report-poll.js",
        ),
        AssetDefinition::js(
            storage_js.join("components/sp-chart-scale.js"),
            "js/components/sp-chart-scale.js",
        ),
        AssetDefinition::js(
            storage_js.join("components/sp-chart-draw.js"),
            "js/components/sp-chart-draw.js",
        ),
        AssetDefinition::js(
            storage_js.join("components/sp-chart-tooltip.js"),
            "js/components/sp-chart-tooltip.js",
        ),
        AssetDefinition::js(
            storage_js.join("components/sp-chart-update.js"),
            "js/components/sp-chart-update.js",
        ),
    ]
}
