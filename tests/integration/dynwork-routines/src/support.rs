//! The routines router over a throwaway database, a real RS256 token
//! authority, and helpers that speak its HTTP.

use std::sync::{Arc, LazyLock};

use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use dynwork_routines::{RoutinesState, routines_router};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::PgPool;
use systemprompt::identifiers::{AccessTokenId, SessionId, UserId};
use systemprompt::models::auth::{
    AuthenticatedUser, JwtAudience, Permission, TokenAuthority, UserJwtRequest,
};
use systemprompt_security::jwt::DeploymentTokenAuthority;
use systemprompt_security::keys::{RsaSigningKey, SigningKeys};
use tower::ServiceExt;

static AUTHORITY: LazyLock<Arc<DeploymentTokenAuthority>> = LazyLock::new(|| {
    let key = RsaSigningKey::generate().expect("generate an RSA key");
    let keys = SigningKeys::new(key, &[7; 32]).expect("build signing keys");
    Arc::new(DeploymentTokenAuthority::new(keys, "https://routines.test"))
});

pub(crate) fn user(name: &str) -> UserId {
    UserId::try_new(name).expect("a test user id")
}

pub(crate) async fn seed_user(pool: &PgPool, name: &str) -> UserId {
    sqlx::query("INSERT INTO users (id, name, email) VALUES ($1, $1, $2)")
        .bind(name)
        .bind(format!("{name}@example.test"))
        .execute(pool)
        .await
        .expect("seed a user");
    user(name)
}

pub(crate) fn router(pool: &Arc<PgPool>) -> Router {
    let authority: Arc<DeploymentTokenAuthority> = Arc::clone(&AUTHORITY);
    let authority: Arc<dyn TokenAuthority> = authority;
    routines_router(RoutinesState {
        pool: Arc::clone(pool),
        authority,
    })
}

// The DynWork desktop's JWT for `name`, as core's bridge device link mints it.
pub(crate) fn desktop_token(name: &str) -> String {
    AUTHORITY
        .mint_user_jwt(&UserJwtRequest {
            user: AuthenticatedUser::new(
                user(name),
                name.to_owned(),
                format!("{name}@example.test"),
                vec![Permission::User],
            ),
            session_id: SessionId::generate(),
            jti: AccessTokenId::generate(),
            permissions: vec![Permission::User],
            audience: vec![JwtAudience::Bridge, JwtAudience::Mcp],
            ttl: chrono::Duration::minutes(30),
            client_id: None,
            purpose: None,
            plugin_id: None,
            interaction: None,
            browser_session: false,
        })
        .expect("mint a token")
}

pub(crate) struct Call<'a> {
    pub(crate) method: Method,
    pub(crate) uri: String,
    pub(crate) who: &'a str,
    pub(crate) if_match: Option<i64>,
    pub(crate) peer: Option<String>,
    pub(crate) body: Option<Value>,
}

impl<'a> Call<'a> {
    pub(crate) fn new(method: Method, uri: impl Into<String>, who: &'a str) -> Self {
        Self {
            method,
            uri: uri.into(),
            who,
            if_match: None,
            peer: None,
            body: None,
        }
    }

    pub(crate) const fn if_match(mut self, revision: i64) -> Self {
        self.if_match = Some(revision);
        self
    }

    pub(crate) fn peer(mut self, peer: &str) -> Self {
        self.peer = Some(peer.to_owned());
        self
    }

    pub(crate) fn body(mut self, body: Value) -> Self {
        self.body = Some(body);
        self
    }

    pub(crate) async fn send(self, router: &Router) -> (StatusCode, Value) {
        let mut request = Request::builder()
            .method(self.method)
            .uri(self.uri)
            .header(
                "authorization",
                format!("Bearer {}", desktop_token(self.who)),
            )
            .header("content-type", "application/json");
        if let Some(revision) = self.if_match {
            request = request.header("if-match", format!("\"{revision}\""));
        }
        if let Some(peer) = self.peer {
            request = request.header("x-dynwork-peer", peer);
        }
        let body = self
            .body
            .map_or_else(Body::empty, |body| Body::from(body.to_string()));
        let response = router
            .clone()
            .oneshot(request.body(body).expect("a request"))
            .await
            .expect("the router answers");
        let status = response.status();
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("a body")
            .to_bytes();
        let value = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes).expect("a JSON body")
        };
        (status, value)
    }
}

pub(crate) fn definition(name: &str) -> Value {
    json!({
        "name": name,
        "enabled": true,
        "instructions": { "kind": "prompt", "text": "Summarise my inbox" },
        "schedule": { "cron": "0 9 * * 1-5", "timezone": "Europe/Madrid" },
        "options": {},
    })
}

pub(crate) fn create_body(name: &str, device: &str) -> Value {
    json!({ "owner_device": device, "owner_name": "Laptop", "definition": definition(name) })
}

pub(crate) async fn create(router: &Router, who: &str, id: &str, device: &str) -> Value {
    let (status, body) = Call::new(Method::PUT, format!("/routines/{id}"), who)
        .body(create_body(id, device))
        .send(router)
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

// Every routine change after `since`, page by page, and the final cursor.
pub(crate) async fn drain_feed(
    router: &Router,
    who: &str,
    since: &str,
    limit: u32,
) -> (Vec<Value>, String) {
    let mut cursor = since.to_owned();
    let mut seen = Vec::new();
    loop {
        let (status, page) = Call::new(
            Method::GET,
            format!("/changes?since={cursor}&limit={limit}"),
            who,
        )
        .send(router)
        .await;
        assert_eq!(status, StatusCode::OK, "{page}");
        seen.extend(
            page["routines"]
                .as_array()
                .expect("routines")
                .iter()
                .cloned(),
        );
        cursor = page["cursor"].as_str().expect("a cursor").to_owned();
        if !page["more"].as_bool().expect("more") {
            return (seen, cursor);
        }
    }
}
