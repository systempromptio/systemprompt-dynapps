//! Public managed-resource reads must expose the reviewed generation and its
//! immutable bundle, rather than only accepting a publication write.

use std::collections::BTreeMap;

use axum::http::StatusCode;
use serde_json::Value;
use systemprompt::identifiers::{ManagedResourceId, ResourceRevisionId};
use systemprompt::marketplace::managed::{
    ComparisonEvidence, ManagedRepository, PublicationAction, PublicationRequest, RevisionBundle,
};

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal, seed};

fn origin() -> String {
    systemprompt::config::ProfileBootstrap::get()
        .expect("fixture profile")
        .server
        .api_external_url
        .clone()
}

#[tokio::test(flavor = "multi_thread")]
async fn publication_history_and_bundle_expose_the_reviewed_generation() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let owner = credentials.admin_user_id.clone();
    let app = App::new(&db.pool, credentials);
    let capture = Call {
        method: "post",
        path: "/api/public/admin/managed/baselines/super-admin",
        principal: Principal::Admin,
        content_type: None,
        body: None,
    };
    let request_origin = origin();
    let (status, _) = app
        .response_headers_with(capture, &[("origin", &request_origin)])
        .await;
    assert_eq!(status, StatusCode::CREATED);

    let (resource_id, revision_id): (String, String) = sqlx::query_as(
        "SELECT r.id, v.id
         FROM managed_resources r
         JOIN managed_revisions v ON v.owner_id = r.owner_id AND v.resource_id = r.id
         WHERE r.owner_id = $1 AND r.resource_key = 'systemprompt_cli'
         ORDER BY v.created_at DESC LIMIT 1",
    )
    .bind(owner.as_str())
    .fetch_one(&*db.pool)
    .await
    .expect("captured baseline revision");
    let resource = ManagedResourceId::new(resource_id.clone());
    let revision = ResourceRevisionId::new(revision_id.clone());
    let repository = ManagedRepository::new(&db.db_pool());
    let publication = repository
        .review_and_publish(
            &owner,
            &owner,
            &PublicationRequest {
                resource_id: resource.clone(),
                revision_id: Some(revision),
                action: PublicationAction::InitialAdoption,
                expected_generation: 0,
                operation_key: seed::unique("publication-api"),
                comparison_evidence: ComparisonEvidence {
                    recorded: BTreeMap::from([(
                        "bundle_content_hash".to_owned(),
                        Value::from("publication-api-provenance"),
                    )]),
                },
                limitations: "Reviewed through the public API contract".to_owned(),
                cohort: None,
            },
        )
        .await
        .expect("publish captured baseline");
    let digest = publication
        .bundle_digest
        .as_ref()
        .expect("published revision has a bundle digest");

    let history_path = format!("/api/public/admin/managed/resources/{resource_id}/publications");
    let (status, body) = app.call(Call::get(&history_path, Principal::Admin)).await;
    assert_eq!(status, StatusCode::OK, "publication history: {body}");
    let history: Vec<Value> = serde_json::from_str(&body).expect("publication history JSON");
    let entry = history
        .iter()
        .find(|entry| entry["decision"]["publication_id"] == publication.publication_id.as_str())
        .expect("the just-reviewed generation is returned");
    assert_eq!(entry["decision"]["generation"], 1);
    assert_eq!(entry["decision"]["action"], "initial_adoption");
    assert_eq!(entry["decision"]["bundle_digest"], digest.as_str());

    let bundle_path = format!(
        "/api/public/admin/managed/resources/{resource_id}/publications/1/bundle?digest={}",
        digest.as_str()
    );
    let (status, body) = app.call(Call::get(&bundle_path, Principal::Admin)).await;
    assert_eq!(status, StatusCode::OK, "publication bundle: {body}");
    let bundle: RevisionBundle = serde_json::from_str(&body).expect("publication bundle JSON");
    assert_eq!(bundle.root.as_str(), revision_id);
    assert_eq!(
        bundle.canonical_bytes().expect("canonical bundle"),
        body.as_bytes()
    );
    db.cleanup().await;
}
