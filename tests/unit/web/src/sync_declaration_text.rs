//! Export → import is a no-op for an install plane: its database rendered as
//! its file, parsed back as an uploaded declaration, drifts by nothing against
//! the rows it was rendered from.

use systemprompt_web_admin_core::repositories::sync::plane::DeclarationSource;
use systemprompt_web_admin_identity::repositories::sync_planes::groups_db::{
    parse_groups_doc, render_groups_export,
};
use systemprompt_web_admin_identity::repositories::sync_planes::groups_drift::{
    MappingRow, MemberSetRow, compute_groups_drift,
};

#[test]
fn groups_export_then_import_is_clean() {
    let sets = vec![
        MemberSetRow {
            kind: "group".to_owned(),
            id: "europe-devs".to_owned(),
            name: "Europe developers".to_owned(),
            description: Some("Commerce Cloud developers.".to_owned()),
            source: "yaml".to_owned(),
            is_system: false,
        },
        MemberSetRow {
            kind: "project".to_owned(),
            id: "commerce".to_owned(),
            name: "Commerce".to_owned(),
            description: None,
            source: "dashboard".to_owned(),
            is_system: false,
        },
    ];
    let mappings = vec![MappingRow {
        kind: "group".to_owned(),
        ad_group: "Systemprompt-Commerce".to_owned(),
        set_id: "europe-devs".to_owned(),
        source: "yaml".to_owned(),
    }];
    let exported = render_groups_export(&sets, &mappings);
    let doc = parse_groups_doc(&exported).expect("export parses as a declaration");
    let drift = compute_groups_drift(&doc, &sets, &mappings);
    assert!(drift.is_clean(), "{:?}", drift.rows);
}

#[test]
fn the_default_declaration_source_is_disk() {
    assert!(DeclarationSource::default().is_disk());
    assert!(!DeclarationSource::Text("x").is_disk());
}
