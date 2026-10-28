//! Structural invariants of the drive pages.
//!
//! The rows are read from core's document API by the page scripts and set as
//! text into server-rendered `<template>` rows, so a template that
//! interpolates a row value, a script that writes HTML or skips the shared
//! transport, a page that stops naming one of the document routes, or a
//! sidebar entry outside the admin guard would each pass a compile and a
//! happy-path click-through. The admin gate on both routes is pinned by the
//! contract baseline.

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
    for name in ["drive", "drive-document"] {
        let source = read(&format!("storage/files/admin/templates/{name}.hbs"));
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
        "storage/files/js/pages/admin-drive.js",
        "storage/files/js/pages/admin-drive-document.js",
        "storage/files/js/services/drive-rows.js",
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
    for suffix in ["/preview", "/content", "/revisions", "/grants", "/presence"] {
        assert!(
            source.contains(&format!("format!(\"{{api}}{suffix}\")")),
            "the document page does not name {suffix}"
        );
    }
}

#[test]
fn the_sidebar_entry_sits_inside_the_admin_guard() {
    let source = read("storage/files/admin/partials/sidebar.hbs");
    let entry = source
        .find("href=\"/admin/drive\"")
        .expect("drive nav entry");
    let guard = source[..entry]
        .rfind("{{#if current_user.is_admin}}")
        .expect("admin guard");
    assert!(!source[guard..entry].contains("{{/if}}"));
}

#[test]
fn the_sidebar_entry_sits_inside_the_documents_switch() {
    let source = read("storage/files/admin/partials/sidebar.hbs");
    let entry = source
        .find("href=\"/admin/drive\"")
        .expect("drive nav entry");
    let switch = source[..entry]
        .rfind("{{#if documents_enabled}}")
        .expect("documents switch");
    assert!(!source[switch..entry].contains("{{/if}}"));
}

#[test]
fn the_page_scripts_are_registered_assets() {
    let admin = read("extensions/web/shared/admin-core/src/assets.rs");
    assert!(admin.contains("\"admin-drive.js\""));
    assert!(admin.contains("\"admin-drive-document.js\""));
    let services = read("extensions/web/site/src/assets/js_services.rs");
    assert!(services.contains("\"drive-rows.js\""));
}
