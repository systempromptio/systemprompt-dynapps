//! The sources core holds back on a previous bundle, from the bundle cache.
//!
//! Under `on_fetch_failure: use_last_good` core resolves a source that cannot
//! be fetched or verified to the bundle already cached for it, composes, and
//! records the hold. The composition loads and its provenance is `fetched`, so
//! that record is the only place a held-back source shows.

use serde::Serialize;
use systemprompt::loader::bundle::{BundleCache, SourceRefusal};
use systemprompt::manifest::profile::Profile;

/// A pinned source held back on its previous bundle; `detail` is the sentence
/// shown on the page, on the badge and in the activity row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RefusedSource {
    pub name: String,
    pub serving_version: String,
    pub serving_digest: String,
    pub since: String,
    pub detail: String,
}

impl RefusedSource {
    #[must_use]
    pub fn from_refusal(refusal: &SourceRefusal) -> Self {
        Self {
            name: refusal.source.clone(),
            serving_version: refusal.serving_version.clone(),
            serving_digest: refusal.serving_digest.clone(),
            since: refusal.recorded_at.to_rfc3339(),
            detail: format!(
                "{} was refused and still serves {}: {}",
                refusal.source, refusal.serving_version, refusal.reason
            ),
        }
    }
}

/// The pinned sources held back, or why that is unknown: `unreadable` is set
/// when core's record exists and cannot be read, and `held` is then empty
/// without meaning that no source was refused.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct RefusalsView {
    pub held: Vec<RefusedSource>,
    pub unreadable: Option<String>,
}

impl RefusalsView {
    #[must_use]
    pub fn read(cache: &BundleCache, profile: &Profile) -> Self {
        let pinned = &profile.services.sources;
        if pinned.is_empty() {
            return Self::default();
        }
        match cache.read_refusals() {
            Ok(refusals) => Self {
                held: refusals
                    .iter()
                    .filter(|refusal| pinned.iter().any(|s| s.name == refusal.source))
                    .map(RefusedSource::from_refusal)
                    .collect(),
                unreadable: None,
            },
            Err(error) => Self {
                held: Vec::new(),
                unreadable: Some(format!(
                    "core's record of refused sources could not be read, so whether a source is \
                     held back is unknown: {error}"
                )),
            },
        }
    }
}
