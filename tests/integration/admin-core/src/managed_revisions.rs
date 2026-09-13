//! Immutable revision storage preserves binary files and rejects cross-owner
//! links.

use crate::fixtures::{insert_user, unclaimed_email, unique};
use crate::tempdb::TempDb;
use std::collections::BTreeMap;
use systemprompt::identifiers::UserId;
use systemprompt::marketplace::managed::{
    AssetDigest, AssetFile, ManagedError, ManagedRepository, NewResource, NewRevision,
    ResourceKind, RevisionFiles, SnapshotProvenance, SourceSpec,
};

async fn revision_input(repo: &ManagedRepository, owner: &UserId, key: &str) -> NewRevision {
    let source = repo
        .register_source(owner, "control-plane", &SourceSpec::Managed)
        .await
        .expect("source");
    let provenance = SnapshotProvenance {
        source_kind: "managed".to_owned(),
        commit: None,
        tree_digest: AssetDigest::of(b"baseline"),
        importer_version: "test-v1".to_owned(),
    };
    let snapshot = repo
        .capture_snapshot(owner, &source, &provenance)
        .await
        .expect("snapshot");
    let resource = repo
        .bind_resource(
            owner,
            &NewResource {
                source_id: source,
                upstream_key: key.to_owned(),
                kind: ResourceKind::Skill,
                resource_key: key.to_owned(),
            },
        )
        .await
        .expect("resource");
    NewRevision {
        resource_id: resource,
        snapshot_id: snapshot,
        parent_id: None,
        files: RevisionFiles(BTreeMap::from([
            (
                "SKILL.md".to_owned(),
                AssetFile {
                    bytes: b"Read evidence before reporting.".to_vec(),
                    media_type: "text/markdown".to_owned(),
                    executable: false,
                },
            ),
            (
                "assets/binary.dat".to_owned(),
                AssetFile {
                    bytes: vec![0, 255, 128, 13, 10],
                    media_type: "application/octet-stream".to_owned(),
                    executable: false,
                },
            ),
        ])),
        dependencies: BTreeMap::new(),
        rationale: "Freeze baseline".to_owned(),
    }
}

#[tokio::test]
async fn revision_round_trip_is_immutable_idempotent_and_owner_scoped() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let alice = insert_user(
        &db.pool,
        &unique("revision-alice"),
        &unclaimed_email("revision-alice"),
    )
    .await;
    let bob = insert_user(
        &db.pool,
        &unique("revision-bob"),
        &unclaimed_email("revision-bob"),
    )
    .await;
    let repo = ManagedRepository::new((*db.pool).clone());
    let input = revision_input(&repo, &alice, &unique("managed_skill")).await;
    let id = repo
        .create_revision(&alice, &input)
        .await
        .expect("revision");
    assert_eq!(
        id,
        repo.create_revision(&alice, &input).await.expect("retry")
    );
    let files = repo.get_revision_files(&alice, &id).await.expect("files");
    assert_eq!(files.0["assets/binary.dat"].bytes, [0, 255, 128, 13, 10]);
    assert!(matches!(
        repo.get_revision(&bob, &id).await,
        Err(ManagedError::Unavailable)
    ));
    assert!(matches!(
        repo.create_revision(&bob, &input).await,
        Err(ManagedError::Unavailable)
    ));
    let mutation = sqlx::query("UPDATE managed_revisions SET rationale='changed' WHERE id=$1")
        .bind(id.as_str())
        .execute(db.pool.as_ref())
        .await;
    assert!(
        mutation.is_err(),
        "database must reject changes to immutable revisions"
    );
    let mut candidate = input.clone();
    candidate.parent_id = Some(id.clone());
    candidate.files.0.get_mut("SKILL.md").expect("skill").bytes =
        b"Cite evidence before reporting.".to_vec();
    let next = repo
        .create_revision(&alice, &candidate)
        .await
        .expect("candidate");
    assert_ne!(id, next);
    let summaries = repo.list_resources(&alice, 0).await.expect("resources");
    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0].revision_count, 2);
    assert_eq!(summaries[0].latest_revision, Some(next.clone()));
    assert!(
        repo.list_resources(&bob, 0)
            .await
            .expect("foreign listing")
            .is_empty()
    );
    assert_eq!(
        repo.list_revisions(&alice, &input.resource_id, 0)
            .await
            .expect("history")
            .len(),
        2
    );
    assert_eq!(
        repo.get_revision(&alice, &next)
            .await
            .expect("manifest")
            .parent_id,
        Some(id)
    );
}

#[tokio::test]
async fn resources_reject_source_collisions_and_foreign_parents() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let alice = insert_user(
        &db.pool,
        &unique("source-alice"),
        &unclaimed_email("source-alice"),
    )
    .await;
    let bob = insert_user(
        &db.pool,
        &unique("source-bob"),
        &unclaimed_email("source-bob"),
    )
    .await;
    let repo = ManagedRepository::new((*db.pool).clone());
    let key = unique("source_skill");
    let input = revision_input(&repo, &alice, &key).await;
    let id = repo
        .create_revision(&alice, &input)
        .await
        .expect("baseline");
    let foreign_source = repo
        .register_source(&bob, "control-plane", &SourceSpec::Managed)
        .await
        .expect("bob source");
    assert!(matches!(
        repo.bind_resource(
            &bob,
            &NewResource {
                source_id: foreign_source,
                upstream_key: key.clone(),
                kind: ResourceKind::Skill,
                resource_key: key,
            }
        )
        .await,
        Err(ManagedError::Conflict(_))
    ));
    let mut other = revision_input(&repo, &bob, &unique("other_skill")).await;
    other.parent_id = Some(id);
    assert!(matches!(
        repo.create_revision(&bob, &other).await,
        Err(ManagedError::Unavailable)
    ));
}

#[tokio::test]
async fn importing_the_current_baseline_is_repeatable_and_preserves_source_bytes() {
    use systemprompt::marketplace::managed::capture_skills;
    let Some(db) = TempDb::create().await else {
        return;
    };
    let owner = insert_user(
        &db.pool,
        &unique("baseline-owner"),
        &unclaimed_email("baseline-owner"),
    )
    .await;
    let repo = ManagedRepository::new((*db.pool).clone());
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../services");
    let ids = [
        "admin_daily_brief",
        "admin_critical_projects",
        "admin_ai_usage",
        "systemprompt_cli",
    ]
    .map(str::to_owned);
    let captured = capture_skills(&root, &ids).expect("real baseline source");
    let source = repo
        .register_source(
            &owner,
            "baseline",
            &SourceSpec::LocalTree {
                root: root.to_string_lossy().into_owned(),
            },
        )
        .await
        .expect("source");
    let first = repo
        .import_skills(&owner, &source, &captured, None)
        .await
        .expect("import");
    let retry = repo
        .import_skills(&owner, &source, &captured, None)
        .await
        .expect("retry");
    assert_eq!(first.revisions.len(), 4);
    assert_eq!(first.snapshot_id, retry.snapshot_id);
    assert_eq!(first.revisions, retry.revisions);
    for (skill, revision) in first.revisions {
        let stored = repo
            .get_revision_files(&owner, &revision)
            .await
            .expect("stored files");
        assert_eq!(
            stored.0["SKILL.md"].bytes,
            std::fs::read(root.join("skills").join(skill).join("SKILL.md")).expect("source")
        );
    }
}

#[tokio::test]
async fn text_candidates_inherit_assets_and_comparisons_reject_foreign_resources() {
    use systemprompt::marketplace::managed::TextCandidate;
    let Some(db) = TempDb::create().await else {
        return;
    };
    let owner = insert_user(
        &db.pool,
        &unique("candidate-owner"),
        &unclaimed_email("candidate-owner"),
    )
    .await;
    let repo = ManagedRepository::new((*db.pool).clone());
    let input = revision_input(&repo, &owner, &unique("candidate_skill")).await;
    let baseline = repo
        .create_revision(&owner, &input)
        .await
        .expect("baseline");
    let candidate = repo
        .create_text_candidate(
            &owner,
            &baseline,
            &TextCandidate {
                path: "SKILL.md".to_owned(),
                content: "Check and cite each claim.".to_owned(),
                rationale: "Improve evidence coverage".to_owned(),
            },
        )
        .await
        .expect("candidate");
    let files = repo
        .get_revision_files(&owner, &candidate)
        .await
        .expect("files");
    assert_eq!(
        files.0["assets/binary.dat"].bytes,
        input.files.0["assets/binary.dat"].bytes
    );
    assert_eq!(files.0["SKILL.md"].bytes, b"Check and cite each claim.");
    let comparison = repo
        .compare_revisions(&owner, &baseline, &candidate)
        .await
        .expect("comparison");
    assert_eq!(comparison.changes.len(), 1);
    assert_eq!(comparison.changes[0].path, "SKILL.md");
    assert!(!comparison.dependencies_changed);
    assert!(!comparison.source_snapshot_changed);
    let other_input = revision_input(&repo, &owner, &unique("unrelated_skill")).await;
    let other = repo
        .create_revision(&owner, &other_input)
        .await
        .expect("other resource");
    assert!(
        repo.compare_revisions(&owner, &baseline, &other)
            .await
            .is_err()
    );
    assert!(
        repo.create_text_candidate(
            &owner,
            &baseline,
            &TextCandidate {
                path: "assets/binary.dat".to_owned(),
                content: "not a binary".to_owned(),
                rationale: "Invalid edit".to_owned(),
            }
        )
        .await
        .is_err()
    );
}

#[tokio::test]
async fn bundle_resolves_the_exact_owned_dependency_closure() {
    use systemprompt::marketplace::managed::DependencyRef;
    let Some(db) = TempDb::create().await else {
        return;
    };
    let owner = insert_user(
        &db.pool,
        &unique("bundle-owner"),
        &unclaimed_email("bundle-owner"),
    )
    .await;
    let foreign = insert_user(
        &db.pool,
        &unique("bundle-foreign"),
        &unclaimed_email("bundle-foreign"),
    )
    .await;
    let repo = ManagedRepository::new((*db.pool).clone());
    let mut supporting = revision_input(&repo, &owner, &unique("bundle_reference")).await;
    supporting
        .files
        .0
        .get_mut("assets/binary.dat")
        .unwrap()
        .executable = true;
    let dependency = repo.create_revision(&owner, &supporting).await.unwrap();
    let mut primary = revision_input(&repo, &owner, &unique("bundle_skill")).await;
    primary.dependencies.insert(
        "reference".to_owned(),
        DependencyRef {
            revision_id: dependency.clone(),
            digest: repo
                .get_revision(&owner, &dependency)
                .await
                .unwrap()
                .digest()
                .unwrap(),
        },
    );
    let root = repo.create_revision(&owner, &primary).await.unwrap();
    let bundle = repo.get_revision_bundle(&owner, &root).await.unwrap();
    assert_eq!(bundle.revisions.len(), 2);
    assert_eq!(
        bundle.assets.len(),
        2,
        "identical assets deduplicate across revisions"
    );
    assert!(bundle.revision_files(&dependency).unwrap().0["assets/binary.dat"].executable);
    assert_eq!(
        bundle.canonical_bytes().unwrap(),
        repo.get_revision_bundle(&owner, &root)
            .await
            .unwrap()
            .canonical_bytes()
            .unwrap()
    );
    assert!(matches!(
        repo.get_revision_bundle(&foreign, &root).await,
        Err(ManagedError::Unavailable)
    ));
}

#[tokio::test]
async fn reviewed_publications_are_idempotent_fenced_and_generation_pinned() {
    use systemprompt::marketplace::managed::{
        ManagedResolution, PublicationAction, PublicationRequest, TextCandidate,
    };
    let Some(db) = TempDb::create().await else {
        return;
    };
    let owner = insert_user(
        &db.pool,
        &unique("publication-owner"),
        &unclaimed_email("publication-owner"),
    )
    .await;
    let repo = ManagedRepository::new((*db.pool).clone());
    let key = unique("published_skill");
    let input = revision_input(&repo, &owner, &key).await;
    let baseline = repo.create_revision(&owner, &input).await.unwrap();
    assert!(matches!(
        repo.resolve_managed(&owner, ResourceKind::Skill, &key)
            .await
            .unwrap(),
        ManagedResolution::NeverAdopted { .. }
    ));
    let initial_request = PublicationRequest {
        resource_id: input.resource_id.clone(),
        revision_id: Some(baseline.clone()),
        action: PublicationAction::InitialAdoption,
        expected_generation: 0,
        operation_key: unique("initial-adoption"),
        comparison_evidence: serde_json::json!({"kind": "unevaluated_baseline"}),
        limitations: "Initial adoption is not an improvement claim".to_owned(),
    };
    let initial = repo
        .review_and_publish(&owner, &owner, &initial_request)
        .await
        .unwrap();
    assert_eq!(
        initial,
        repo.review_and_publish(&owner, &owner, &initial_request)
            .await
            .unwrap()
    );
    let initial_digest = initial.bundle_digest.clone().unwrap();
    assert_eq!(
        repo.get_publication_bundle(&owner, &input.resource_id, 1, &initial_digest)
            .await
            .unwrap()
            .root,
        baseline
    );

    let candidate = repo
        .create_text_candidate(
            &owner,
            &baseline,
            &TextCandidate {
                path: "SKILL.md".to_owned(),
                content: "Check, reconcile and cite each claim.".to_owned(),
                rationale: "Address evidence mismatch".to_owned(),
            },
        )
        .await
        .unwrap();
    let improvement = repo
        .review_and_publish(
            &owner,
            &owner,
            &PublicationRequest {
                resource_id: input.resource_id.clone(),
                revision_id: Some(candidate.clone()),
                action: PublicationAction::PublishImprovement,
                expected_generation: 1,
                operation_key: unique("publish-improvement"),
                comparison_evidence: serde_json::json!({"experiment_id": "exp_test"}),
                limitations: "Fixture comparison only".to_owned(),
            },
        )
        .await
        .unwrap();
    assert_eq!(improvement.generation, 2);
    assert_eq!(
        repo.get_publication_bundle(&owner, &input.resource_id, 1, &initial_digest)
            .await
            .unwrap()
            .root,
        baseline,
        "a download remains bound to its advertised generation"
    );
    assert!(matches!(
        repo.resolve_managed(&owner, ResourceKind::Skill, &key)
            .await
            .unwrap(),
        ManagedResolution::Published {
            generation: 2,
            revision_id,
            ..
        } if revision_id == candidate
    ));

    let stale = PublicationRequest {
        resource_id: input.resource_id.clone(),
        revision_id: Some(candidate),
        action: PublicationAction::Rollback,
        expected_generation: 1,
        operation_key: unique("stale-rollback"),
        comparison_evidence: serde_json::json!({"reason": "stale"}),
        limitations: String::new(),
    };
    assert!(matches!(
        repo.review_and_publish(&owner, &owner, &stale).await,
        Err(ManagedError::Conflict(_))
    ));
    let withdrawal = repo
        .review_and_publish(
            &owner,
            &owner,
            &PublicationRequest {
                resource_id: input.resource_id.clone(),
                revision_id: None,
                action: PublicationAction::Withdraw,
                expected_generation: 2,
                operation_key: unique("withdraw"),
                comparison_evidence: serde_json::json!({"reason": "upstream_deleted"}),
                limitations: "Disk content must not become authoritative".to_owned(),
            },
        )
        .await
        .unwrap();
    assert_eq!(withdrawal.generation, 3);
    assert!(matches!(
        repo.resolve_managed(&owner, ResourceKind::Skill, &key)
            .await
            .unwrap(),
        ManagedResolution::Withdrawn { generation: 3, .. }
    ));
    let rows: i64 =
        sqlx::query_scalar("SELECT count(*) FROM managed_distribution_outbox WHERE owner_id=$1")
            .bind(owner.as_str())
            .fetch_one(db.pool.as_ref())
            .await
            .unwrap();
    assert_eq!(rows, 3, "each committed generation has one outbox event");
}
