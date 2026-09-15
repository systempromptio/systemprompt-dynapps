//! An execution admission that accepts deterministic fixtures. Production
//! admission demands a reviewed native proof for each variant's exact
//! client/version/image and this harness ships none, so every experiment the
//! evaluation suites create would be refused at budget reservation — before
//! the fences, pins and owner scoping they actually test ever ran.

use std::collections::BTreeMap;
use std::sync::Arc;
use systemprompt::evaluation::capabilities::{ExecutionAdmission, paired_variants};
use systemprompt::evaluation::experiments::{ExecutionMode, ExperimentSpec};
use systemprompt::evaluation::repository::experiments::{EvaluationRepositories, EvaluationSeams};
use systemprompt::identifiers::{ManagedResourceId, ResourceRevisionId, UserId};
use systemprompt::marketplace::managed::{
    AssetDigest, AssetFile, ManagedRepository, NewResource, NewRevision, ResourceKind,
    RevisionFiles, SnapshotProvenance, SourceSpec,
};

use astound_test_common::TempDb;

#[derive(Debug)]
struct FixtureAdmission;

impl ExecutionAdmission for FixtureAdmission {
    fn admit(&self, spec: &ExperimentSpec) -> systemprompt::evaluation::Result<()> {
        spec.validate()?;
        paired_variants(spec)?;
        if spec.execution_mode != ExecutionMode::Fixture {
            return Err(systemprompt::evaluation::EvaluationError::InvalidSpec(
                "The integration harness admits deterministic fixtures only".to_owned(),
            ));
        }
        Ok(())
    }
}

pub(crate) fn fixture_admission() -> Arc<dyn ExecutionAdmission> {
    Arc::new(FixtureAdmission)
}

// Why: every evaluation repository now reads foreign rows through the shared
// seams core's runtime wires, so the suites build the whole set once, with the
// fixture admission in place of the native-proof gate, and pick from it.
pub(crate) fn evaluation_repositories(db: &TempDb) -> EvaluationRepositories {
    let pool = db.db_pool();
    let managed = systemprompt::marketplace::managed::ManagedRepository::new(&pool)
        .expect("build the managed repository");
    let seams = EvaluationSeams {
        trace: Arc::new(
            systemprompt::ai::repository::AiRequestRepository::new(&pool)
                .expect("build the AI request repository"),
        ),
        sessions: Arc::new(
            systemprompt::users::UsersAiSessionProvider::from_repository(
                systemprompt::users::SessionRepository::new(&pool)
                    .expect("build the session repository"),
            ),
        ),
        managed_revisions: Arc::new(managed),
    };
    EvaluationRepositories::with_admission(&pool, seams, fixture_admission())
        .expect("build the evaluation repositories")
}

// Why: a campaign owns its baseline, so the policy must name a managed
// revision the owner holds; a generated id is refused before the transition
// and report fences under test ever run.
pub(crate) async fn seed_managed_baseline(
    db: &TempDb,
    owner: &UserId,
    key: &str,
) -> (ManagedResourceId, ResourceRevisionId) {
    let pool = db.db_pool();
    let managed = ManagedRepository::new(&pool).expect("build the managed repository");
    let source = managed
        .register_source(owner, key, &SourceSpec::Managed)
        .await
        .expect("register the baseline source");
    let snapshot = managed
        .capture_snapshot(
            owner,
            &source,
            &SnapshotProvenance {
                source_kind: "managed".to_owned(),
                commit: None,
                tree_digest: AssetDigest::of(key.as_bytes()),
                importer_version: "test-fixture".to_owned(),
            },
        )
        .await
        .expect("capture the baseline snapshot");
    let resource = managed
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
        .expect("bind the baseline resource");
    let files = RevisionFiles(BTreeMap::from([(
        "index.md".to_owned(),
        AssetFile {
            bytes: b"# baseline".to_vec(),
            media_type: "text/markdown".to_owned(),
            executable: false,
        },
    )]));
    let revision = managed
        .create_revision(
            owner,
            &NewRevision {
                resource_id: resource.clone(),
                snapshot_id: snapshot,
                parent_id: None,
                files,
                dependencies: BTreeMap::new(),
                rationale: "baseline".to_owned(),
            },
        )
        .await
        .expect("create the baseline revision");
    (resource, revision)
}
