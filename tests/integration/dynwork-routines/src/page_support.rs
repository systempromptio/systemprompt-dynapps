//! The admin pages of the extension as the console mounts them: its routes
//! over the pool, with the real templates under `storage/files/admin`, a
//! signed-in viewer that is or is not an admin, and a services view. Paths
//! are relative to `/admin`, the prefix the console nests the router at.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use axum::{Extension, Router};
use http_body_util::BodyExt;
use sqlx::PgPool;
use systemprompt::identifiers::{Email, UserId};
use systemprompt::manifest::services::ServicesView;
use systemprompt_web_admin_core::repositories::scope::Participation;
use systemprompt_web_admin_core::templates::AdminTemplateEngine;
use systemprompt_web_admin_core::types::{MarketplaceContext, UserContext};
use systemprompt_web_shared::RankTier;
use tower::ServiceExt;

// A services view with the repository's own MCP servers and plugins, the
// lists the definition form picks from.
pub(crate) fn repo_services() -> ServicesView {
    dynapps_test_common::services_view(&dynapps_test_common::repo_path(
        "services/config/config.yaml",
    ))
}

pub(crate) fn pages(
    pool: &Arc<PgPool>,
    viewer: &UserId,
    admin: bool,
    services: ServicesView,
) -> Router {
    dynwork_routines::page::routes()
        .with_state(Arc::clone(pool))
        .layer(Extension(engine()))
        .layer(Extension(viewer_context(viewer, admin)))
        .layer(Extension(marketplace(viewer)))
        .layer(Extension(dynapps_test_common::offline_extension_services(
            services,
        )))
}

fn engine() -> AdminTemplateEngine {
    let admin = dynapps_test_common::repo_path("storage/files/admin");
    AdminTemplateEngine::new(&admin, &admin).expect("the admin templates load")
}

// A console viewer: the gate lets them reach every console page, so only the
// handler's own `is_admin` check stands between them and the templates.
fn viewer_context(viewer: &UserId, admin: bool) -> UserContext {
    UserContext {
        user_id: viewer.clone(),
        username: viewer.as_str().to_owned(),
        email: Email::try_new(format!("{viewer}@example.test")).expect("an email"),
        roles: if admin {
            vec!["user".to_owned(), "admin".to_owned()]
        } else {
            vec!["user".to_owned()]
        },
        group_ids: Vec::new(),
        project_ids: Vec::new(),
        participation: Participation::default(),
        is_admin: admin,
        is_console: true,
        is_platform_admin: false,
        is_developer: false,
        email_verified: true,
        session_id: None,
    }
}

fn marketplace(viewer: &UserId) -> MarketplaceContext {
    MarketplaceContext {
        user_id: viewer.clone(),
        site_url: "https://governance.invalid".to_owned(),
        total_plugins: 0,
        total_skills: 0,
        agents_count: 0,
        mcp_count: 0,
        rank_level: 0,
        rank_name: String::new(),
        rank_tier: RankTier::default(),
        total_xp: 0,
        xp_progress_pct: 0.0,
        has_completed_onboarding: true,
        current_streak: 0,
        longest_streak: 0,
        next_rank_name: String::new(),
        xp_to_next_rank: 0,
    }
}

pub(crate) struct Html {
    pub(crate) status: StatusCode,
    pub(crate) location: Option<String>,
    pub(crate) text: String,
}

impl Html {
    // Whether the page holds `needle` once Handlebars' escaping is undone.
    pub(crate) fn has(&self, needle: &str) -> bool {
        self.text
            .replace("&#x3D;", "=")
            .replace("&#x27;", "'")
            .replace("&#x60;", "`")
            .replace("&quot;", "\"")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&")
            .contains(needle)
    }

    // A form post's redirect: 303 to `to` (before the flash), and the flash
    // tone it carries.
    pub(crate) fn redirected(&self, to: &str, tone: &str) -> bool {
        self.status == StatusCode::SEE_OTHER
            && self.location.as_deref().is_some_and(|location| {
                location.starts_with(&format!("{to}?flash="))
                    && location.ends_with(&format!("flash_tone={tone}"))
            })
    }
}

pub(crate) async fn get_page(router: &Router, uri: &str) -> Html {
    send(
        router,
        Request::builder()
            .uri(uri)
            .body(Body::empty())
            .expect("a request"),
    )
    .await
}

pub(crate) async fn post_form(router: &Router, uri: &str, fields: &[(&str, &str)]) -> Html {
    let body = url::form_urlencoded::Serializer::new(String::new())
        .extend_pairs(fields)
        .finish();
    send(
        router,
        Request::builder()
            .method("POST")
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(body))
            .expect("a request"),
    )
    .await
}

async fn send(router: &Router, request: Request<Body>) -> Html {
    let response = router
        .clone()
        .oneshot(request)
        .await
        .expect("the router answers");
    let status = response.status();
    let location = response
        .headers()
        .get(header::LOCATION)
        .map(|value| value.to_str().expect("an ASCII location").to_owned());
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("a body")
        .to_bytes();
    Html {
        status,
        location,
        text: String::from_utf8_lossy(&bytes).into_owned(),
    }
}
