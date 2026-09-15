//! Retained application attestation fixture for publication repository tests.
//! The injected reader supplies owned fixture bytes, not native Git acceptance.
//! Production verification still checks the registered binding, exact closure,
//! content and executable modes before retaining its complete manifest.

use std::collections::BTreeMap;
use std::sync::Arc;
use systemprompt::identifiers::{ResourceRevisionId, UserId};
use systemprompt::marketplace::managed::{
    GitSourceBinding, GitTreeRead, GitTreeReader, GitVerificationService, ManagedRepository,
    RevisionFiles, SourceSpec,
};
use systemprompt::models::feedback::verification::{
    DependencyVerificationInput, DependencyVerificationRequest,
};

struct FixtureReader(RevisionFiles);
impl GitTreeReader for FixtureReader {
    fn read(
        &self,
        _: &GitTreeRead<'_>,
    ) -> systemprompt::marketplace::managed::Result<RevisionFiles> {
        Ok(self.0.clone())
    }
}
pub(crate) async fn retain(
    pool: &sqlx::PgPool,
    repo: &ManagedRepository,
    owner: &UserId,
    revision: &ResourceRevisionId,
    experiment: &str,
) {
    let resource = repo.revision_resource(owner, revision).await.unwrap();
    let bundle = repo.get_revision_bundle(owner, revision).await.unwrap();
    assert_eq!(
        bundle.revisions.len(),
        1,
        "This publication fixture owns one revision and no dependencies"
    );
    let digest = bundle.digest().unwrap();
    let commit = "a".repeat(40);
    let source = repo
        .register_source(
            owner,
            "publication-fixture-git",
            &SourceSpec::Git {
                repository: "https://github.com/systempromptio/publication-fixture.git".to_owned(),
                reference: commit.clone(),
                subdirectory: None,
                credential_reference: None,
            },
        )
        .await
        .unwrap();
    repo.bind_git_verification_source(
        owner,
        owner,
        &GitSourceBinding {
            resource: &resource,
            source: &source,
            relative_root: "fixture-skill",
        },
    )
    .await
    .unwrap();
    let request = DependencyVerificationRequest {
        root_revision_id: revision.clone(),
        revisions: vec![DependencyVerificationInput {
            revision_id: revision.clone(),
            source_id: source,
            exact_commit: commit.clone(),
            relative_root: "fixture-skill".to_owned(),
            dependencies: Vec::new(),
        }],
    };
    let service = GitVerificationService::new(
        repo.clone(),
        Arc::new(FixtureReader(bundle.revision_files(revision).unwrap())),
    );
    let manifest = service
        .verify(owner, &request, &BTreeMap::new())
        .await
        .unwrap();
    manifest.validate_complete().unwrap();
    assert_eq!(manifest.bundle_digest.as_str(), digest.as_str());
    sqlx::query("INSERT INTO managed_evaluation_attestations(owner_id,resource_id,revision_id,bundle_digest,experiment_id,campaign_id,evidence_digest,source_commit,attested_by) VALUES($1,$2,$3,$4,$5,'fixture-campaign','fixture-evidence',$6,$1)")
        .bind(owner.as_str()).bind(resource.as_str()).bind(revision.as_str()).bind(digest.as_str()).bind(experiment).bind(commit).execute(pool).await.unwrap();
}
