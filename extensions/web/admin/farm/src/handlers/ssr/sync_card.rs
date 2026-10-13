//! Where a farm stands against the bundle that ships its marketplace: the
//! commit the last farm sync recorded, compared with the source commit the
//! pinned bundle was built from. The overview card and the farm page both
//! read their sync badge from [`sync_state`], so the two never disagree.

use chrono::{DateTime, Utc};
use serde::Serialize;
use systemprompt::config::ProfileBootstrap;
use systemprompt::identifiers::MarketplaceId;
use systemprompt::loader::bundle::{BundleCache, cache_root};
use systemprompt_web_admin_core::error::AdminError;
use systemprompt_web_admin_core::repositories::sync::sources::build_sources;
use systemprompt_web_admin_core::ssr::format::{local_time, relative_time};

use super::shared::BadgeView;

const SHORT_SHA: usize = 7;

/// What the farm row records about its last sync.
#[derive(Debug, Clone, Copy, Default)]
pub struct FarmSyncFacts<'a> {
    pub synced_commit_sha: Option<&'a str>,
    pub synced_at: Option<DateTime<Utc>>,
    pub synced_by: Option<&'a str>,
    pub sync_error: Option<&'a str>,
    pub source_repo: &'a str,
    pub source_ref: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BundleBadge {
    pub tone: &'static str,
    pub text: &'static str,
    pub detail: String,
}

/// One pinned bundle as the sync card needs it: what it owns, what is active
/// in the cache, and the source commit its manifest names.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct BundleFacts {
    pub name: String,
    pub owner_key: String,
    pub marketplaces: Vec<String>,
    pub active_digest: Option<String>,
    pub version: Option<String>,
    pub fetched_at: Option<String>,
    pub source_commit: Option<String>,
    pub badges: Vec<BundleBadge>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncState {
    NeverSynced,
    Failed,
    Behind,
    InStep,
    Synced,
}

impl SyncState {
    #[must_use]
    pub const fn tone(self) -> &'static str {
        match self {
            Self::NeverSynced => "muted",
            Self::Failed => "err",
            Self::Behind => "warn",
            Self::InStep | Self::Synced => "ok",
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::NeverSynced => "Never synced",
            Self::Failed => "Sync failed",
            Self::Behind => "Behind bundle",
            Self::InStep => "In step",
            Self::Synced => "Synced",
        }
    }

    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::NeverSynced => "never",
            Self::Failed => "failed",
            Self::Behind => "behind",
            Self::InStep => "in-step",
            Self::Synced => "synced",
        }
    }
}

#[must_use]
pub fn commits_match(a: &str, b: &str) -> bool {
    let (a, b) = (a.trim().to_ascii_lowercase(), b.trim().to_ascii_lowercase());
    let (short, long) = if a.len() <= b.len() { (a, b) } else { (b, a) };
    short.len() >= SHORT_SHA && long.starts_with(&short)
}

#[must_use]
pub fn sync_state(farm: &FarmSyncFacts<'_>, bundle: Option<&BundleFacts>) -> SyncState {
    if farm.sync_error.is_some_and(|e| !e.trim().is_empty()) {
        return SyncState::Failed;
    }
    let Some(synced) = farm.synced_commit_sha.filter(|s| !s.trim().is_empty()) else {
        return SyncState::NeverSynced;
    };
    match bundle.and_then(|b| b.source_commit.as_deref()) {
        Some(commit) if commits_match(synced, commit) => SyncState::InStep,
        Some(_) => SyncState::Behind,
        None => SyncState::Synced,
    }
}

#[must_use]
pub(crate) fn sync_badge(state: SyncState) -> BadgeView {
    BadgeView::new(state.tone(), state.label(), state.key())
}

#[must_use]
pub fn short_sha(sha: &str) -> String {
    sha.chars().take(SHORT_SHA).collect()
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BundleCardView {
    pub(crate) name: String,
    pub(crate) owner_key: String,
    pub(crate) digest_short: String,
    pub(crate) version: String,
    pub(crate) fetched_at: String,
    pub(crate) commit: String,
    pub(crate) commit_short: String,
    pub(crate) badges: Vec<BundleBadge>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SyncCardView {
    pub(crate) state: SyncState,
    pub(crate) badge: BadgeView,
    pub(crate) is_behind: bool,
    pub(crate) message: String,
    pub(crate) source_repo: String,
    pub(crate) source_ref: String,
    pub(crate) synced_sha: String,
    pub(crate) synced_sha_short: String,
    pub(crate) synced_at_local: String,
    pub(crate) synced_at_relative: String,
    pub(crate) synced_by: String,
    pub(crate) sync_error: String,
    pub(crate) bundle: Option<BundleCardView>,
}

fn message(state: SyncState, bundle: Option<&BundleFacts>) -> String {
    let name = bundle.map_or("", |b| b.name.as_str());
    match state {
        SyncState::NeverSynced => "This farm has not been synced yet.".to_owned(),
        SyncState::Failed => "The last sync failed; the agents shown are from the sync before it."
            .to_owned(),
        SyncState::Behind => format!(
            "Bundle {name} was built from a different commit than the farm last synced. Sync the farm to bring its agents in step."
        ),
        SyncState::InStep => format!("The farm and bundle {name} name the same commit."),
        SyncState::Synced => {
            "No pinned bundle names a source commit for this marketplace, so there is nothing to compare against."
                .to_owned()
        },
    }
}

fn bundle_card(bundle: &BundleFacts) -> BundleCardView {
    let digest = bundle.active_digest.as_deref().unwrap_or_default();
    let commit = bundle.source_commit.clone().unwrap_or_default();
    BundleCardView {
        name: bundle.name.clone(),
        owner_key: bundle.owner_key.clone(),
        digest_short: digest
            .trim_start_matches("sha256:")
            .chars()
            .take(12)
            .collect(),
        version: bundle.version.clone().unwrap_or_default(),
        fetched_at: bundle.fetched_at.clone().unwrap_or_default(),
        commit_short: short_sha(&commit),
        commit,
        badges: bundle.badges.clone(),
    }
}

#[must_use]
pub(crate) fn sync_card_view(
    farm: &FarmSyncFacts<'_>,
    bundle: Option<&BundleFacts>,
) -> SyncCardView {
    let state = sync_state(farm, bundle);
    let sha = farm.synced_commit_sha.unwrap_or_default();
    SyncCardView {
        state,
        badge: sync_badge(state),
        is_behind: state == SyncState::Behind,
        message: message(state, bundle),
        source_repo: farm.source_repo.to_owned(),
        source_ref: farm.source_ref.to_owned(),
        synced_sha: sha.to_owned(),
        synced_sha_short: short_sha(sha),
        synced_at_local: farm.synced_at.map(local_time).unwrap_or_default(),
        synced_at_relative: farm.synced_at.map(relative_time).unwrap_or_default(),
        synced_by: farm.synced_by.unwrap_or_default().to_owned(),
        sync_error: farm.sync_error.unwrap_or_default().to_owned(),
        bundle: bundle.map(bundle_card),
    }
}

#[must_use]
pub fn bundle_for<'a>(
    bundles: &'a [BundleFacts],
    marketplace_id: &MarketplaceId,
) -> Option<&'a BundleFacts> {
    let marketplace_id = marketplace_id.as_str();
    bundles
        .iter()
        .find(|b| b.marketplaces.iter().any(|m| m == marketplace_id))
        .or_else(|| bundles.iter().find(|b| b.name == marketplace_id))
}

pub(crate) fn load_bundles() -> Result<Vec<BundleFacts>, AdminError> {
    let sources = build_sources()?;
    let profile = ProfileBootstrap::get()?;
    let cache = BundleCache::new(cache_root(profile));
    let state = cache.read_state();
    Ok(sources
        .bundles
        .into_iter()
        .map(|b| {
            let source_commit = state
                .sources
                .get(&b.name)
                .and_then(|s| cache.read_manifest(&b.name, &s.content_hash).ok())
                .and_then(|m| m.manifest.source.commit);
            BundleFacts {
                badges: b
                    .badges
                    .iter()
                    .map(|s| BundleBadge {
                        tone: s.tone,
                        text: s.text,
                        detail: s.detail.clone(),
                    })
                    .collect(),
                marketplaces: b.owns.marketplaces,
                name: b.name,
                owner_key: b.owner_key,
                active_digest: b.active_digest,
                version: b.version,
                fetched_at: b.fetched_at,
                source_commit,
            }
        })
        .collect())
}
