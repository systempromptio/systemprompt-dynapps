//! Two inventory generations side by side: the skills that entered or left,
//! the revisions whose bytes moved, and the source hashes that explain both.

use std::collections::BTreeMap;

use serde::Serialize;

use super::generations::short;
use crate::error::{AdminError, AdminHtmlResult};
use crate::repositories::analysis::source_generations::{self, GenerationEntry, SourceGeneration};
use crate::routes::managed_state::ManagedState;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::Extension;
use axum::extract::{Path, State};
use axum::response::Response;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Serialize)]
struct SkillDiff {
    resource_key: String,
    kind: String,
    before: Option<String>,
    after: Option<String>,
}

#[derive(Serialize)]
struct SourceDiff {
    name: String,
    before: Option<String>,
    after: Option<String>,
    moved: bool,
}

#[derive(Serialize)]
struct ComparePage {
    page: &'static str,
    title: &'static str,
    baseline: SourceGeneration,
    candidate: SourceGeneration,
    added: Vec<GenerationEntry>,
    removed: Vec<GenerationEntry>,
    revisions: Vec<SkillDiff>,
    sources: Vec<SourceDiff>,
    added_count: usize,
    removed_count: usize,
    revisions_count: usize,
}

#[expect(
    clippy::too_many_arguments,
    reason = "axum extractor list; the router decides the arity, not this signature"
)]
pub(crate) async fn compare_page(
    Extension(user): Extension<UserContext>,
    Extension(marketplace): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    Extension(state): Extension<Arc<ManagedState>>,
    State(pool): State<Arc<PgPool>>,
    Path((baseline, candidate)): Path<(String, String)>,
) -> AdminHtmlResult<Response> {
    super::require_console(&user)?;
    let owner = &state.owner;
    let missing = || AdminError::NotFound("Unknown inventory generation".to_owned());
    let (baseline, candidate): (i64, i64) = match (baseline.parse(), candidate.parse()) {
        (Ok(baseline), Ok(candidate)) => (baseline, candidate),
        _ => return Err(missing().into()),
    };
    let baseline = source_generations::find_source_generation(&pool, owner, baseline)
        .await?
        .ok_or_else(missing)?;
    let candidate = source_generations::find_source_generation(&pool, owner, candidate)
        .await?
        .ok_or_else(missing)?;
    let before = index(
        source_generations::list_generation_entries(&pool, owner, baseline.observed_at).await?,
    );
    let after = index(
        source_generations::list_generation_entries(&pool, owner, candidate.observed_at).await?,
    );
    let wanted: Vec<String> = before
        .values()
        .chain(after.values())
        .filter_map(|entry| entry.latest_revision_id.clone())
        .collect();
    let digests = source_generations::list_revision_digests(&pool, owner, &wanted).await?;
    let added = present(&after, &before);
    let removed = present(&before, &after);
    let revisions = revision_diffs(&before, &after, &digests);
    let data = ComparePage {
        page: "analysis-versions",
        title: "Generation comparison",
        added_count: added.len(),
        removed_count: removed.len(),
        revisions_count: revisions.len(),
        added,
        removed,
        revisions,
        sources: source_diffs(&baseline, &candidate),
        baseline,
        candidate,
    };
    Ok(crate::handlers::ssr::render_typed_page(
        &engine,
        "analysis-generation-compare",
        &data,
        &user,
        &marketplace,
    ))
}

fn index(entries: Vec<GenerationEntry>) -> BTreeMap<String, GenerationEntry> {
    entries
        .into_iter()
        .filter(|entry| entry.availability != "withdrawn")
        .map(|entry| (entry.entry_id.clone(), entry))
        .collect()
}

fn present(
    side: &BTreeMap<String, GenerationEntry>,
    other: &BTreeMap<String, GenerationEntry>,
) -> Vec<GenerationEntry> {
    side.iter()
        .filter(|(id, _)| !other.contains_key(*id))
        .map(|(_, entry)| entry.clone())
        .collect()
}

fn revision_diffs(
    before: &BTreeMap<String, GenerationEntry>,
    after: &BTreeMap<String, GenerationEntry>,
    digests: &BTreeMap<String, String>,
) -> Vec<SkillDiff> {
    let digest = |entry: &GenerationEntry| {
        entry
            .latest_revision_id
            .as_ref()
            .and_then(|id| digests.get(id))
            .map(|value| short(value))
    };
    after
        .iter()
        .filter_map(|(id, entry)| {
            let previous = before.get(id)?;
            (previous.latest_revision_id != entry.latest_revision_id).then(|| SkillDiff {
                resource_key: entry.resource_key.clone(),
                kind: entry.kind.clone(),
                before: digest(previous),
                after: digest(entry),
            })
        })
        .collect()
}

fn source_diffs(baseline: &SourceGeneration, candidate: &SourceGeneration) -> Vec<SourceDiff> {
    let mut names: Vec<String> = vec!["base".to_owned()];
    names.extend(candidate.sources.bundles.iter().map(|b| b.name.clone()));
    let extra: Vec<String> = baseline
        .sources
        .bundles
        .iter()
        .map(|b| b.name.clone())
        .filter(|name| !names.contains(name))
        .collect();
    names.extend(extra);
    names
        .into_iter()
        .map(|name| {
            let hash_of = |row: &SourceGeneration| {
                if name == "base" {
                    row.sources.base_tree_hash.clone()
                } else {
                    row.sources
                        .bundles
                        .iter()
                        .find(|b| b.name == name)
                        .and_then(|b| b.content_hash.clone())
                }
            };
            let before = hash_of(baseline);
            let after = hash_of(candidate);
            SourceDiff {
                moved: before != after,
                before: before.as_deref().map(short),
                after: after.as_deref().map(short),
                name,
            }
        })
        .collect()
}
