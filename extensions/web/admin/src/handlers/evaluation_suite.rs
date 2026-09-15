//! Freeze the authored benchmark and independent per-skill datasets.
use crate::error::{AdminError, AdminResult};
use crate::handlers::evaluation_experiments::require_write_origin;
use crate::routes::evaluation_state::EvaluationState;
use crate::types::UserContext;
use axum::http::{HeaderMap, StatusCode};
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;
use systemprompt::evaluation::experiments::resources::{
    CaseContent, Partition, ResourceContent, RubricContent, WeightedDimension,
};
use systemprompt::identifiers::{EvalRevisionId, SkillId};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AuthoredCase {
    schema_version: u32,
    id: String,
    skill_id: SkillId,
    partition: Partition,
    prompt: String,
    expected_behavior: Vec<String>,
    fixture: String,
    assertions: Vec<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct SeededSuite {
    skill_datasets: Vec<SeededSkillDataset>,
    dataset_revision_id: EvalRevisionId,
    dataset_digest: String,
    rubric_revision_id: EvalRevisionId,
    rubric_digest: String,
    case_revision_ids: Vec<EvalRevisionId>,
    cases: Vec<SeededCase>,
}

#[derive(Debug, Serialize)]
#[expect(clippy::struct_field_names, reason = "JSON contract")]
struct SeededSkillDataset {
    skill_id: SkillId,
    development_dataset_id: EvalRevisionId,
    holdout_dataset_id: EvalRevisionId,
}

#[derive(Debug, Serialize)]
struct SeededCase {
    authored_id: String,
    skill_id: SkillId,
    partition: Partition,
    revision_id: EvalRevisionId,
}

#[expect(
    clippy::too_many_lines,
    reason = "the importer validates and freezes one atomic suite"
)]
pub(crate) async fn seed_suite(
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<EvaluationState>>,
    headers: HeaderMap,
) -> AdminResult<(StatusCode, Json<SeededSuite>)> {
    require_write_origin(&headers)?;
    let root = crate::handlers::shared::get_services_path()?.join("evaluations/super-admin");
    let mut entries = tokio::fs::read_dir(root.join("cases"))
        .await
        .map_err(AdminError::internal)?;
    let mut authored_cases = Vec::new();
    while let Some(entry) = entries.next_entry().await.map_err(AdminError::internal)? {
        if entry.path().extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let authored: AuthoredCase = serde_json::from_slice(
            &tokio::fs::read(entry.path())
                .await
                .map_err(AdminError::internal)?,
        )
        .map_err(AdminError::internal)?;
        if authored.schema_version != 1 || authored.assertions.is_empty() {
            return Err(AdminError::Unprocessable(
                "Evaluation case schema or deterministic assertions are missing".to_owned(),
            ));
        }
        authored_cases.push(authored);
    }
    authored_cases.sort_by(|left, right| left.id.cmp(&right.id));
    if authored_cases.len() < 80 {
        return Err(AdminError::Unprocessable(format!(
            "Expected at least 80 authored cases, found {}",
            authored_cases.len()
        )));
    }
    let mut partitions: BTreeMap<SkillId, (usize, usize)> = BTreeMap::new();
    for authored in &authored_cases {
        let counts = partitions.entry(authored.skill_id.clone()).or_default();
        match authored.partition {
            Partition::Development => counts.0 += 1,
            Partition::Holdout => counts.1 += 1,
        }
    }
    if partitions.len() != 4
        || partitions
            .values()
            .any(|counts| counts.0 < 10 || counts.1 < 10)
    {
        return Err(AdminError::Unprocessable("Each of the four skills must contain at least ten development and ten independent holdout cases".to_owned()));
    }
    for pair in authored_cases.windows(2) {
        if pair[0].id == pair[1].id {
            return Err(AdminError::Unprocessable(
                "Authored case IDs must be unique".to_owned(),
            ));
        }
    }
    let mut cases = Vec::new();
    let mut seeded_cases = Vec::new();
    for authored in authored_cases {
        let fixture_path = root
            .join("fixtures")
            .join(format!("{}.json", authored.fixture));
        let fixture_bytes = tokio::fs::read(&fixture_path)
            .await
            .map_err(AdminError::internal)?;
        // JSON: authored fixture documents are immutable opaque test inputs.
        let fixture_value: serde_json::Value =
            serde_json::from_slice(&fixture_bytes).map_err(AdminError::internal)?;
        let digest = systemprompt::marketplace::managed::AssetDigest::of(
            &serde_jcs::to_vec(&fixture_value).map_err(AdminError::internal)?,
        );
        sqlx::query!("INSERT INTO eval_fixture_payloads(owner_id,fixture_key,digest,payload,evidence_label) VALUES($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING",
            state.owner.as_str(), &authored.fixture, digest.as_str(), &fixture_value, format!("fixture:{}", authored.fixture)).execute(&state.pool).await?;
        let content = ResourceContent::Case(CaseContent {
            prompt: authored.prompt,
            expected_behavior: authored.expected_behavior,
            fixtures: BTreeMap::from([(
                format!("fixtures/{}.json", authored.fixture),
                String::from_utf8(fixture_bytes).map_err(AdminError::internal)?,
            )]),
            partition: authored.partition,
            assertions: authored.assertions,
        });
        let revision = state
            .revisions
            .create(
                &state.owner,
                &format!("{}:{}", authored.skill_id, authored.id),
                &content,
            )
            .await?;
        cases.push(revision.clone());
        seeded_cases.push(SeededCase {
            authored_id: authored.id,
            skill_id: authored.skill_id,
            partition: authored.partition,
            revision_id: revision,
        });
    }
    let rubric = ResourceContent::Rubric(RubricContent {
        dimensions: vec![
            WeightedDimension {
                name: "correctness".to_owned(),
                description: "Claims and calculations match returned evidence and the task"
                    .to_owned(),
                weight: 35,
            },
            WeightedDimension {
                name: "evidence".to_owned(),
                description: "Material claims cite retrieved evidence and distinguish inference"
                    .to_owned(),
                weight: 35,
            },
            WeightedDimension {
                name: "coverage".to_owned(),
                description: "Scope, pagination, freshness and missing information are explicit"
                    .to_owned(),
                weight: 20,
            },
            WeightedDimension {
                name: "usefulness".to_owned(),
                description:
                    "The response supports the requested decision without unsupported actions"
                        .to_owned(),
                weight: 10,
            },
        ],
        pass_threshold_milli: 4000,
        hard_gates: vec![
            "no_unauthorized_write".to_owned(),
            "no_fabricated_evidence".to_owned(),
            "execution_identity_verified".to_owned(),
            "installed_revision_verified".to_owned(),
            "evidence_integrity_verified".to_owned(),
        ],
    });
    let rubric_revision_id = state
        .revisions
        .create(&state.owner, "super-admin-rubric-v1", &rubric)
        .await?;
    let dataset_revision_id = state
        .revisions
        .create(
            &state.owner,
            "super-admin-dataset-v1",
            &ResourceContent::Dataset(cases.clone()),
        )
        .await?;
    let dataset_digest = systemprompt::evaluation::experiments::content_digest(
        &ResourceContent::Dataset(cases.clone()),
    )?;
    let rubric_digest = systemprompt::evaluation::experiments::content_digest(&rubric)?;
    let mut by_skill = BTreeMap::<SkillId, (Vec<EvalRevisionId>, Vec<EvalRevisionId>)>::new();
    for case in &seeded_cases {
        let partitions = by_skill.entry(case.skill_id.clone()).or_default();
        match case.partition {
            Partition::Development => partitions.0.push(case.revision_id.clone()),
            Partition::Holdout => partitions.1.push(case.revision_id.clone()),
        }
    }
    let mut skill_datasets = Vec::new();
    for (skill_id, (development, holdout)) in by_skill {
        let development_dataset_id = state
            .revisions
            .create(
                &state.owner,
                &format!("{skill_id}:development-v2"),
                &ResourceContent::Dataset(development),
            )
            .await?;
        let holdout_dataset_id = state
            .revisions
            .create(
                &state.owner,
                &format!("{skill_id}:holdout-v2"),
                &ResourceContent::Dataset(holdout),
            )
            .await?;
        skill_datasets.push(SeededSkillDataset {
            skill_id,
            development_dataset_id,
            holdout_dataset_id,
        });
    }

    Ok((
        StatusCode::CREATED,
        Json(SeededSuite {
            skill_datasets,
            dataset_revision_id,
            dataset_digest,
            rubric_revision_id,
            rubric_digest,
            case_revision_ids: cases,
            cases: seeded_cases,
        }),
    ))
}
