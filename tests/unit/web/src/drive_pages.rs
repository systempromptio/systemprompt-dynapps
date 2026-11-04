//! Structural invariants of the drive pages.
//!
//! The rows are read from core's document API by the page scripts and set as
//! text into server-rendered `<template>` rows, so a template that
//! interpolates a row value, a script that writes HTML or skips the shared
//! transport, a page that stops naming one of the document routes, or a
//! page that reaches back for core's retired document UI would each pass a
//! compile and a happy-path click-through. Who reaches each route is pinned
//! by the contract baseline.

use crate::support::repo_root;

fn read(relative: &str) -> String {
    let path = repo_root().join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn row_templates(source: &str) -> Vec<&str> {
    source
        .split("<template data-template=")
        .skip(1)
        .map(|rest| &rest[..rest.find("</template>").expect("closed template")])
        .collect()
}

#[test]
fn row_templates_carry_no_handlebars_values() {
    for name in [
        "templates/drive",
        "templates/drive-document",
        "partials/workspace/rows",
    ] {
        let source = read(&format!("storage/files/admin/{name}.hbs"));
        let rows = row_templates(&source);
        assert!(!rows.is_empty(), "{name}.hbs has no row templates");
        for row in rows {
            assert!(
                !row.contains("{{"),
                "{name}.hbs fills a row server-side; rows are set as text by the script"
            );
        }
    }
}

#[test]
fn the_scripts_set_text_and_never_html() {
    for path in [
        "storage/files/js/pages/drive.js",
        "storage/files/js/pages/admin-drive-document.js",
        "storage/files/js/services/drive-rows.js",
        "storage/files/js/services/drive-class-badges.js",
        "storage/files/js/services/drive-folder-class.js",
        "storage/files/js/services/drive-listing-create.js",
        "storage/files/js/services/drive-listing-menu.js",
        "storage/files/js/services/drive-listing-registry.js",
        "storage/files/js/services/drive-listing-view.js",
        "storage/files/js/workspace/commands.js",
        "storage/files/js/workspace/drawer.js",
        "storage/files/js/workspace/editors.js",
        "storage/files/js/workspace/palette.js",
        "storage/files/js/workspace/people.js",
        "storage/files/js/workspace/rail.js",
        "storage/files/js/workspace/shell.js",
    ] {
        let source = read(path);
        assert!(!source.contains("innerHTML"), "{path} writes HTML");
        assert!(
            !source.contains("fetch("),
            "{path} bypasses services/api.js"
        );
    }
}

#[test]
fn the_document_page_names_every_document_route() {
    let source = read("extensions/web/admin/platform/src/handlers/ssr/drive/detail.rs");
    for suffix in ["/preview", "/content", "/revisions", "/presence"] {
        assert!(
            source.contains(&format!("format!(\"{{api}}{suffix}\")")),
            "the document page does not name {suffix}"
        );
    }
}

#[test]
fn the_workspace_names_every_document_route() {
    let source = read("extensions/web/admin/platform/src/handlers/ssr/drive/workspace.rs");
    for suffix in [
        "/preview",
        "/content",
        "/revisions",
        "/presence",
        "/projection",
        "/access",
    ] {
        assert!(
            source.contains(&format!("format!(\"{{api}}{suffix}\")")),
            "the workspace does not name {suffix}"
        );
    }
}

#[test]
fn the_workspace_renders_without_the_console_shell() {
    let source = read("storage/files/admin/templates/drive-workspace.hbs");
    assert!(source.starts_with("{{#> workspace-layout"));
    let layout = read("storage/files/admin/partials/workspace-layout.hbs");
    assert!(
        !layout.contains("{{> sidebar"),
        "the workspace layout draws the sidebar"
    );
    assert!(
        !layout.contains("sp-topbar"),
        "the workspace layout draws the console top bar"
    );
    let handler = read("extensions/web/admin/platform/src/handlers/ssr/drive/workspace.rs");
    assert!(handler.contains("render_workspace_page("));
}

#[test]
fn every_menu_command_has_a_behaviour() {
    let handler = read("extensions/web/admin/platform/src/handlers/ssr/drive/workspace.rs");
    let commands = read("storage/files/js/workspace/commands.js");
    for id in handler
        .split("Command::menu(\"")
        .chain(handler.split("Command::palette(\"").skip(1))
        .skip(1)
        .map(|rest| &rest[..rest.find('"').expect("closed id")])
    {
        assert!(
            commands.contains(&format!("  {id},"))
                || commands.contains(&format!("'{id}':"))
                || commands.contains(&format!("  {id}: ")),
            "command {id} has no behaviour in commands.js"
        );
    }
}

#[test]
fn the_page_scripts_are_registered_assets() {
    let admin = read("extensions/web/shared/admin-core/src/assets.rs");
    assert!(admin.contains("\"drive.js\""));
    assert!(admin.contains("\"admin-drive-document.js\""));
    for retired in ["admin-drive.js", "my-drive.js", "my-drive-document.js"] {
        assert!(
            !admin.contains(&format!("\"{retired}\"")),
            "{retired} is still registered"
        );
    }
    let services = read("extensions/web/site/src/assets/js_services.rs");
    for name in [
        "drive-rows",
        "drive-class-badges",
        "drive-folder-class",
        "drive-listing-create",
        "drive-listing-menu",
        "drive-listing-registry",
        "drive-listing-view",
    ] {
        assert!(
            services.contains(&format!("svc_js!(p, \"{name}.js\")")),
            "{name}.js unregistered"
        );
    }
    for name in [
        "commands", "drawer", "editors", "palette", "people", "rail", "shell",
    ] {
        assert!(
            services.contains(&format!("ws_js!(p, \"{name}.js\")")),
            "{name}.js unregistered"
        );
    }
}

#[test]
fn the_drive_entry_is_for_everyone_while_the_store_is_on() {
    let source = read("storage/files/admin/partials/sidebar.hbs");
    assert!(
        !source.contains("href=\"/admin/drive\""),
        "the sidebar still links the retired admin drive"
    );
    let entry = source.find("href=\"/drive\"").expect("drive nav entry");
    let switch = source[..entry]
        .rfind("{{#if documents_enabled}}")
        .expect("documents switch");
    assert!(!source[switch..entry].contains("{{/if}}"));
    let guard = source[..entry].rfind("{{#if current_user.is_admin}}");
    assert!(
        guard.is_none_or(|at| source[at..entry].contains("{{/if}}")),
        "the drive entry sits inside the admin guard"
    );
}

#[test]
fn the_listing_authorizes_the_open_folder_as_the_viewer() {
    let source = read("extensions/web/admin/platform/src/handlers/ssr/drive/list.rs");
    let access = source
        .find(".folder_access_of(who, folder,")
        .expect("folder read as the viewer");
    let render = source
        .rfind("render_page_with_faults(")
        .expect("render call");
    assert!(access < render);
    assert!(source.contains("caller(&shell.user)"));
    assert!(
        !source.contains("Admin access required"),
        "the listing is admin-only again"
    );
}

#[test]
fn the_admin_only_controls_render_for_admins_alone() {
    let page = read("storage/files/admin/templates/drive.hbs");
    let policy = page
        .find("data-action=\"drive-policy\"")
        .expect("agent edit policy select");
    let cell = page[..policy].rfind("<td").expect("policy cell");
    assert!(
        page[cell..policy].contains("data-admin-only"),
        "the agent edit policy cell is not marked admin-only"
    );
    let view = read("storage/files/js/services/drive-listing-view.js");
    assert!(
        view.contains("[data-admin-only]") && view.contains("'admin' in dataset"),
        "a non-admin listing no longer drops the admin-only cells"
    );
}
