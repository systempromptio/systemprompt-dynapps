//! Managed authoring preserves identity and instruction bytes through HTTP.

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal};
use axum::http::StatusCode;

fn request<'a>(method: &'a str, path: &'a str, principal: Principal) -> Call<'a> {
    Call {
        method,
        path,
        principal,
        content_type: None,
        body: None,
    }
}

#[tokio::test]
async fn baseline_capture_requires_origin_and_organizational_revisions_are_shared_by_admins() {
    assert!(globals::init());
    let db = TempDb::create().await.expect("PostgreSQL required");
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let path = "/api/public/admin/managed/baselines/super-admin";
    let (status, _) = app.call(request("post", path, Principal::Admin)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let origin = globals::boot().config().api_external_url.clone();
    for principal in [Principal::ProjectManager, Principal::NonAdmin] {
        let (status, _) = app
            .response_headers_with(request("post", path, principal), &[("origin", &origin)])
            .await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }
    let (status, _) = app
        .response_headers_with(
            request("post", path, Principal::Admin),
            &[("origin", &origin)],
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    let id: String = sqlx::query_scalar("SELECT v.id FROM managed_revisions v JOIN managed_resources r ON r.id=v.resource_id WHERE r.resource_key='systemprompt_cli'")
        .fetch_one(&*db.pool).await.expect("captured daily brief");
    let path = format!("/api/public/admin/managed/revisions/{id}");
    let (status, _) = app
        .call(request("get", &path, Principal::PlatformAdmin))
        .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "authorized administrators share organizational revisions without changing the owner"
    );
    let (status, _) = app.call(request("get", &path, Principal::Admin)).await;
    assert_eq!(status, StatusCode::OK);
    let path = format!("/admin/analysis/revisions/{id}");
    let (status, body) = app.call(request("get", &path, Principal::Admin)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, _) = app
        .call(request("get", &path, Principal::PlatformAdmin))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_bundle(&app, &id).await;
    db.cleanup().await;
}

async fn assert_bundle(app: &App, id: &str) {
    use systemprompt::marketplace::managed::RevisionBundle;
    let path = format!("/admin/analysis/revisions/{id}");
    let (status, _) = app
        .call(request("get", &path, Principal::ProjectManager))
        .await;
    assert_eq!(status, StatusCode::SEE_OTHER, "{path}");
    let path = format!("/api/public/admin/managed/revisions/{id}/bundle");
    let (status, body) = app.call(request("get", &path, Principal::Admin)).await;
    assert_eq!(status, StatusCode::OK);
    let bundle: RevisionBundle = serde_json::from_str(&body).unwrap();
    assert_eq!(bundle.root.as_str(), id);
    assert_eq!(bundle.canonical_bytes().unwrap(), body.as_bytes());
    let (status, _) = app
        .call(request("get", &path, Principal::PlatformAdmin))
        .await;
    assert_eq!(status, StatusCode::OK);
}
