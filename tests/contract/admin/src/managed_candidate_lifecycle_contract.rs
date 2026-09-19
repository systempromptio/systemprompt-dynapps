//! Text candidates are new immutable revisions: an HTTP edit must diff from,
//! never overwrite, its captured baseline.

use axum::http::StatusCode;
use serde_json::Value;
use systemprompt::marketplace::managed::RevisionFiles;

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal};

fn origin() -> String {
    systemprompt::config::ProfileBootstrap::get()
        .expect("fixture profile")
        .server
        .api_external_url
        .clone()
}

#[tokio::test(flavor = "multi_thread")]
async fn candidate_edit_compares_against_and_preserves_its_baseline_revision() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let request_origin = origin();
    let capture = Call {
        method: "post",
        path: "/api/public/admin/managed/baselines/super-admin",
        principal: Principal::Admin,
        content_type: None,
        body: None,
    };
    let (status, _) = app
        .response_headers_with(capture, &[("origin", &request_origin)])
        .await;
    assert_eq!(status, StatusCode::CREATED);
    let baseline: String = sqlx::query_scalar("SELECT v.id FROM managed_revisions v JOIN managed_resources r ON r.id = v.resource_id WHERE r.resource_key = 'admin_daily_brief' ORDER BY v.created_at DESC LIMIT 1")
        .fetch_one(&*db.pool).await.expect("captured baseline");
    let files_path = format!("/api/public/admin/managed/revisions/{baseline}/files");
    let (status, baseline_before) = app.call(Call::get(&files_path, Principal::Admin)).await;
    assert_eq!(status, StatusCode::OK);
    let candidate_path = format!("/api/public/admin/managed/revisions/{baseline}/candidates");
    let edit = "{\"path\":\"SKILL.md\",\"content\":\"# Revised brief\\n\\nCite the source before each claim.\",\"rationale\":\"Require attributable evidence.\"}";
    let candidate_call = Call::json("post", &candidate_path, Principal::Admin, edit);
    let (status, _) = app
        .response_headers_with(candidate_call, &[("origin", &request_origin)])
        .await;
    assert_eq!(status, StatusCode::CREATED);
    let candidate: String = sqlx::query_scalar(
        "SELECT id FROM managed_revisions WHERE parent_id = $1 ORDER BY created_at DESC LIMIT 1",
    )
    .bind(&baseline)
    .fetch_one(&*db.pool)
    .await
    .expect("candidate revision");
    let compare_path =
        format!("/api/public/admin/managed/revisions/{baseline}/compare/{candidate}");
    let (status, comparison) = app.call(Call::get(&compare_path, Principal::Admin)).await;
    assert_eq!(status, StatusCode::OK, "comparison: {comparison}");
    let comparison: Value = serde_json::from_str(&comparison).expect("comparison JSON");
    assert!(
        comparison["changes"]
            .as_array()
            .expect("comparison changes")
            .iter()
            .any(|change| change["path"] == "SKILL.md" && change["kind"] == "modified"),
        "the comparison identifies the edited file as modified: {comparison:?}"
    );
    let (status, baseline_files) = app.call(Call::get(&files_path, Principal::Admin)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        baseline_files, baseline_before,
        "the baseline bytes remain immutable"
    );
    let (status, candidate_files) = app
        .call(Call::get(
            &format!("/api/public/admin/managed/revisions/{candidate}/files"),
            Principal::Admin,
        ))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_ne!(
        candidate_files, baseline_before,
        "candidate is a distinct revision"
    );
    let candidate_files: RevisionFiles =
        serde_json::from_str(&candidate_files).expect("candidate files JSON");
    let skill = candidate_files
        .0
        .get("SKILL.md")
        .expect("edited skill file");
    assert_eq!(
        std::str::from_utf8(&skill.bytes).expect("candidate text bytes"),
        "# Revised brief\n\nCite the source before each claim."
    );
    db.cleanup().await;
}
