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
