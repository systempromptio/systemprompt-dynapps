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
