//! The device link that also carries the person's UI session.
//!
//! A client adds a `person_ui` client's PKCE request to the link; the page
//! checks it as core's `check_ui_request` does and stores it on the exchange
//! code, and core's `session-pat` redemption then mints the UI session's
//! authorization code. Every way the request can be wrong is refused with a
//! page and issues nothing. Only a browser sign-in may approve it, and only
//! one under a day old: an older one is sent through Entra first with a
//! `redirect` core's `sanitize_return_to` keeps. `replaces` names a device
//! key by its `key_prefix` (the PAT before the dot, all a device knows of it)
//! and retires the approving person's own device key and nothing else.
//!
//! `session-pat` itself is core's HTTP handler and needs a booted runtime; its
//! two steps — consuming the code and minting the bound authorization code —
//! are driven here through core's own repository.

use std::sync::Arc;

use axum::http::StatusCode;
use base64::Engine as _;
use sha2::{Digest, Sha256};
use sqlx::Row;
use systemprompt::database::Database;
use systemprompt::identifiers::{AccessTokenId, ApiKeyId, ClientId, SessionId, UserId};
use systemprompt::models::auth::{AuthenticatedUser, JwtAudience, Permission, UserJwtRequest};
use systemprompt::oauth::repository::{
    ClientRepository, DeclaredClientWrite, MintAuthCodeParams, OAuthRepository,
};
use systemprompt::oauth::services::hash_exchange_code;

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal};

const PAGE: &str = "/bridge-auth/device-link";
const APPROVE: &str = "/bridge-auth/device-link/approve";
const FORM: &str = "application/x-www-form-urlencoded";

const CLIENT: &str = "contract-person-ui";
const PLAIN_CLIENT: &str = "contract-not-person-ui";
const CALLBACK: &str = "http://127.0.0.1:9876/callback";
const UI_REDIRECT: &str = "http://127.0.0.1:9876/oauth/callback";
const VERIFIER: &str = "contract-device-link-verifier-0123456789abcdefghijklmnop";
const DAY: i64 = 86_400;

// A malformed UI request: what it is, the query fields it overrides (`None`
// drops one), and the word the refusal must name.
type BadUiRequest = (
    &'static str,
    &'static [(&'static str, Option<&'static str>)],
    &'static str,
);

fn challenge() -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(Sha256::digest(VERIFIER.as_bytes()))
}

// The five PKCE fields as a client sends them, with one field replaced or
// dropped per refusal case.
fn ui_fields(overrides: &[(&str, Option<&str>)]) -> Vec<(String, String)> {
    let challenge = challenge();
    let base = [
        ("client_id", CLIENT),
        ("redirect_uri", UI_REDIRECT),
        ("scope", "user"),
        ("code_challenge", challenge.as_str()),
        ("code_challenge_method", "S256"),
    ];
    base.iter()
        .filter_map(|&(name, value)| {
            let value = overrides
                .iter()
                .find(|&&(field, _)| field == name)
                .map_or(Some(value), |&(_, replaced)| replaced)?;
            Some((name.to_owned(), value.to_owned()))
        })
        .collect()
}

fn encode(pairs: &[(String, String)]) -> String {
    let mut out = url::form_urlencoded::Serializer::new(String::new());
    for (name, value) in pairs {
        out.append_pair(name, value);
    }
    out.finish()
}

fn with_callback(mut pairs: Vec<(String, String)>) -> Vec<(String, String)> {
    pairs.insert(0, ("redirect".to_owned(), CALLBACK.to_owned()));
    pairs
}

fn post<'a>(path: &'a str, body: &'a str) -> Call<'a> {
    Call {
        method: "post",
        path,
        principal: Principal::Anonymous,
        content_type: Some(FORM),
        body: Some(body),
    }
}

// The people a case acts as: the admin principal approves; the non-admin
// owns the key that is someone else's.
struct People {
    admin: UserId,
    other: UserId,
}

// The `access_token` cookie core's web sign-in sets for the admin principal,
// stating `auth_time` as the moment the person signed in.
fn browser_cookie(people: &People, auth_time: i64) -> String {
    let user = AuthenticatedUser::new_with_roles(
        people.admin.clone(),
        "contract-admin".to_owned(),
        "contract-admin@contract.test".to_owned(),
        vec![Permission::User],
        vec!["admin".to_owned(), "user".to_owned()],
    );
    let token = globals::boot()
        .token_authority()
        .mint_user_jwt(&UserJwtRequest {
            user,
            session_id: SessionId::new(uuid::Uuid::new_v4().to_string()),
            jti: AccessTokenId::generate(),
            permissions: vec![Permission::User],
            audience: JwtAudience::standard(),
            ttl: chrono::Duration::hours(1),
            client_id: None,
            purpose: None,
            plugin_id: None,
            interaction: None,
            browser_session: true,
            ui_session: false,
            auth_time: Some(auth_time),
        })
        .expect("mint a browser session token");
    format!("access_token={token}")
}

fn signed_in_ago(seconds: i64) -> i64 {
    chrono::Utc::now().timestamp() - seconds
}

async fn declare_clients(pool: &Arc<sqlx::PgPool>, owner: &UserId) {
    let redirects = ["http://127.0.0.1/oauth/callback".to_owned()];
    let grants = ["authorization_code".to_owned(), "refresh_token".to_owned()];
    let scopes = ["user".to_owned()];
    let mut tx = pool.begin().await.expect("begin");
    for (id, person_ui) in [(CLIENT, true), (PLAIN_CLIENT, false)] {
        ClientRepository::upsert_declared_on(
            &mut tx,
            &DeclaredClientWrite {
                client_id: &ClientId::new(id),
                client_name: id,
                owner_user_id: owner,
                is_active: true,
                secret_key: None,
                secret_hash: None,
                redirect_uris: &redirects,
                grant_types: &grants,
                scopes: &scopes,
                person_ui,
                source: "services/identity/contract.yaml",
            },
        )
        .await
        .expect("declare the client");
    }
    tx.commit().await.expect("commit");
}

// A throwaway database with both clients declared, under `TempDb::create`'s
// CI guard: it gives up only where no database server is configured.
async fn setup_or_skip() -> Option<(TempDb, App, People)> {
    if !globals::init() {
        return None;
    }
    let db = TempDb::create().await?;
    let credentials = principal::provision(&db.pool).await;
    let people = People {
        admin: credentials.admin_user_id.clone(),
        other: credentials.non_admin_user_id.clone(),
    };
    declare_clients(&db.pool, &people.admin).await;
    let app = App::new(&db.pool, credentials);
    Some((db, app, people))
}

async fn codes_for(pool: &sqlx::PgPool, user: &UserId) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM bridge_exchange_codes WHERE user_id = $1")
        .bind(user.as_str())
        .fetch_one(pool)
        .await
        .expect("count exchange codes")
}

fn code_of(location: &str) -> String {
    location
        .split("code=")
        .nth(1)
        .unwrap_or_default()
        .to_owned()
}

#[tokio::test]
async fn a_pkce_approval_carries_the_ui_session_to_session_pat() {
    let Some((db, app, people)) = setup_or_skip().await else {
        return;
    };
    let signed_in = signed_in_ago(60);
    let cookie = browser_cookie(&people, signed_in);
    let query = encode(&with_callback(ui_fields(&[])));

    let page = format!("{PAGE}?{query}");
    let (status, body) = app
        .call_with_headers(
            Call::get(&page, Principal::Anonymous),
            &[("cookie", cookie.as_str())],
        )
        .await;
    assert_eq!(status, StatusCode::OK, "the page renders: {body}");
    for field in ["client_id", "redirect_uri", "scope", "code_challenge"] {
        assert!(
            body.contains(&format!("name=\"{field}\"")),
            "the Allow form carries {field} through approval"
        );
    }
    assert!(body.contains(CLIENT), "the page names the app it signs in");

    let (status, location) = app
        .redirect_with_headers(post(APPROVE, &query), &[("cookie", cookie.as_str())])
        .await;
    assert!(status.is_redirection(), "approve redirects, got {status}");
    assert!(
        location.starts_with(&format!("{CALLBACK}?code=")),
        "the code goes to the loopback callback, got {location}"
    );
    let code = code_of(&location);

    let row = sqlx::query(
        "SELECT user_id, ui_client_id, ui_redirect_uri, ui_scope, ui_code_challenge,
                ui_authenticated_at
         FROM bridge_exchange_codes WHERE code_hash = $1",
    )
    .bind(hash_exchange_code(&code))
    .fetch_one(&*db.pool)
    .await
    .expect("the code is stored by its hash");
    let text = |column: &str| row.get::<Option<String>, _>(column);
    assert_eq!(text("user_id").as_deref(), Some(people.admin.as_str()));
    assert_eq!(text("ui_client_id").as_deref(), Some(CLIENT));
    assert_eq!(text("ui_redirect_uri").as_deref(), Some(UI_REDIRECT));
    assert_eq!(text("ui_scope").as_deref(), Some("user"));
    assert_eq!(text("ui_code_challenge"), Some(challenge()));
    assert_eq!(
        row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("ui_authenticated_at")
            .map(|at| at.timestamp()),
        Some(signed_in),
        "the UI session starts from the cookie's sign-in, never the click"
    );

    // What core's `session-pat` does with the code: consume it, mint the
    // device key, and bind the UI session's authorization code to that key.
    let database = Arc::new(Database::from_pools(
        Arc::clone(&db.pool),
        Some(Arc::clone(&db.pool)),
    ));
    let oauth = OAuthRepository::new(&database, Arc::from(globals::OAUTH_PEPPER));
    let link = oauth
        .consume_device_link_code(&hash_exchange_code(&code))
        .await
        .expect("consume the code")
        .expect("the code is live");
    let grant = link.ui_grant.expect("the code carries the UI session");
    let (device_key, _) = insert_key(&db.pool, &people.admin, "device").await;
    let ui_code = oauth
        .mint_device_authorization_code(
            MintAuthCodeParams {
                client_id: &grant.client_id,
                user_id: &link.user_id,
                redirect_uri: &grant.redirect_uri,
                scope: Some(&grant.scope),
                code_challenge: &grant.code_challenge,
                code_challenge_method: "S256",
                resource: None,
                authenticated_at: grant.authenticated_at,
            },
            &ApiKeyId::new(device_key),
        )
        .await
        .expect("session-pat mints the ui_code");
    assert!(!ui_code.as_str().is_empty(), "a ui_code is returned");

    db.cleanup().await;
}

#[tokio::test]
async fn a_device_link_without_pkce_stays_a_plain_code() {
    let Some((db, app, people)) = setup_or_skip().await else {
        return;
    };
    let cookie = browser_cookie(&people, signed_in_ago(3 * DAY));
    let body = encode(&with_callback(Vec::new()));

    let (status, location) = app
        .redirect_with_headers(post(APPROVE, &body), &[("cookie", cookie.as_str())])
        .await;
    assert!(status.is_redirection(), "approve redirects, got {status}");
    let code = code_of(&location);
    assert!(
        !code.is_empty(),
        "a plain code is issued, sign-in age aside"
    );

    let database = Arc::new(Database::from_pools(
        Arc::clone(&db.pool),
        Some(Arc::clone(&db.pool)),
    ));
    let link = OAuthRepository::new(&database, Arc::from(globals::OAUTH_PEPPER))
        .consume_device_link_code(&hash_exchange_code(&code))
        .await
        .expect("consume the code")
        .expect("the code is live");
    assert!(
        link.ui_grant.is_none(),
        "no UI session rides on a plain link"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn each_bad_ui_request_is_refused_and_issues_nothing() {
    let Some((db, app, people)) = setup_or_skip().await else {
        return;
    };
    let cookie = browser_cookie(&people, signed_in_ago(60));
    let cases: [BadUiRequest; 8] = [
        (
            "an undeclared client",
            &[("client_id", Some("nobody"))],
            "person_ui",
        ),
        (
            "a declared client that is not person_ui",
            &[("client_id", Some(PLAIN_CLIENT))],
            "person_ui",
        ),
        (
            "an unregistered redirect",
            &[("redirect_uri", Some("http://127.0.0.1:9876/elsewhere"))],
            "redirect_uri",
        ),
        (
            "a non-loopback redirect",
            &[("redirect_uri", Some("https://example.com/oauth/callback"))],
            "redirect_uri",
        ),
        (
            "the plain method",
            &[("code_challenge_method", Some("plain"))],
            "S256",
        ),
        (
            "an empty challenge",
            &[("code_challenge", Some(""))],
            "S256",
        ),
        (
            "a scope the client does not declare",
            &[("scope", Some("user admin"))],
            "scope",
        ),
        (
            "only some of the five fields",
            &[("code_challenge", None)],
            "together",
        ),
    ];

    for (label, overrides, names) in cases {
        let query = encode(&with_callback(ui_fields(overrides)));
        let before = codes_for(&db.pool, &people.admin).await;

        let page = format!("{PAGE}?{query}");
        let (status, body) = app
            .call_with_headers(
                Call::get(&page, Principal::Anonymous),
                &[("cookie", cookie.as_str())],
            )
            .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "page, {label}: {body}");
        assert!(
            body.contains(names),
            "page, {label}: the refusal says why: {body}"
        );

        let (status, body) = app
            .call_with_headers(post(APPROVE, &query), &[("cookie", cookie.as_str())])
            .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "approve, {label}: {body}");
        assert!(
            !body.contains("code="),
            "approve, {label}: no code is handed out"
        );
        assert_eq!(
            codes_for(&db.pool, &people.admin).await,
            before,
            "approve, {label}: nothing is issued, not even a plain code"
        );
    }

    db.cleanup().await;
}

#[tokio::test]
async fn only_a_browser_sign_in_approves_a_ui_session() {
    let Some((db, app, people)) = setup_or_skip().await else {
        return;
    };
    let query = encode(&with_callback(ui_fields(&[])));
    let before = codes_for(&db.pool, &people.admin).await;

    // A bearer token is what an agent holding the person's credential has;
    // it must not turn itself into the person's UI session.
    let page = format!("{PAGE}?{query}");
    let (status, body) = app.call(Call::get(&page, Principal::Admin)).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "page with a bearer: {body}");
    let approve = Call {
        principal: Principal::Admin,
        ..post(APPROVE, &query)
    };
    let (status, body) = app.call(approve).await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "approve with a bearer: {body}"
    );
    assert_eq!(
        codes_for(&db.pool, &people.admin).await,
        before,
        "a bearer approval issues nothing"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn a_sign_in_older_than_a_day_goes_through_entra_first() {
    let Some((db, app, people)) = setup_or_skip().await else {
        return;
    };
    let mut pairs = with_callback(ui_fields(&[]));
    pairs.push(("replaces".to_owned(), "sp-live-0123456789ab".to_owned()));
    let query = encode(&pairs);
    let before = codes_for(&db.pool, &people.admin).await;

    for (label, auth_time) in [
        ("a day and a minute", signed_in_ago(DAY + 60)),
        ("no sign-in time", 0),
    ] {
        let cookie = browser_cookie(&people, auth_time);
        let page = format!("{PAGE}?{query}");
        for (step, call) in [
            ("page", Call::get(&page, Principal::Anonymous)),
            ("approve", post(APPROVE, &query)),
        ] {
            let (status, location) = app
                .redirect_with_headers(call, &[("cookie", cookie.as_str())])
                .await;
            assert_eq!(status, StatusCode::SEE_OTHER, "{step}, {label}");
            let (path, start) = location.split_once('?').unwrap_or_default();
            assert_eq!(
                path, "/admin/auth/entra/start",
                "{step}, {label}: {location}"
            );
            let params: Vec<(String, String)> = url::form_urlencoded::parse(start.as_bytes())
                .into_owned()
                .collect();
            let value = |name: &str| {
                params
                    .iter()
                    .find(|(key, _)| key == name)
                    .map(|(_, v)| v.clone())
                    .unwrap_or_default()
            };
            assert_eq!(
                value("max_age"),
                "86399",
                "{step}, {label}: Entra's longest max_age"
            );
            let back = value("redirect");
            assert_eq!(
                systemprompt::sso::sanitize_return_to(&back),
                Some(back.as_str()),
                "{step}, {label}: core keeps the return path {back}"
            );
            let (back_path, back_query) = back.split_once('?').unwrap_or_default();
            assert_eq!(back_path, PAGE, "{step}, {label}: back to this page");
            let returned: Vec<(String, String)> =
                url::form_urlencoded::parse(back_query.as_bytes())
                    .into_owned()
                    .collect();
            assert_eq!(
                returned, pairs,
                "{step}, {label}: the same request comes back"
            );
        }
    }
    assert_eq!(
        codes_for(&db.pool, &people.admin).await,
        before,
        "nothing is issued before the fresh sign-in"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn replaces_retires_only_the_same_persons_device_key() {
    let Some((db, app, people)) = setup_or_skip().await else {
        return;
    };
    let (_, own_device) = insert_key(&db.pool, &people.admin, "device").await;
    let (_, own_user_key) = insert_key(&db.pool, &people.admin, "user").await;
    let (_, foreign_device) = insert_key(&db.pool, &people.other, "device").await;

    for (replaces, retired) in [
        (own_user_key.as_str(), false),
        (foreign_device.as_str(), false),
        ("sp-live-000000000000", false),
        (own_device.as_str(), true),
    ] {
        let mut pairs = with_callback(Vec::new());
        pairs.push(("replaces".to_owned(), replaces.to_owned()));
        let body = encode(&pairs);
        let approve = Call {
            principal: Principal::Admin,
            ..post(APPROVE, &body)
        };
        let (status, location) = app.redirect_of(approve).await;
        assert!(
            status.is_redirection() && location.contains("code="),
            "replaces={replaces}: a prefix that matches nothing is no error, got {status} {location}"
        );
        assert_eq!(
            is_revoked(&db.pool, replaces).await,
            retired,
            "replaces={replaces}"
        );
    }
    assert!(
        !is_revoked(&db.pool, &own_user_key).await,
        "a user key is never retired"
    );
    assert!(
        !is_revoked(&db.pool, &foreign_device).await,
        "another person's key stays"
    );

    db.cleanup().await;
}

// A key as core stores it: its id, and the `key_prefix` a device sends back
// in `replaces` (`sp-live-` and 12 hex, the PAT before the dot).
async fn insert_key(pool: &sqlx::PgPool, user: &UserId, kind: &str) -> (String, String) {
    let id = format!("ak_{}", uuid::Uuid::new_v4().simple());
    let key_prefix = format!(
        "sp-live-{}",
        &uuid::Uuid::new_v4().simple().to_string()[..12]
    );
    sqlx::query(
        "INSERT INTO user_api_keys (id, user_id, name, key_prefix, key_hash, kind)
         VALUES ($1, $2, 'contract device', $3, 'not-a-hash', $4)",
    )
    .bind(&id)
    .bind(user.as_str())
    .bind(&key_prefix)
    .bind(kind)
    .execute(pool)
    .await
    .expect("insert an api key");
    (id, key_prefix)
}

async fn is_revoked(pool: &sqlx::PgPool, key_prefix: &str) -> bool {
    sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM user_api_keys WHERE key_prefix = $1 AND revoked_at IS NOT NULL)",
    )
    .bind(key_prefix)
    .fetch_one(pool)
    .await
    .expect("read the key")
}
