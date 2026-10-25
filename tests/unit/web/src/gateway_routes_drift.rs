//! `gateway.yaml` against `gateway_routes`: the declaration parses every
//! field core's `GatewayRoute` carries, the drift names what moved and
//! sees order, and the render puts the rows back into the file without
//! touching the settings above them.

use chrono::Utc;
use systemprompt_web_admin_core::types::{GatewayRouteView, RouteDeploymentView};
use systemprompt_web_admin_governance::repositories::gateway_routes::declared::{
    DeclaredRoutes, parse_declared_routes,
};
use systemprompt_web_admin_governance::repositories::gateway_routes::drift::compute_route_drift;
use systemprompt_web_admin_governance::repositories::gateway_routes::render::render_routes_export;
use systemprompt_web_admin_governance::repositories::gateway_routes::rows::{
    RouteRow, SOURCE_CODE, SOURCE_DASHBOARD,
};

const DECLARED: &str = r"# Gateway routes — header comment.
gateway:
  enabled: true
  routes:
  - id: claude-star
    label: Claude
    models: claude-*
    when:
      scope:
        project: storefront
    deployments:
    - anthropic/api
    - endpoint: vertex/messages
      weight: 2
    strategy: weighted
  - id: go-alias
    alias:
      requested: go-deepseek
      model: deepseek-v4-pro
    deployments:
    - opencode-go/chat
  - id: gpt
    models:
    - gpt-4.1*
    requires:
      european: true
  default_model: claude-sonnet-5-5
  quota_fault_mode: closed
";

fn rows_from(declared: &DeclaredRoutes, source: &str) -> Vec<RouteRow> {
    declared
        .routes
        .iter()
        .enumerate()
        .map(|(i, d)| RouteRow {
            route: d.route.clone(),
            position: i32::try_from(i).expect("small index"),
            source: source.to_owned(),
            updated_at: Utc::now(),
        })
        .collect()
}

#[test]
fn declaration_carries_every_route_field() {
    let declared = parse_declared_routes(DECLARED).expect("parses");
    assert_eq!(declared.routes.len(), 3);
    let claude = &declared.routes[0].route;
    assert_eq!(claude.id, "claude-star");
    assert_eq!(claude.name.as_deref(), Some("Claude"));
    assert_eq!(claude.models, vec!["claude-*".to_owned()]);
    assert_eq!(
        claude.deployments,
        vec![
            RouteDeploymentView {
                endpoint: "anthropic/api".to_owned(),
                weight: None,
            },
            RouteDeploymentView {
                endpoint: "vertex/messages".to_owned(),
                weight: Some(2),
            },
        ]
    );
    assert_eq!(claude.strategy.as_deref(), Some("weighted"));
    assert!(claude.when.is_some(), "scope predicates are carried");
    let alias = declared.routes[1]
        .route
        .alias
        .as_ref()
        .expect("alias carried");
    assert_eq!(alias.requested, "go-deepseek");
    assert_eq!(alias.model, "deepseek-v4-pro");
    assert!(declared.routes[2].route.requires.is_some());
}

#[test]
fn declared_hash_is_blind_to_comments_and_sees_order() {
    let a = parse_declared_routes(DECLARED).expect("parses");
    let b = parse_declared_routes(&DECLARED.replace("# Gateway routes — header comment.\n", ""))
        .expect("parses");
    assert_eq!(a.declared_hash(), b.declared_hash());
    let mut reordered = a.clone();
    reordered.routes.swap(0, 1);
    assert_ne!(a.declared_hash(), reordered.declared_hash());
}

#[test]
fn parse_refuses_a_repeated_id_and_a_route_without_an_id() {
    let dup = DECLARED.replace("  - id: gpt\n", "  - id: claude-star\n");
    assert!(
        parse_declared_routes(&dup)
            .expect_err("refused")
            .to_string()
            .contains("repeats id")
    );
    let bare = "gateway:\n  routes:\n  - models: x-*\n";
    assert!(
        parse_declared_routes(bare)
            .expect_err("refused")
            .to_string()
            .contains("routes[0]")
    );
}

#[test]
fn in_step_rows_are_clean() {
    let declared = parse_declared_routes(DECLARED).expect("parses");
    let drift = compute_route_drift(&declared, &rows_from(&declared, SOURCE_CODE));
    assert!(drift.is_clean(), "{drift:?}");
}

#[test]
fn drift_names_missing_orphan_changed_fields_and_console_origin() {
    let declared = parse_declared_routes(DECLARED).expect("parses");
    let mut rows = rows_from(&declared, SOURCE_CODE);
    rows.remove(2);
    rows[0].route.deployments = vec![RouteDeploymentView {
        endpoint: "cerebras/chat".to_owned(),
        weight: None,
    }];
    rows[0].source = SOURCE_DASHBOARD.to_owned();
    rows.push(RouteRow {
        route: GatewayRouteView {
            id: "console-only".to_owned(),
            models: vec!["o4-*".to_owned()],
            ..Default::default()
        },
        position: 9,
        source: SOURCE_DASHBOARD.to_owned(),
        updated_at: Utc::now(),
    });
    let drift = compute_route_drift(&declared, &rows);
    assert_eq!(drift.missing_in_db, vec!["gpt".to_owned()]);
    assert_eq!(drift.only_in_db.len(), 1);
    assert!(drift.only_in_db[0].dashboard);
    assert_eq!(drift.changed.len(), 1);
    assert_eq!(drift.changed[0].fields, vec!["deployments".to_owned()]);
    assert!(drift.changed[0].dashboard);
    assert!(drift.changed[0].in_db.contains("→ cerebras/chat"));
    assert!(drift.reordered.is_none(), "shared routes keep their order");
}

#[test]
fn drift_reports_order_once_not_per_route() {
    let declared = parse_declared_routes(DECLARED).expect("parses");
    let mut rows = rows_from(&declared, SOURCE_CODE);
    rows.swap(0, 2);
    let drift = compute_route_drift(&declared, &rows);
    assert!(drift.changed.is_empty());
    let (code, db) = drift.reordered.expect("order differs");
    assert_eq!(code[0], "claude-star");
    assert_eq!(db[0], "gpt");
}

#[test]
fn render_replaces_the_sequence_and_keeps_settings_and_header() {
    let declared = parse_declared_routes(DECLARED).expect("parses");
    let mut rows = rows_from(&declared, SOURCE_CODE);
    rows.truncate(1);
    let out = render_routes_export(DECLARED, &rows).expect("renders");
    assert!(out.starts_with("# Gateway routes — header comment.\n"));
    assert!(out.contains("quota_fault_mode: closed"));
    assert!(out.contains("default_model: claude-sonnet-5-5"));
    assert!(out.contains("- anthropic/api"));
    assert!(out.contains("endpoint: vertex/messages"));
    assert!(out.contains("scope:"));
    assert!(!out.contains("go-deepseek"));
    assert!(!out.contains("gpt-4.1*"));
    let back = parse_declared_routes(&out).expect("round-trips");
    assert_eq!(back.routes.len(), 1);
    assert_eq!(back.declared_hash(), {
        let mut one = declared;
        one.routes.truncate(1);
        one.declared_hash()
    });
}

#[test]
fn render_writes_every_routes_declared_id() {
    let declared = parse_declared_routes(DECLARED).expect("parses");
    let rows = rows_from(&declared, SOURCE_CODE);
    let out = render_routes_export(DECLARED, &rows).expect("renders");
    for id in ["claude-star", "go-alias", "gpt"] {
        assert!(out.contains(&format!("id: {id}")), "{id} written: {out}");
    }
    let back = parse_declared_routes(&out).expect("round-trips");
    assert_eq!(back.declared_hash(), declared.declared_hash());
}
