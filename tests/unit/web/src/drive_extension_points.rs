//! The Drive pages own their UI.
//!
//! Core serves no document UI and no document modules: its `sp-*` web
//! components, stylesheets and `/assets/documents/` scripts are gone, and
//! dynapps vendors the two headless modules it needs (`documents-api.js`,
//! `documents-events.js`) under `js/documents/`. A template or script that
//! still names core's path renders an empty custom element or 404s, and
//! neither fails a build. Astound's own features reach the listing and the
//! workspace through their registries, so a feature wired in by name instead
//! would quietly stop being one an install can extend the same way.

use std::path::PathBuf;

use crate::support::{repo_root, walk};

const RETIRED_ELEMENTS: [&str; 11] = [
    "sp-comments",
    "sp-document-browser",
    "sp-document-editor",
    "sp-document-preview",
    "sp-history",
    "sp-presence-bar",
    "sp-proposal-list",
    "sp-share-dialog",
    "sp-sheet-grid",
    "sp-slide-text",
    "sp-subject-picker",
];

const VENDORED: [&str; 2] = [
    "documents/documents-api.js",
    "documents/documents-events.js",
];

fn sources() -> Vec<PathBuf> {
    let root = repo_root().join("storage/files");
    let mut out = Vec::new();
    walk(&root.join("admin"), "hbs", &mut out);
    walk(&root.join("js"), "js", &mut out);
    assert!(out.len() > 50, "front-end corpus looks empty");
    out
}

fn read(path: &std::path::Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

#[test]
fn no_page_loads_core_ui() {
    for path in sources() {
        let source = read(&path);
        let shown = path.display();
        assert!(
            !source.contains("assets/documents/"),
            "{shown} loads from core's retired /assets/documents/"
        );
        for element in RETIRED_ELEMENTS {
            for form in [
                format!("<{element} "),
                format!("<{element}>"),
                format!("'{element}'"),
                format!("\"{element}\""),
                format!("{element}.js"),
            ] {
                assert!(
                    !source.contains(&form),
                    "{shown} uses core's retired <{element}>"
                );
            }
        }
    }
    for module in VENDORED {
        let vendored = repo_root().join("storage/files/js").join(module);
        assert!(vendored.exists(), "the vendored {module} is missing");
    }
    let tokens = repo_root().join("storage/files/css/admin/01-tokens-documents.css");
    assert!(!tokens.exists(), "core token aliases outlived core's CSS");
}

fn script(relative: &str) -> String {
    read(&repo_root().join("storage/files/js").join(relative))
}

#[test]
fn extension_points_are_used_by_dynapps() {
    let registry = "drive-listing-registry.js";
    let badges = script("services/drive-class-badges.js");
    assert!(badges.contains(registry) && badges.contains("registerRowExtension("));
    let class = script("services/drive-folder-class.js");
    assert!(class.contains(registry) && class.contains("registerFolderMenuItem("));
    let page = script("pages/drive.js");
    assert!(page.contains("initClassBadges(") && page.contains("initFolderClass("));
    assert!(
        !page.contains("registerRowExtension(") && !page.contains("registerFolderMenuItem("),
        "the listing page registers its own features instead of importing them"
    );

    let commands = script("workspace/commands.js");
    assert!(commands.contains("./registry.js") && commands.contains("registerCommand("));
    let mut tabs = Vec::new();
    walk(
        &repo_root().join("storage/files/js/workspace"),
        "js",
        &mut tabs,
    );
    let registering = tabs
        .iter()
        .filter(|path| read(path).contains("registerDrawerTab("))
        .count();
    assert!(
        registering >= 3,
        "drawer tabs no longer register through registry.js"
    );
}
