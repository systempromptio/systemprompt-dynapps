//! The four account templates — profile, history, settings, setup — are held
//! to the design system rather than to a screenshot.
//!
//! Each of these pages carried its own header, its own card and its own list
//! markup before the rebuild, which is how the account section came to look
//! like a second product. Two properties keep that from happening again: every
//! account page opens with the shared page header and breadcrumb trail, and
//! none of the retired bespoke class families survives anywhere in the
//! templates. Both are textual facts about the template source, so they hold
//! without a server or a browser.

use crate::support::repo_root;

const PAGES: [&str; 4] = ["profile", "history", "settings", "setup"];

// Every class family the account pages used to define for themselves. Their
// stylesheets are deleted, so any surviving reference styles nothing at all.
const RETIRED: [&str; 14] = [
    "sp-profile-hero",
    "sp-profile-card",
    "sp-profile-row",
    "sp-quickstat",
    "sp-tile",
    "sp-modellist",
    "sp-agentlist",
    "sp-conv-table",
    "sp-deflist",
    "sp-settings-form-grid",
    "sp-settings-field",
    "sp-danger-zone-item",
    "sp-setup-stepper",
    "sp-setup-phase",
];

fn template(name: &str) -> String {
    let path = repo_root().join(format!("storage/files/admin/templates/{name}.hbs"));
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("account template {} is tracked: {e}", path.display()))
}

#[test]
fn every_account_page_opens_with_the_shared_header_and_trail() {
    let mut missing = Vec::new();
    for name in PAGES {
        let body = template(name);
        if !body.contains("components/breadcrumbs") {
            missing.push(format!("{name}: no breadcrumb trail"));
        }
        if !body.contains("components/page-header") {
            missing.push(format!("{name}: no shared page header"));
        }
    }
    assert!(
        missing.is_empty(),
        "account page(s) not wearing the shell:\n{}",
        missing.join("\n")
    );
}

#[test]
fn no_account_page_keeps_a_retired_bespoke_class() {
    let mut found = Vec::new();
    for name in PAGES {
        let body = template(name);
        for class in RETIRED {
            if body.contains(class) {
                found.push(format!("{name}: {class}"));
            }
        }
    }
    assert!(
        found.is_empty(),
        "retired account-page class(es) still referenced — their CSS is deleted, so they style \
         nothing:\n{}",
        found.join("\n")
    );
}

#[test]
fn the_retired_account_stylesheets_are_gone() {
    let css = repo_root().join("storage/files/css/admin");
    let mut left = Vec::new();
    for file in [
        "20-page-profile2.css",
        "20-page-profile3.css",
        "20-page-profile4-connect.css",
        "20-page-profile5-tiles.css",
        "20-page-profile6-sections.css",
        "20-page-profile7-deflist.css",
        "90-setup.css",
        "90-setup-actions.css",
    ] {
        if css.join(file).exists() {
            left.push(file);
        }
    }
    assert!(
        left.is_empty(),
        "account page stylesheets that the rebuild replaced are still on disk: {left:?}"
    );
}

#[test]
fn every_account_table_carries_a_screen_reader_caption() {
    let mut bare = Vec::new();
    for name in PAGES {
        let body = template(name);
        let tables = body.matches("{{#> components/table").count();
        let captions = body.matches("caption=").count();
        if tables > captions {
            bare.push(format!("{name}: {tables} tables, {captions} captions"));
        }
    }
    assert!(
        bare.is_empty(),
        "account page table(s) with no caption — a caption is the only name a screen reader has \
         for a table:\n{}",
        bare.join("\n")
    );
}
