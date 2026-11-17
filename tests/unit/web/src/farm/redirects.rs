//! The old `/admin/agent-farm/*` addresses redirect to `/admin/farm/*`,
//! keeping the path and the query. The handlers need neither farm-api nor
//! systemprompt, so this file builds the routes with an unconfigured client
//! and a source that has nothing.

use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use systemprompt_web_admin_agent_farm::client::FarmApi;
use systemprompt_web_admin_agent_farm::routes::routes_with;
use systemprompt_web_admin_agent_farm::sp::{SharedSp, SpCall, SpError, SpModelTotal, SpUsage};
use systemprompt_web_admin_agent_farm::types::SessionTotalRow;
use tower::ServiceExt;

struct NoSp;

#[async_trait::async_trait]
impl SpUsage for NoSp {
    async fn calls(&self, _: &str) -> Result<Vec<SpCall>, SpError> {
        Ok(Vec::new())
    }
    async fn totals_by_model(&self, _: &str) -> Result<Vec<SpModelTotal>, SpError> {
        Ok(Vec::new())
    }
    async fn contexts(&self, _: &str) -> Result<Vec<String>, SpError> {
        Ok(Vec::new())
    }
    async fn session_totals(&self, _: &[String]) -> Result<Vec<SessionTotalRow>, SpError> {
        Ok(Vec::new())
    }
}

async fn moved(uri: &str) -> Result<(StatusCode, String), Box<dyn std::error::Error>> {
    let sp: SharedSp = Arc::new(NoSp);
    let app: Router = routes_with(FarmApi::disabled(), sp);
    let response = app
        .oneshot(Request::builder().uri(uri).body(Body::empty())?)
        .await?;
    let location = response
        .headers()
        .get("location")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_owned();
    Ok((response.status(), location))
}

#[tokio::test]
async fn every_old_page_redirects_permanently_to_the_new_one()
-> Result<(), Box<dyn std::error::Error>> {
    for (old, new) in [
        ("/agent-farm", "/admin/farm"),
        ("/agent-farm/tenants", "/admin/farm/tenants"),
        ("/agent-farm/tasks", "/admin/farm/tasks"),
        ("/agent-farm/agents", "/admin/farm/agents"),
        (
            "/agent-farm/agents/code-reviewer",
            "/admin/farm/agents/code-reviewer",
        ),
        ("/agent-farm/schedules", "/admin/farm/schedules"),
        ("/agent-farm/credentials", "/admin/farm/credentials"),
        ("/agent-farm/cost", "/admin/farm/cost"),
        (
            "/agent-farm/tasks/01a17751-467a-7792-b87c-8b67e9d8fa36",
            "/admin/farm/tasks/01a17751-467a-7792-b87c-8b67e9d8fa36",
        ),
    ] {
        let (status, location) = moved(old).await?;
        assert_eq!(status, StatusCode::PERMANENT_REDIRECT, "{old}");
        assert_eq!(location, new, "{old}");
    }
    Ok(())
}

#[tokio::test]
async fn the_query_survives_the_redirect() -> Result<(), Box<dyn std::error::Error>> {
    for (old, new) in [
        ("/agent-farm?tenant=lab-b", "/admin/farm?tenant=lab-b"),
        (
            "/agent-farm/tasks?tenant=lab-b&cursor=159",
            "/admin/farm/tasks?tenant=lab-b&cursor=159",
        ),
        (
            "/agent-farm/cost?tenant=all&from=2026-10-20&to=2026-10-27",
            "/admin/farm/cost?tenant=all&from=2026-10-20&to=2026-10-27",
        ),
    ] {
        let (status, location) = moved(old).await?;
        assert_eq!(status, StatusCode::PERMANENT_REDIRECT, "{old}");
        assert_eq!(location, new, "{old}");
    }
    Ok(())
}

#[tokio::test]
async fn a_redirect_never_leaves_the_new_root() -> Result<(), Box<dyn std::error::Error>> {
    for old in [
        "/agent-farm//evil.example",
        "/agent-farm/..%2F..%2Fevil",
        "/agent-farm/%2F%2Fevil.example",
    ] {
        let (status, location) = moved(old).await?;
        assert_eq!(status, StatusCode::PERMANENT_REDIRECT, "{old}");
        assert!(location.starts_with("/admin/farm"), "{old} -> {location}");
        assert!(!location.starts_with("//"), "{old} -> {location}");
        assert!(!location.contains(".."), "{old} -> {location}");
    }
    Ok(())
}
