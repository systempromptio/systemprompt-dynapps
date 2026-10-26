//! Gateway route matching, validation and id stability, and the client-facing
//! filter the console applies against the resolved catalog.

use systemprompt_web_admin_core::repositories::gateway_catalog::{
    create_route, find_matching_route, find_matching_route_index, find_route_index_by_id,
    get_gateway_config, glob_match, reorder_routes, resolved_catalog, retain_client_facing,
    slugify_pattern, synthesize_route_id, update_route, validate_route,
};
use systemprompt_web_admin_core::types::{GatewayRouteView, RouteAliasView, RouteDeploymentView};
use systemprompt_web_shared::error::MarketplaceError;

use crate::support::committed_services;

const TWO_ROUTE_PROFILE: &str = r"gateway:
  enabled: true
  routes:
    - id: claude
      models: claude-*
    - id: everything
      models: '*'
";

fn route(id: &str, models: &[&str]) -> GatewayRouteView {
    GatewayRouteView {
        id: id.into(),
        models: models.iter().map(|m| (*m).to_owned()).collect(),
        ..Default::default()
    }
}

#[test]
fn glob_matches_exact_and_wildcard() {
    assert!(glob_match("*", "anything"));
    assert!(glob_match("claude-*", "claude-sonnet"));
    assert!(!glob_match("claude-*", "gpt-4"));
    assert!(glob_match("gpt-4", "gpt-4"));
    assert!(glob_match("*-latest", "gpt-4-latest"));
}

#[test]
fn first_match_wins_and_an_alias_matches_its_requested_id() {
    let mut aliased = route("go-alias", &[]);
    aliased.alias = Some(RouteAliasView {
        requested: "go-deepseek".into(),
        model: "deepseek-v4-pro".into(),
    });
    let routes = vec![
        route("claude-abc123", &["claude-*"]),
        aliased,
        route("star-def456", &["*"]),
    ];
    assert_eq!(find_matching_route_index(&routes, "claude-3"), Some(0));
    assert_eq!(find_matching_route_index(&routes, "go-deepseek"), Some(1));
    assert_eq!(find_matching_route_index(&routes, "gpt-4"), Some(2));
    assert_eq!(
        find_matching_route(&routes, "claude-3").map(|r| r.id.as_str()),
        Some("claude-abc123"),
    );
    assert_eq!(find_route_index_by_id(&routes, "star-def456"), Some(2));
}

#[test]
fn validate_rejects_a_route_with_no_models_and_a_bare_deployment() {
    assert!(validate_route(&route("empty", &[])).is_err());

    let mut bare = route("bare", &["*"]);
    bare.deployments = vec![RouteDeploymentView {
        endpoint: "anthropic".into(),
        weight: None,
    }];
    assert!(validate_route(&bare).is_err());

    bare.deployments[0].endpoint = "anthropic/api".into();
    assert!(validate_route(&bare).is_ok());
}

#[test]
fn slugify_replaces_star_and_non_alnum() {
    assert_eq!(slugify_pattern("*"), "star");
    assert_eq!(slugify_pattern("claude-*"), "claude-star");
    assert_eq!(slugify_pattern("*-latest"), "star-latest");
    assert_eq!(slugify_pattern("GPT-4"), "gpt-4");
    assert_eq!(slugify_pattern("foo.bar/baz"), "foo-bar-baz");
    assert_eq!(slugify_pattern(""), "route");
}

#[test]
fn synthesized_id_is_stable() {
    let a = synthesize_route_id("claude-*", "anthropic");
    let b = synthesize_route_id("claude-*", "anthropic");
    assert_eq!(a, b);
    assert!(a.starts_with("claude-star-"));
    let c = synthesize_route_id("claude-*", "openai");
    assert_ne!(a, c, "provider change must produce a different id");
}

#[test]
fn ids_stable_across_reorder() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("profile.yaml");
    std::fs::write(&path, TWO_ROUTE_PROFILE)?;
    let cfg = get_gateway_config(&path)?;
    let id0 = cfg.routes[0].id.clone();
    let id1 = cfg.routes[1].id.clone();

    reorder_routes(&path, &[1, 0])?;
    let cfg2 = get_gateway_config(&path)?;
    assert_eq!(cfg2.routes[0].id, id1);
    assert_eq!(cfg2.routes[1].id, id0);
    Ok(())
}

#[test]
fn create_route_rejects_duplicate_id() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("profile.yaml");
    std::fs::write(&path, "gateway:\n  enabled: true\n  routes: []\n")?;
    let route = route("fixed-id", &["claude-*"]);
    create_route(&path, &route)?;
    assert!(matches!(
        create_route(&path, &route),
        Err(MarketplaceError::BadRequest(_))
    ));
    Ok(())
}

const COMMENTED_PROFILE: &str = r"# Gateway routes - which models a policy route pins.
# Edit by hand or with: systemprompt admin config gateway ...
gateway:
  enabled: true
  routes:
  - id: claude
    models:
    - claude-*
    deployments:
    - anthropic/api
  - id: hand-written
    models:
    - gpt-*
";

#[test]
fn reading_the_config_never_writes_the_file() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("gateway.yaml");
    std::fs::write(&path, COMMENTED_PROFILE)?;

    let cfg = get_gateway_config(&path)?;
    assert_eq!(cfg.routes.len(), 2);
    assert_eq!(cfg.routes[1].id, "hand-written");

    assert_eq!(
        std::fs::read_to_string(&path)?,
        COMMENTED_PROFILE,
        "a read must leave the operator's file byte-identical"
    );
    Ok(())
}

#[test]
fn updating_one_route_changes_only_that_field() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("gateway.yaml");
    std::fs::write(&path, COMMENTED_PROFILE)?;

    let cfg = get_gateway_config(&path)?;
    let mut edited = cfg.routes[0].clone();
    edited.deployments[0].endpoint = "vertex/messages".into();
    assert!(update_route(&path, 0, &edited)?);

    let after = std::fs::read_to_string(&path)?;
    assert!(
        after.starts_with("# Gateway routes"),
        "the comment header must survive the write: {after}"
    );
    assert!(
        after.contains("id: hand-written"),
        "a hand-chosen id must survive: {after}"
    );

    let expected: Vec<String> = COMMENTED_PROFILE
        .lines()
        .map(|line| line.replace("- anthropic/api", "- vertex/messages"))
        .collect();
    let actual: Vec<String> = after.lines().map(str::to_owned).collect();
    assert_eq!(actual, expected, "only the edited field may differ");
    Ok(())
}

#[test]
fn a_route_reaching_no_advertised_model_is_never_client_facing() {
    let catalog = resolved_catalog(&committed_services()).expect("catalog resolves");
    let routes = vec![
        route("claude", &["claude-*"]),
        route("e2e-mock-route", &["e2e-mock-*"]),
        route("unknown", &["ghost-*"]),
    ];

    let kept = retain_client_facing(routes, &catalog);
    assert_eq!(
        kept.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(),
        vec!["claude"],
        "the e2e mock lists to no client, and a glob over no catalog model reaches nothing"
    );
}
