//! Gateway route CRUD and settings against a `gateway.yaml` on disk.

use std::path::{Path, PathBuf};

use systemprompt_web_admin_core::repositories::gateway_catalog::{
    create_route, delete_route, get_gateway_config, glob_match, reorder_routes, slugify_pattern,
    synthesize_route_id, update_gateway_settings, update_route,
};
use systemprompt_web_admin_core::types::{
    GatewayRouteView, RouteAliasView, RouteDeploymentView, UpdateGatewaySettingsRequest,
};
use systemprompt_web_shared::error::MarketplaceError;

fn profile(dir: &Path, body: &str) -> PathBuf {
    let path = dir.join("profile.yaml");
    std::fs::write(&path, body).expect("write profile");
    path
}

fn route(id: &str, pattern: &str, endpoint: &str) -> GatewayRouteView {
    GatewayRouteView {
        id: id.to_owned(),
        models: Some(pattern)
            .filter(|p| !p.is_empty())
            .map(str::to_owned)
            .into_iter()
            .collect(),
        deployments: vec![RouteDeploymentView {
            endpoint: endpoint.to_owned(),
            weight: None,
        }],
        ..Default::default()
    }
}

#[test]
fn glob_match_enforces_prefix_suffix_and_length() {
    assert!(glob_match("claude-*-latest", "claude-3-latest"));
    assert!(!glob_match("claude-*-latest", "claude-3-preview"));
    assert!(glob_match("*sonnet*", "claude-sonnet-5-5"));
    assert!(!glob_match("*sonnet*", "claude-opus-5-5"));
    assert!(glob_match("abc*def", "abcdef"));
    assert!(
        !glob_match("abc*bc", "abc"),
        "prefix and suffix must not overlap"
    );
    assert!(glob_match("*", ""));
    assert!(!glob_match("gpt-4", "gpt-4o"));
}

#[test]
fn glob_match_anchors_the_leading_segment() {
    assert!(!glob_match("claude*x*", "xclaudex"));
    assert!(glob_match("claude*x*", "claude-3x-end"));
}

#[test]
fn slugify_collapses_runs_and_trims_separators() {
    assert_eq!(slugify_pattern("--foo..bar--"), "foo-bar");
    assert_eq!(slugify_pattern("///"), "route");
    assert_eq!(slugify_pattern("**"), "starstar");
    assert_eq!(slugify_pattern("A_B C"), "a-b-c");
}

#[test]
fn synthesized_ids_differ_per_pattern() {
    assert_ne!(
        synthesize_route_id("claude-*", "anthropic/api"),
        synthesize_route_id("gpt-*", "anthropic/api")
    );
}

#[test]
fn create_route_bootstraps_a_missing_gateway_block() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = profile(dir.path(), "name: local\n");
    let index = create_route(&path, &route("", "claude-*", "anthropic/api"))?;
    assert_eq!(index, 0);

    let cfg = get_gateway_config(&path)?;
    assert_eq!(cfg.routes.len(), 1);
    assert_eq!(
        cfg.routes[0].id, "claude-star",
        "a new route's id is its glob's slug"
    );
    assert!(!cfg.enabled, "enabled defaults to false");
    assert_eq!(cfg.auth_scheme, "bearer");
    assert_eq!(cfg.inference_path_prefix, "/v1");
    assert_eq!(cfg.source_path, path.display().to_string());
    Ok(())
}

#[test]
fn create_route_rejects_no_models_or_a_bare_deployment() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = profile(dir.path(), "gateway:\n  routes: []\n");
    assert!(matches!(
        create_route(&path, &route("a", "", "anthropic/api")),
        Err(MarketplaceError::BadRequest(_))
    ));
    assert!(matches!(
        create_route(&path, &route("a", "claude-*", "anthropic")),
        Err(MarketplaceError::BadRequest(_))
    ));
    assert!(get_gateway_config(&path)?.routes.is_empty());
    Ok(())
}

#[test]
fn alias_and_weighted_deployments_survive_a_round_trip() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = profile(dir.path(), "gateway:\n  routes: []\n");
    let mut r = route("go-alias", "", "opencode-go/chat");
    let (requested, model) = ("go-deepseek".to_owned(), "deepseek-v4-pro".to_owned());
    r.alias = Some(RouteAliasView { requested, model });
    r.deployments[0].weight = Some(3);
    create_route(&path, &r)?;
    let back = &get_gateway_config(&path)?.routes[0];
    assert_eq!((&back.alias, &back.deployments), (&r.alias, &r.deployments));
    Ok(())
}

#[test]
fn routes_core_refuses_are_dropped_from_the_read_view() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = profile(
        dir.path(),
        "gateway:\n  routes:\n    - models: no-id\n    - id: alone\n      provider: anthropic\n    - id: ok\n      models: ok\n",
    );
    let cfg = get_gateway_config(&path)?;
    assert_eq!(cfg.routes.len(), 1);
    assert_eq!(cfg.routes[0].id, "ok");
    Ok(())
}

#[test]
fn update_route_replaces_in_place_and_reports_out_of_range() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = profile(dir.path(), "gateway:\n  routes: []\n");
    create_route(&path, &route("first", "claude-*", "anthropic/api"))?;

    assert!(update_route(
        &path,
        0,
        &route("first", "claude-*", "openai/chat")
    )?);
    let cfg = get_gateway_config(&path)?;
    assert_eq!(cfg.routes[0].deployments[0].endpoint, "openai/chat");

    assert!(
        !update_route(&path, 9, &route("first", "claude-*", "openai/chat"))?,
        "an out-of-range index is a miss, not an error"
    );
    Ok(())
}

#[test]
fn update_route_validates_before_touching_disk() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = profile(dir.path(), "gateway:\n  routes: []\n");
    create_route(&path, &route("first", "claude-*", "anthropic/api"))?;
    assert!(matches!(
        update_route(&path, 0, &route("first", "", "openai/chat")),
        Err(MarketplaceError::BadRequest(_))
    ));
    let kept = &get_gateway_config(&path)?.routes[0].deployments[0];
    assert_eq!(kept.endpoint, "anthropic/api");
    Ok(())
}

#[test]
fn delete_route_removes_by_index() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = profile(dir.path(), "gateway:\n  routes: []\n");
    create_route(&path, &route("a", "a-*", "anthropic/api"))?;
    create_route(&path, &route("b", "b-*", "openai/chat"))?;

    assert!(delete_route(&path, 0)?);
    let cfg = get_gateway_config(&path)?;
    assert_eq!(cfg.routes.len(), 1);
    assert_eq!(cfg.routes[0].id, "b");

    assert!(!delete_route(&path, 5)?);
    Ok(())
}

#[test]
fn reorder_rejects_wrong_length_and_non_permutations() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = profile(dir.path(), "gateway:\n  routes: []\n");
    create_route(&path, &route("a", "a-*", "anthropic/api"))?;
    create_route(&path, &route("b", "b-*", "openai/chat"))?;

    assert!(matches!(
        reorder_routes(&path, &[0]),
        Err(MarketplaceError::BadRequest(_))
    ));
    assert!(matches!(
        reorder_routes(&path, &[0, 0]),
        Err(MarketplaceError::BadRequest(_))
    ));
    assert!(matches!(
        reorder_routes(&path, &[0, 7]),
        Err(MarketplaceError::BadRequest(_))
    ));

    let ids: Vec<String> = get_gateway_config(&path)?
        .routes
        .into_iter()
        .map(|r| r.id)
        .collect();
    assert_eq!(ids, vec!["a".to_owned(), "b".to_owned()]);
    Ok(())
}

#[test]
fn update_gateway_settings_writes_each_supplied_field() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = profile(dir.path(), "name: local\n");
    let cfg = update_gateway_settings(
        &path,
        &UpdateGatewaySettingsRequest {
            enabled: Some(true),
            auth_scheme: Some("oauth".to_owned()),
            inference_path_prefix: Some("/api/v2".to_owned()),
        },
    )?;
    assert!(cfg.enabled);
    assert_eq!(cfg.auth_scheme, "oauth");
    assert_eq!(cfg.inference_path_prefix, "/api/v2");

    let unchanged = update_gateway_settings(
        &path,
        &UpdateGatewaySettingsRequest {
            enabled: None,
            auth_scheme: None,
            inference_path_prefix: None,
        },
    )?;
    assert!(unchanged.enabled);
    assert_eq!(unchanged.auth_scheme, "oauth");
    Ok(())
}

#[test]
fn inference_path_prefix_must_be_absolute() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = profile(dir.path(), "gateway:\n  enabled: true\n");
    assert!(matches!(
        update_gateway_settings(
            &path,
            &UpdateGatewaySettingsRequest {
                enabled: None,
                auth_scheme: None,
                inference_path_prefix: Some("v1".to_owned()),
            },
        ),
        Err(MarketplaceError::BadRequest(_))
    ));
    assert_eq!(get_gateway_config(&path)?.inference_path_prefix, "/v1");
    Ok(())
}

#[test]
fn a_non_mapping_profile_root_is_an_internal_error() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = profile(dir.path(), "- just\n- a list\n");
    assert!(matches!(
        create_route(&path, &route("a", "a-*", "anthropic/api")),
        Err(MarketplaceError::Internal(_))
    ));
    Ok(())
}

#[test]
fn name_and_description_round_trip_and_blank_ones_are_dropped() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = profile(dir.path(), "gateway:\n  routes: []\n");
    let mut r = route("named", "claude-*", "anthropic/api");
    r.name = Some("Claude (Anthropic)".to_owned());
    r.description = Some("  Every claude-* request.  ".to_owned());
    create_route(&path, &r)?;
    let cfg = get_gateway_config(&path)?;
    assert_eq!(cfg.routes[0].name.as_deref(), Some("Claude (Anthropic)"));
    assert_eq!(
        cfg.routes[0].description.as_deref(),
        Some("Every claude-* request.")
    );
    let text = std::fs::read_to_string(&path)?;
    let name_at = text.find("label: Claude").expect("label written");
    let pattern_at = text.find("models:").expect("models written");
    assert!(name_at < pattern_at, "name is written before the pattern");

    let mut cleared = cfg.routes[0].clone();
    cleared.name = Some("   ".to_owned());
    cleared.description = None;
    assert!(update_route(&path, 0, &cleared)?);
    let cfg = get_gateway_config(&path)?;
    assert_eq!(cfg.routes[0].name, None);
    assert_eq!(cfg.routes[0].description, None);
    assert!(!std::fs::read_to_string(&path)?.contains("label:"));
    Ok(())
}
