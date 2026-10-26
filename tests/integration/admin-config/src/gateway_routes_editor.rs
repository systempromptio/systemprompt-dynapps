//! Dashboard route edits are durable only when the database and boot YAML
//! agree; partial form saves must retain policy blocks that the form omits.

use systemprompt_web_admin_core::types::{GatewayRouteView, RouteDeploymentView};
use systemprompt_web_admin_governance::repositories::gateway_routes::editor::{
    create_route_entry, delete_route_at, reorder_route_positions, update_route_at,
};
use systemprompt_web_admin_governance::repositories::gateway_routes::rows::list_gateway_routes;

use crate::tempdb::TempDb;

fn route(id: &str, pattern: &str) -> GatewayRouteView {
    GatewayRouteView {
        id: id.into(),
        name: Some(format!("Route {id}")),
        description: Some("initial description".into()),
        models: vec![pattern.into()],
        deployments: vec![
            RouteDeploymentView {
                endpoint: "anthropic/direct".into(),
                weight: Some(3),
            },
            RouteDeploymentView {
                endpoint: "openai/direct".into(),
                weight: Some(1),
            },
        ],
        when: Some(serde_yaml::from_str("stream: true").expect("when yaml")),
        requires: Some(
            serde_yaml::from_str("european: true\nno_retain: false").expect("requires yaml"),
        ),
        strategy: Some("weighted".into()),
        context_fallbacks: vec!["claude-long-context".into()],
        ..GatewayRouteView::default()
    }
}

#[tokio::test]
async fn route_editor_preserves_hidden_policy_blocks_and_regenerates_dispatch_order() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    // The extension seed is not part of this editor scenario; a clean route
    // list makes the positional API assertions describe the rows we own.
    sqlx::query("DELETE FROM gateway_routes")
        .execute(&*db.pool)
        .await
        .expect("clear seeded routes");
    let dir = tempfile::tempdir().expect("gateway dir");
    let gateway = dir.path().join("gateway.yaml");
    std::fs::write(&gateway, "gateway:\n  enabled: true\n  routes: []\n").expect("gateway fixture");

    assert_eq!(
        create_route_entry(&db.pool, &gateway, &route("primary", "claude-*"))
            .await
            .expect("create primary"),
        0
    );
    let mut partial = route("", "claude-updated-*");
    partial.description = Some("edited from dashboard".into());
    partial.when = None;
    partial.requires = None;
    partial.strategy = None;
    partial.context_fallbacks.clear();
    update_route_at(&db.pool, &gateway, 0, &partial)
        .await
        .expect("save partial form route");

    create_route_entry(&db.pool, &gateway, &route("secondary", "gpt-*"))
        .await
        .expect("create secondary");
    reorder_route_positions(&db.pool, &gateway, &[1, 0])
        .await
        .expect("reorder routes");
    let rows = list_gateway_routes(&db.pool).await.expect("read routes");
    assert_eq!(
        rows.iter().map(|r| r.route.id.as_str()).collect::<Vec<_>>(),
        ["secondary", "primary"]
    );
    let primary = rows
        .iter()
        .find(|r| r.route.id == "primary")
        .expect("primary row");
    assert_eq!(
        primary.route.description.as_deref(),
        Some("edited from dashboard")
    );
    assert_eq!(primary.route.models, ["claude-updated-*"]);
    assert_eq!(primary.route.deployments, route("x", "x").deployments);
    assert_eq!(primary.route.when, route("x", "x").when);
    assert_eq!(primary.route.requires, route("x", "x").requires);
    assert_eq!(primary.route.strategy, route("x", "x").strategy);
    assert_eq!(
        primary.route.context_fallbacks,
        route("x", "x").context_fallbacks
    );
    let rendered = std::fs::read_to_string(&gateway).expect("regenerated gateway");
    assert!(
        rendered.find("secondary").expect("secondary yaml")
            < rendered.find("primary").expect("primary yaml")
    );

    delete_route_at(&db.pool, &gateway, 0)
        .await
        .expect("delete first route");
    let remaining = list_gateway_routes(&db.pool)
        .await
        .expect("read remaining route");
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].position, 0);
    assert_eq!(remaining[0].route.id, "primary");
    db.cleanup().await;
}
