//! The `/admin/farm/*` pages. Paths are relative to the admin SSR mount,
//! which runs the user-context and non-admin gates before these handlers.

use std::sync::Arc;

use axum::extract::{Path, RawQuery};
use axum::response::Redirect;
use axum::routing::get;
use axum::{Extension, Router};
use sqlx::PgPool;

use systemprompt_web_admin_core::extension_boot::ExtensionBoot;

use crate::client::FarmApi;
use crate::config::FarmConfig;
use crate::sp::{PgSpUsage, SharedSp};
use crate::{agents, cost, credentials, dashboard, handlers, schedules, tenants};

/// Where the pages are served.
const NEW_ROOT: &str = "/admin/farm";

/// The routes with farm-api as `services/web/config/farm.yaml` names it (its
/// token read from the boot's secrets store) and systemprompt's usage read
/// through `pool` (the admin router's read pool).
pub fn routes<S: Clone + Send + Sync + 'static>(pool: Arc<PgPool>, boot: &ExtensionBoot) -> Router<S> {
    let config = FarmConfig::load(boot.services_root());
    let api = FarmApi::from_config(&config, Arc::clone(boot.secrets()));
    routes_with(api, Arc::new(PgSpUsage::new(pool)))
}

/// The routes against a given farm-api client and systemprompt usage source
/// (tests point them at fakes).
pub fn routes_with<S: Clone + Send + Sync + 'static>(api: FarmApi, sp: SharedSp) -> Router<S> {
    Router::new()
        .route("/farm", get(dashboard::dashboard_page))
        .route("/farm/tenants", get(tenants::tenants_page))
        .route("/farm/tasks", get(handlers::tasks_page))
        .route("/farm/tasks/{task_id}", get(handlers::task_page))
        .route("/farm/agents", get(agents::agents_page))
        .route("/farm/agents/{slug}", get(agents::agent_page))
        .route("/farm/schedules", get(schedules::schedules_page))
        .route("/farm/credentials", get(credentials::credentials_page))
        .route("/farm/cost", get(cost::cost_page))
        // The pages' old home. Bookmarks and links keep working.
        .route("/agent-farm", get(moved_root))
        .route("/agent-farm/{*rest}", get(moved))
        .layer(Extension(api))
        .layer(Extension(sp))
}

/// `/admin/agent-farm` → `/admin/farm`, keeping the query.
async fn moved_root(RawQuery(query): RawQuery) -> Redirect {
    Redirect::permanent(&target(None, query.as_deref()))
}

/// `/admin/agent-farm/<rest>` → `/admin/farm/<rest>`, keeping the query.
async fn moved(Path(rest): Path<String>, RawQuery(query): RawQuery) -> Redirect {
    Redirect::permanent(&target(Some(&rest), query.as_deref()))
}

/// The new location. The path is rebuilt from the decoded segments (empty and
/// dot segments dropped), each one encoded again, so what is sent back is
/// always under [`NEW_ROOT`]; the query is passed through untouched.
pub fn target(rest: Option<&str>, query: Option<&str>) -> String {
    let mut url = NEW_ROOT.to_owned();
    for segment in rest
        .unwrap_or_default()
        .split('/')
        .filter(|s| !matches!(*s, "" | "." | ".."))
    {
        url.push('/');
        url.push_str(&urlencoding::encode(segment));
    }
    if let Some(query) = query.filter(|q| !q.is_empty()) {
        url.push('?');
        url.push_str(query);
    }
    url
}
