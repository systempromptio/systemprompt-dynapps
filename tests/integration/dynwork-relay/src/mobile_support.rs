//! What a browser and a phone do against the mobile sign-in routes, for the
//! `mobile_auth` suite: the instance's browser session cookie, the consent
//! page, the decision form and the code exchange.

use axum::body::Body;
use axum::http::{HeaderMap, Method, Request, StatusCode, header};
use dynwork_relay::MOBILE_BASE_PATH;
use dynwork_relay::ids::PeerId;
use dynwork_relay::mobile_auth::params::{challenge_of, percent_encode};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use systemprompt::identifiers::{AccessTokenId, SessionId};
use systemprompt::models::auth::{AuthenticatedUser, JwtAudience, Permission, UserJwtRequest};
use tower::ServiceExt;

use crate::support::{Relay, authority, user};

pub(crate) struct Answer {
    pub(crate) status: StatusCode,
    pub(crate) headers: HeaderMap,
    pub(crate) body: String,
}

impl Answer {
    pub(crate) fn location(&self) -> &str {
        self.headers
            .get(header::LOCATION)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
    }

    pub(crate) fn json(&self) -> Value {
        serde_json::from_str(&self.body).unwrap_or(Value::Null)
    }
}

// The instance's browser session for `name`, as core's sign-in mints it:
// first-party audiences, a browser session, the person's own permissions.
pub(crate) fn browser_session(name: &str, permissions: Vec<Permission>, browser: bool) -> String {
    authority()
        .mint_user_jwt(&UserJwtRequest {
            user: AuthenticatedUser::new(
                user(name),
                name.to_owned(),
                format!("{name}@example.test"),
                permissions.clone(),
            ),
            session_id: SessionId::generate(),
            jti: AccessTokenId::generate(),
            permissions,
            audience: JwtAudience::FIRST_PARTY.to_vec(),
            ttl: chrono::Duration::minutes(30),
            client_id: None,
            purpose: None,
            plugin_id: None,
            interaction: None,
            browser_session: browser,
        })
        .expect("mint a browser session")
}

pub(crate) fn cookie(name: &str) -> String {
    format!(
        "theme=dark; access_token={}",
        browser_session(name, vec![Permission::User], true)
    )
}

pub(crate) async fn send(
    relay: &Relay,
    method: Method,
    path: &str,
    headers: &[(&str, &str)],
    body: Option<(&str, String)>,
) -> Answer {
    let mut builder = Request::builder()
        .method(method)
        .uri(format!("{MOBILE_BASE_PATH}{path}"));
    for (name, value) in headers {
        builder = builder.header(*name, *value);
    }
    let request = match body {
        Some((kind, body)) => builder
            .header(header::CONTENT_TYPE, kind)
            .body(Body::from(body)),
        None => builder.body(Body::empty()),
    }
    .expect("a request");
    let response = relay
        .router
        .clone()
        .oneshot(request)
        .await
        .expect("a response");
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("a body")
        .to_bytes();
    Answer {
        status,
        headers,
        body: String::from_utf8_lossy(&bytes).into_owned(),
    }
}

// A phone's PKCE verifier and `state` for one sign-in.
pub(crate) struct Phone {
    pub(crate) verifier: String,
    pub(crate) state: String,
    pub(crate) peer: PeerId,
}

impl Phone {
    pub(crate) fn new(byte: u8) -> Self {
        Self {
            verifier: format!("verifier-{byte:03}-{}", "v".repeat(40)),
            state: format!("state-{byte:03}-{}", "s".repeat(32)),
            peer: PeerId::from_bytes([byte; 16]),
        }
    }

    pub(crate) fn authorize_path(&self, device_name: &str) -> String {
        format!(
            "/authorize?redirect_uri=dynwork%3A%2F%2Fauth%2Fcallback&code_challenge={}&code_challenge_method=S256&state={}&device_name={}",
            challenge_of(&self.verifier),
            self.state,
            percent_encode(device_name)
        )
    }
}

pub(crate) async fn authorize(relay: &Relay, owner: &str, phone: &Phone, name: &str) -> Answer {
    let cookie = cookie(owner);
    send(
        relay,
        Method::GET,
        &phone.authorize_path(name),
        &[("cookie", &cookie)],
        None,
    )
    .await
}

// The consent request token the page's form carries.
pub(crate) fn request_token(page: &str) -> String {
    let start = page
        .find(r#"name="request" value=""#)
        .expect("a request field")
        + r#"name="request" value=""#.len();
    page[start..].split('"').next().expect("a value").to_owned()
}

pub(crate) async fn decide(
    relay: &Relay,
    owner: Option<&str>,
    request: &str,
    decision: &str,
) -> Answer {
    let cookie = owner.map(cookie).unwrap_or_default();
    let headers: &[(&str, &str)] = if owner.is_some() {
        &[("cookie", &cookie)]
    } else {
        &[]
    };
    send(
        relay,
        Method::POST,
        "/authorize/decision",
        headers,
        Some((
            "application/x-www-form-urlencoded",
            format!("request={request}&decision={decision}"),
        )),
    )
    .await
}

pub(crate) fn query_value(location: &str, key: &str) -> Option<String> {
    location
        .split_once('?')?
        .1
        .split('&')
        .find_map(|pair| pair.strip_prefix(&format!("{key}=")))
        .map(str::to_owned)
}

// Consent, Allow, and the code from the callback.
pub(crate) async fn approved_code(relay: &Relay, owner: &str, phone: &Phone) -> String {
    let page = authorize(relay, owner, phone, "Pixel 9").await;
    assert_eq!(page.status, StatusCode::OK, "{}", page.body);
    let answer = decide(relay, Some(owner), &request_token(&page.body), "allow").await;
    assert_eq!(answer.status, StatusCode::SEE_OTHER, "{}", answer.body);
    query_value(answer.location(), "code").expect("a code")
}

pub(crate) async fn exchange(relay: &Relay, code: &str, verifier: &str, peer: PeerId) -> Answer {
    send(
        relay,
        Method::POST,
        "/token",
        &[],
        Some((
            "application/json",
            json!({ "code": code, "code_verifier": verifier, "peer_id": peer.to_hex() })
                .to_string(),
        )),
    )
    .await
}

// The whole path 1 for `owner`: the token answer.
pub(crate) async fn sign_in(relay: &Relay, owner: &str, phone: &Phone) -> Value {
    let code = approved_code(relay, owner, phone).await;
    let answer = exchange(relay, &code, &phone.verifier, phone.peer).await;
    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
    answer.json()
}

pub(crate) async fn with_bearer(relay: &Relay, method: Method, path: &str, token: &str) -> Answer {
    let bearer = format!("Bearer {token}");
    send(relay, method, path, &[("authorization", &bearer)], None).await
}
