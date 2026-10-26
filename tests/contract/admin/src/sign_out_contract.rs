//! Sign-out ends the server session, not just the cookie.
//!
//! `DELETE /api/public/auth/session` used to clear the cookies and nothing
//! else, so a copied `access_token` kept working until the row expired. The
//! aggregator now calls `admin::sign_out::revoke_cookie_session` first; these
//! cases pin what that call may and may not end.

use axum::http::{HeaderMap, HeaderValue, header};
use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};
use systemprompt_security::{AdminTokenParams, JwtService};
use systemprompt_web_admin_identity::sign_out::revoke_cookie_session;

use crate::tempdb::TempDb;
use crate::{globals, principal};

async fn insert_session(pool: &PgPool, user_id: &UserId) -> SessionId {
    let session_id = SessionId::new(uuid::Uuid::new_v4().to_string());
    sqlx::query("INSERT INTO user_sessions (session_id, user_id) VALUES ($1, $2)")
        .bind(session_id.as_str())
        .bind(user_id.as_str())
        .execute(pool)
        .await
        .expect("insert user session");
    session_id
}

async fn is_revoked(pool: &PgPool, session_id: &SessionId) -> bool {
    sqlx::query_scalar::<_, bool>(
        "SELECT revoked_at IS NOT NULL FROM user_sessions WHERE session_id = $1",
    )
    .bind(session_id.as_str())
    .fetch_one(pool)
    .await
    .expect("read session row")
}

fn cookie_for(user_id: &UserId, session_id: &SessionId) -> HeaderMap {
    let token = JwtService::generate_admin_token(
        globals::signing_keys(),
        &AdminTokenParams {
            user_id,
            session_id,
            email: "signout@contract.test",
            issuer: &globals::jwt_issuer(),
            duration: chrono::Duration::hours(1),
            client_id: None,
        },
    )
    .expect("mint a session token");
    let mut headers = HeaderMap::new();
    headers.insert(
        header::COOKIE,
        HeaderValue::from_str(&format!("access_token={}", token.as_str())).expect("cookie"),
    );
    headers
}

#[tokio::test(flavor = "multi_thread")]
async fn sign_out_revokes_only_the_session_the_cookie_names() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        eprintln!("no DATABASE_URL — skipping sign-out suite");
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let me = credentials.non_admin_user_id.clone();
    let other = credentials.knowledge_worker_user_id.clone();

    let current = insert_session(&db.pool, &me).await;
    let my_other_device = insert_session(&db.pool, &me).await;
    let someone_else = insert_session(&db.pool, &other).await;

    assert!(
        revoke_cookie_session(
            &db.pool,
            globals::boot().token_authority().as_ref(),
            &cookie_for(&me, &current)
        )
        .await
    );
    assert!(is_revoked(&db.pool, &current).await);
    assert!(
        !is_revoked(&db.pool, &my_other_device).await,
        "signing out one browser leaves the user's other sessions alone"
    );

    assert!(
        !revoke_cookie_session(
            &db.pool,
            globals::boot().token_authority().as_ref(),
            &cookie_for(&me, &current)
        )
        .await,
        "a second sign-out with the same cookie is a no-op"
    );

    assert!(
        !revoke_cookie_session(
            &db.pool,
            globals::boot().token_authority().as_ref(),
            &cookie_for(&me, &someone_else)
        )
        .await,
        "a token cannot end a session that belongs to another user"
    );
    assert!(!is_revoked(&db.pool, &someone_else).await);

    db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn sign_out_without_a_valid_token_revokes_nothing() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        eprintln!("no DATABASE_URL — skipping sign-out suite");
        return;
    };

    assert!(
        !revoke_cookie_session(
            &db.pool,
            globals::boot().token_authority().as_ref(),
            &HeaderMap::new()
        )
        .await
    );

    let mut garbage = HeaderMap::new();
    garbage.insert(
        header::COOKIE,
        HeaderValue::from_static("access_token=not-a-jwt"),
    );
    assert!(
        !revoke_cookie_session(
            &db.pool,
            globals::boot().token_authority().as_ref(),
            &garbage
        )
        .await
    );

    db.cleanup().await;
}
