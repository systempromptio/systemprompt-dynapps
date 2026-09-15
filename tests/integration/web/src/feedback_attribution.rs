//! Shared fixture steps that turn a raw skill event into a *verified*
//! invocation bound to a published managed resource: enrol the consumer's
//! device, publish a skill, authenticate the event, install the publication
//! on the device and bind the native session. Used by the authentication and
//! retained drill-down suites so both exercise the same attribution path.
use sqlx::PgPool;
use std::collections::BTreeMap;
use systemprompt::identifiers::{
    ConsumerInstallationId, DeviceCertId, ManagedResourceId, NativeSessionId, PublicationId,
    ResourceRevisionId, UserId,
};
use systemprompt::marketplace::managed::consumer::IssuedConsumerCredential;
use systemprompt::marketplace::managed::{
    AssetDigest, AssetFile, ComparisonEvidence, ManagedRepository, NewResource, NewRevision,
    PublicationAction, PublicationRequest, ResourceKind, RevisionFiles, SnapshotProvenance,
    SourceSpec,
};
use systemprompt::models::feedback::receipts::{
    ConsumerReceiptRequest, ReadbackStatus, RuntimeFileReadback, SessionBindingRequest,
};
use systemprompt::models::feedback::{ContentDigest, EvaluatorClient};

pub(super) struct PublishedSkill {
    pub(super) owner: UserId,
    pub(super) resource: ManagedResourceId,
    pub(super) revision: ResourceRevisionId,
    pub(super) publication: PublicationId,
}

pub(super) struct EnrolledDevice {
    pub(super) cert: DeviceCertId,
    pub(super) credential: IssuedConsumerCredential,
}

pub(super) async fn enrol_device(
    pool: &PgPool,
    repo: &ManagedRepository,
    consumer: &UserId,
) -> EnrolledDevice {
    let cert = DeviceCertId::generate();
    sqlx::query(
        "INSERT INTO user_device_certs(id,user_id,fingerprint,label) VALUES($1,$2,$1,'feedback')",
    )
    .bind(cert.as_str())
    .bind(consumer.as_str())
    .execute(pool)
    .await
    .unwrap();
    let credential = repo.issue_consumer_credential(&cert).await.unwrap();
    EnrolledDevice { cert, credential }
}

pub(super) async fn publish_skill(
    repo: &ManagedRepository,
    owner: &UserId,
    upstream_key: &str,
    resource_key: &str,
    body: &[u8],
) -> PublishedSkill {
    let source = repo
        .register_source(owner, "authoring", &SourceSpec::Managed)
        .await
        .unwrap();
    let snapshot = repo
        .capture_snapshot(
            owner,
            &source,
            &SnapshotProvenance {
                source_kind: "managed".to_owned(),
                commit: None,
                tree_digest: AssetDigest::of(body),
                importer_version: "test".to_owned(),
            },
        )
        .await
        .unwrap();
    let resource = repo
        .bind_resource(
            owner,
            &NewResource {
                source_id: source,
                upstream_key: upstream_key.to_owned(),
                kind: ResourceKind::Skill,
                resource_key: resource_key.to_owned(),
            },
        )
        .await
        .unwrap();
    let revision = repo
        .create_revision(
            owner,
            &NewRevision {
                resource_id: resource.clone(),
                snapshot_id: snapshot,
                parent_id: None,
                files: RevisionFiles(BTreeMap::from([(
                    "SKILL.md".to_owned(),
                    AssetFile {
                        bytes: body.to_vec(),
                        media_type: "text/markdown".to_owned(),
                        executable: false,
                    },
                )])),
                dependencies: BTreeMap::new(),
                rationale: "fixture".to_owned(),
            },
        )
        .await
        .unwrap();
    let publication = repo
        .review_and_publish(
            owner,
            owner,
            &PublicationRequest {
                resource_id: resource.clone(),
                revision_id: Some(revision.clone()),
                action: PublicationAction::InitialAdoption,
                expected_generation: 0,
                operation_key: "initial".to_owned(),
                comparison_evidence: ComparisonEvidence::default(),
                limitations: String::new(),
            },
        )
        .await
        .unwrap();
    PublishedSkill {
        owner: owner.clone(),
        resource,
        revision,
        publication: publication.publication_id,
    }
}

pub(super) async fn authenticate_event(
    pool: &PgPool,
    event_id: &str,
    consumer: &UserId,
    device: &EnrolledDevice,
    session: &str,
) {
    sqlx::query("INSERT INTO feedback_authenticated_events(event_id,consumer_id,device_id,host,native_session_id) VALUES($1,$2,$3,'codex',$4)")
        .bind(event_id)
        .bind(consumer.as_str())
        .bind(device.cert.as_str())
        .bind(session)
        .execute(pool)
        .await
        .unwrap();
}

// Grant the consumer the published skill, acknowledge its installation from
// the enrolled device and bind the native session, so the consumer's events
// in that session attribute to the resource on the next drain.
pub(super) async fn attribute(
    repo: &ManagedRepository,
    published: &PublishedSkill,
    consumer: &UserId,
    device: &EnrolledDevice,
    session: &str,
) {
    repo.set_consumer_grant(&published.owner, &published.resource, consumer, true)
        .await
        .unwrap();
    let credential = device.credential.credential.as_str();
    let plan = repo
        .consumer_installation_plan(
            credential,
            &published.resource,
            &published.publication,
            EvaluatorClient::Codex,
        )
        .await
        .unwrap();
    let files = plan
        .canonical_files
        .into_iter()
        .map(|mut file| {
            file.content_check = ReadbackStatus::Verified;
            file.mode_check = ReadbackStatus::Verified;
            file
        })
        .collect();
    let runtime_files = plan
        .runtime_files
        .iter()
        .map(|file| RuntimeFileReadback {
            path: file.path.clone(),
            digest: ContentDigest::of(&file.bytes),
            bytes: file.bytes.len() as u64,
            executable: file.executable,
            content_check: ReadbackStatus::Verified,
            mode_check: ReadbackStatus::Verified,
        })
        .collect();
    let receipt = repo
        .record_consumer_receipt(
            credential,
            &ConsumerReceiptRequest {
                installation_id: ConsumerInstallationId::generate(),
                publication_id: published.publication.clone(),
                resource_id: published.resource.clone(),
                revision_id: published.revision.clone(),
                generation: 1,
                bundle_digest: plan.bundle_digest,
                host: EvaluatorClient::Codex,
                observed_at: chrono::Utc::now(),
                files,
                runtime_files,
            },
        )
        .await
        .unwrap();
    repo.bind_consumer_session(
        credential,
        &SessionBindingRequest {
            receipt_id: receipt.receipt_id,
            host: EvaluatorClient::Codex,
            session_id: NativeSessionId::new(session),
        },
    )
    .await
    .unwrap();
}
