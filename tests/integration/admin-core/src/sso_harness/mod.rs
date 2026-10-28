//! The sign-in router as the server mounts it, plus the request and audit
//! helpers the SSO flow suites share.
//!
//! [`SsoApp::build`] installs the Entra OIDC config the test supplies into
//! the identity crate's process-wide slot, then lets core's driver discover
//! providers — the order `extensions/web/src/router/sso.rs` uses. The slot
//! is a first-write-wins `OnceLock`, so every test builds exactly one app and
//! relies on nextest's process-per-test isolation.

pub(crate) mod oidc;
pub(crate) mod profile;

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use axum::{Extension, Router};
use sqlx::PgPool;
use systemprompt::database::Database;
use systemprompt::oauth::SessionCreationService;
use systemprompt::users::{SessionRepository, UserRepository, UserService};
use systemprompt_web_admin_identity::sso::entra_oidc;
use systemprompt_web_admin_identity::{SsoDeps, routes};
use tower::ServiceExt as _;

pub(crate) use profile::{EXTERNAL_URL, OIDC_TENANT};

pub(crate) const OIDC_START: &str = "/admin/auth/entra/start";
pub(crate) const OIDC_CALLBACK: &str = "/admin/auth/entra/callback";
pub(crate) const BINDER: &str = "sp_sso_bind";

pub(crate) struct SsoApp {
    router: Router,
}

// One response, reduced to what an SSO assertion reads.
#[derive(Debug, Clone)]
pub(crate) struct Reply {
    pub status: StatusCode,
    pub location: String,
    pub set_cookies: Vec<String>,
}

impl Reply {
    // `name=value` of a cookie the response set, empty values excluded.
    pub(crate) fn cookie(&self, name: &str) -> Option<String> {
        self.set_cookies.iter().find_map(|c| {
            let pair = c.split(';').next()?.trim();
            let value = pair.strip_prefix(&format!("{name}="))?;
            (!value.is_empty()).then(|| pair.to_owned())
        })
    }

    // The `?sso=<reason>` a refusal redirects with.
    pub(crate) fn reason(&self) -> Option<&str> {
        let (_, query) = self.location.split_once("sso=")?;
        Some(query.split('&').next().unwrap_or(query))
    }

    pub(crate) fn assert_refused(&self, reason: &str) {
        assert!(
            self.status.is_redirection(),
            "expected a redirect, got {self:?}"
        );
        assert!(
            self.location.starts_with("/admin/login"),
            "a refusal lands on the login page: {self:?}"
        );
        assert_eq!(self.reason(), Some(reason), "refusal reason: {self:?}");
        assert!(
            self.cookie("access_token").is_none(),
            "a refusal mints no session"
        );
    }

    pub(crate) fn assert_signed_in(&self, return_to: &str) {
        assert_eq!(
            self.status,
            StatusCode::SEE_OTHER,
            "success is a 303: {self:?}"
        );
        assert_eq!(
            self.location, return_to,
            "success returns to the start target"
        );
        assert!(
            self.cookie("access_token").is_some(),
            "success sets access_token: {self:?}"
        );
        let session = self
            .set_cookies
            .iter()
            .find(|c| c.starts_with("access_token="))
            .expect("session cookie");
        assert!(
            session.contains("SameSite=Lax") && session.contains("HttpOnly"),
            "the session cookie survives the IdP's cross-site redirect: {session}"
        );
    }
}

fn session_service(
    pool: &Arc<PgPool>,
    config: &Arc<systemprompt::manifest::Config>,
) -> Arc<SessionCreationService> {
    let db = Arc::new(Database::from_pools(
        Arc::clone(pool),
        Some(Arc::clone(pool)),
    ));
    let users = UserService::new(Arc::new(UserRepository::new(&db)));
    Arc::new(SessionCreationService::new(
        Arc::new(SessionRepository::new(&db)),
        Arc::new(users),
        Arc::new(systemprompt::security::jwt::DeploymentTokenAuthority::new(
            dynapps_test_common::ephemeral_signing_keys(),
            config.jwt_issuer.clone(),
        )),
        Arc::clone(config),
    ))
}

impl SsoApp {
    pub(crate) fn build(pool: &Arc<PgPool>, oidc: Option<entra_oidc::EntraOidcConfig>) -> Self {
        let boot = profile::boot();
        let config = Arc::clone(boot.config().expect("fixture profile config"));
        let config_file = profile::services_web_config_dir().join("entra-oidc.yaml");
        match oidc {
            Some(cfg) => std::fs::write(
                &config_file,
                serde_yaml::to_string(&cfg).expect("serialise the Entra OIDC fixture"),
            )
            .expect("write the Entra OIDC fixture"),
            None => {
                if let Err(error) = std::fs::remove_file(&config_file)
                    && error.kind() != std::io::ErrorKind::NotFound
                {
                    panic!("remove a stale Entra OIDC fixture: {error}");
                }
            },
        }
        let database: systemprompt::database::DbPool = Arc::new(Database::from_pools(
            Arc::clone(pool),
            Some(Arc::clone(pool)),
        ));
        let sessions = session_service(pool, &config);
        let hook = systemprompt::sso::SsoHookContext {
            db: Arc::clone(&database),
            config,
            services: dynapps_test_common::empty_services_view(),
            services_root: systemprompt::manifest::services::root::ActiveServicesRoot::baked(
                boot.profile(),
            ),
            secrets: Arc::<systemprompt::config::SecretsService>::clone(boot.secrets()),
        };
        let oauth = systemprompt::oauth::repository::OAuthRepository::new(
            &database,
            Arc::from(boot.secrets().oauth_at_rest_pepper()),
        );
        let deps = SsoDeps {
            driver: Some(systemprompt::sso::SsoDriver::discover(
                &hook,
                oauth,
                Arc::clone(&sessions),
            )
            .expect("SSO provider discovery")),
            write_pool: Arc::clone(pool),
            session_service: sessions,
        };
        let router = Router::new()
            .nest(
                "/admin",
                routes::public::sign_in_routes(boot.profile()).with_state(Arc::clone(pool)),
            )
            .layer(Extension(deps));
        Self { router }
    }

    pub(crate) async fn get(&self, uri: &str, cookie: Option<&str>) -> Reply {
        self.send(Request::get(uri), Body::empty(), cookie).await
    }

    async fn send(
        &self,
        mut builder: axum::http::request::Builder,
        body: Body,
        cookie: Option<&str>,
    ) -> Reply {
        if let Some(cookie) = cookie {
            builder = builder.header(header::COOKIE, cookie);
        }
        let response = self
            .router
            .clone()
            .oneshot(builder.body(body).expect("request"))
            .await
            .expect("router is infallible");
        let headers = response.headers();
        Reply {
            status: response.status(),
            location: headers
                .get(header::LOCATION)
                .and_then(|v| v.to_str().ok())
                .unwrap_or_default()
                .to_owned(),
            set_cookies: headers
                .get_all(header::SET_COOKIE)
                .iter()
                .filter_map(|v| v.to_str().ok().map(str::to_owned))
                .collect(),
        }
    }
}

// A form value, percent-encoded for an `x-www-form-urlencoded` body.
pub(crate) fn form_encode(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

// `(outcome, reason)` of every `sso_login_events` row, oldest first.
pub(crate) async fn login_events(pool: &PgPool) -> Vec<(String, Option<String>)> {
    sqlx::query_as("SELECT outcome, reason FROM sso_login_events ORDER BY id")
        .fetch_all(pool)
        .await
        .expect("read sso_login_events")
}

pub(crate) async fn federated_user(pool: &PgPool, external_sub: &str) -> Option<String> {
    sqlx::query_scalar("SELECT user_id FROM federated_identities WHERE external_sub = $1")
        .bind(external_sub)
        .fetch_optional(pool)
        .await
        .expect("read federated_identities")
}

pub(crate) async fn directory_groups(pool: &PgPool, user_id: &str) -> Vec<String> {
    sqlx::query_scalar(
        "SELECT group_id FROM group_members WHERE user_id = $1 AND source = 'adfs' ORDER BY group_id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .expect("read group_members")
}

// A dashboard group that directory group `ad_group` maps onto.
pub(crate) async fn map_directory_group(pool: &PgPool, group_id: &str, ad_group: &str) {
    sqlx::query("INSERT INTO groups (id, name) VALUES ($1, $1) ON CONFLICT (id) DO NOTHING")
        .bind(group_id)
        .execute(pool)
        .await
        .expect("insert group");
    sqlx::query("INSERT INTO group_ad_mappings (ad_group, group_id) VALUES ($1, $2)")
        .bind(ad_group)
        .bind(group_id)
        .execute(pool)
        .await
        .expect("insert group_ad_mapping");
}
