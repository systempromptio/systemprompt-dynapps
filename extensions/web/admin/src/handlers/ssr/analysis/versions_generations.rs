//! The provenance half of the Versions page: which sources the instance is
//! composing, which generations they produced, and what moved between two.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::error::AdminError;
use crate::repositories::analysis::source_generations::{self, SourceGeneration};
use crate::repositories::sync::sources::{SourcesView, build_sources};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

// Why: `first_generation` is the earliest observation carrying this source's
// content hash, which is what lets a reader jump from a kit to the inventory it
// produced; it is not a property of the source itself.
#[derive(Debug, Clone, Serialize)]
pub(super) struct SourceCard {
    pub(super) name: String,
    pub(super) owner_key: String,
    pub(super) content_hash: Option<String>,
    pub(super) content_hash_short: Option<String>,
    pub(super) pinned_digest: Option<String>,
    pub(super) active_digest: Option<String>,
    pub(super) version: Option<String>,
    pub(super) first_generation: Option<i64>,
    pub(super) last_changed: Option<String>,
    pub(super) highlighted: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct GenerationRow {
    pub(super) generation: i64,
    pub(super) observed_at: String,
    pub(super) entries: i64,
    pub(super) composed_hash: Option<String>,
    pub(super) composed_hash_short: Option<String>,
    pub(super) added: i64,
    pub(super) removed: i64,
    pub(super) changed: i64,
    pub(super) publications: i64,
    pub(super) compare_href: Option<String>,
    pub(super) highlighted: bool,
}

pub(super) fn short(hash: &str) -> String {
    hash.rsplit(':')
        .next()
        .unwrap_or(hash)
        .chars()
        .take(12)
        .collect()
}

pub(super) async fn timeline(
    pool: &PgPool,
    owner: &UserId,
    hash: Option<&str>,
) -> Result<(Vec<GenerationRow>, Vec<SourceCard>), AdminError> {
    let generations = source_generations::list_source_generations(pool, owner, 200).await?;
    let cards = source_cards(&build_sources()?, &generations, hash);
    // Why: rows are built over the whole timeline so each Compare link names the
    // generation that actually precedes it, and only then narrowed to the hash —
    // filtering first would compare against the previous *match*.
    let mut rows = generation_rows(&generations, hash);
    if hash.is_some() {
        rows.retain(|row| row.highlighted);
    }
    Ok((rows, cards))
}

fn generation_rows(generations: &[SourceGeneration], hash: Option<&str>) -> Vec<GenerationRow> {
    generations
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let previous = generations.get(index + 1).map(|older| older.generation);
            GenerationRow {
                generation: row.generation,
                observed_at: row.observed_at.to_rfc3339(),
                entries: row.entries,
                composed_hash_short: row.sources.composed_hash.as_deref().map(short),
                composed_hash: row.sources.composed_hash.clone(),
                added: row.added,
                removed: row.removed,
                changed: row.changed,
                publications: row.publications,
                compare_href: previous.map(|older| {
                    format!(
                        "/admin/analysis/versions/generations/{older}/compare/{}",
                        row.generation
                    )
                }),
                highlighted: hash.is_some_and(|wanted| carries(row, wanted)),
            }
        })
        .collect()
}

// Why: a hash reaches this page from a publication row, and the reader wants
// every generation that carried it — the composed tree, the base tree or any
// one bundle — not only the composition.
fn carries(row: &SourceGeneration, hash: &str) -> bool {
    row.sources.composed_hash.as_deref() == Some(hash)
        || row.sources.base_tree_hash.as_deref() == Some(hash)
        || row
            .sources
            .bundles
            .iter()
            .any(|bundle| bundle.content_hash.as_deref() == Some(hash))
}

fn source_cards(
    sources: &SourcesView,
    generations: &[SourceGeneration],
    hash: Option<&str>,
) -> Vec<SourceCard> {
    let mut first: BTreeMap<String, (i64, String)> = BTreeMap::new();
    for row in generations.iter().rev() {
        if let Some(base) = row.sources.base_tree_hash.clone() {
            first
                .entry(base)
                .or_insert_with(|| (row.generation, row.observed_at.to_rfc3339()));
        }
        for bundle in &row.sources.bundles {
            if let Some(content) = bundle.content_hash.clone() {
                first
                    .entry(content)
                    .or_insert_with(|| (row.generation, row.observed_at.to_rfc3339()));
            }
        }
    }
    let base = SourceCard::identity(
        "base".to_owned(),
        "base".to_owned(),
        sources.base.tree_hash.clone(),
        Some(sources.base.version.to_owned()),
    );
    std::iter::once(base)
        .chain(sources.bundles.iter().map(|bundle| {
            SourceCard::identity(
                bundle.name.clone(),
                bundle.owner_key.clone(),
                bundle.content_hash.clone(),
                bundle.version.clone(),
            )
            .pinned(bundle.pinned_digest.clone(), bundle.active_digest.clone())
        }))
        .map(|card| card.observed(&first, hash))
        .collect()
}

impl SourceCard {
    fn identity(
        name: String,
        owner_key: String,
        content_hash: Option<String>,
        version: Option<String>,
    ) -> Self {
        Self {
            content_hash_short: content_hash.as_deref().map(short),
            name,
            owner_key,
            content_hash,
            pinned_digest: None,
            active_digest: None,
            version,
            first_generation: None,
            last_changed: None,
            highlighted: false,
        }
    }

    fn pinned(mut self, pinned_digest: Option<String>, active_digest: Option<String>) -> Self {
        self.pinned_digest = pinned_digest;
        self.active_digest = active_digest;
        self
    }

    fn observed(mut self, first: &BTreeMap<String, (i64, String)>, hash: Option<&str>) -> Self {
        let seen = self
            .content_hash
            .as_ref()
            .and_then(|value| first.get(value));
        self.first_generation = seen.map(|(generation, _)| *generation);
        self.last_changed = seen.map(|(_, at)| at.clone());
        self.highlighted = hash.is_some() && hash == self.content_hash.as_deref();
        self
    }
}
