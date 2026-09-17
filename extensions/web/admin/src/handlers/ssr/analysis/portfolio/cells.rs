//! The cells a Skills row derives beyond its snapshot: the version chip of
//! the marketplace it sits under, the live figure keyed the way facts key a
//! skill, and a p95 that is measured when it can be and a bound when it must.

use serde::Serialize;
use systemprompt::identifiers::MarketplaceId;

use crate::handlers::ssr::format::format_duration_ms;
use crate::repositories::analysis::marketplace_versions::SkillLiveRow;
use crate::repositories::evaluation::skill_scores::SkillScoreFigures;
use crate::repositories::sync::marketplace_hash::skill_key;
use crate::repositories::sync::marketplace_versions_db::MarketplaceVersionRow;

/// The marketplace version a row is served from: the short hash the table
/// shows and the history page it opens.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RowVersion {
    pub marketplace_id: MarketplaceId,
    pub content_hash: String,
    pub short_hash: String,
    pub href: String,
}

impl RowVersion {
    #[must_use]
    pub fn of(version: &MarketplaceVersionRow) -> Self {
        Self {
            marketplace_id: version.marketplace_id.clone(),
            content_hash: version.content_hash.clone(),
            short_hash: short_hash(&version.content_hash),
            href: version_href(&version.marketplace_id, &version.content_hash),
        }
    }
}

#[must_use]
pub fn short_hash(hash: &str) -> String {
    hash.chars().take(12).collect()
}

#[must_use]
pub fn version_href(marketplace_id: &MarketplaceId, content_hash: &str) -> String {
    format!(
        "/admin/analysis/versions/{}#v-{}",
        urlencoding::encode(marketplace_id.as_str()),
        short_hash(content_hash)
    )
}

// Why: facts key a skill as `<plugin>:<skill-with-dashes>` while the
// inventory carries the underscored config id; the lookup bridges the two.
#[must_use]
pub fn live_for<'a>(
    live: &'a [SkillLiveRow],
    plugin: Option<&str>,
    resource_key: &str,
) -> Option<&'a SkillLiveRow> {
    let plugin = plugin?;
    let key = skill_key(plugin, resource_key);
    live.iter()
        .find(|row| row.plugin_id.as_str() == plugin && row.skill == key)
}

// Why: the judge attribution keys a skill exactly as the facts do.
#[must_use]
pub fn scores_for<'a>(
    scores: &'a [SkillScoreFigures],
    plugin: Option<&str>,
    resource_key: &str,
) -> Option<&'a SkillScoreFigures> {
    let plugin = plugin?;
    let key = skill_key(plugin, resource_key);
    scores
        .iter()
        .find(|row| row.plugin_id.as_str() == plugin && row.skill == key)
}

// Why: the snapshot histogram only knows power-of-two buckets, so its
// percentile is an upper bound and reads `≤`; a real percentile over request
// latencies is preferred whenever the window has any.
#[must_use]
pub fn p95_cell(live: Option<&SkillLiveRow>, snapshot_bound_micros: Option<u64>) -> (String, bool) {
    if let Some(micros) = live
        .filter(|l| l.latency_measured > 0)
        .and_then(|l| l.p95_micros)
    {
        return (format_duration_ms(micros / 1000), true);
    }
    (
        snapshot_bound_micros.map_or_else(
            || "—".to_owned(),
            |micros| {
                format!(
                    "≤ {}",
                    format_duration_ms(i64::try_from(micros / 1000).unwrap_or(i64::MAX))
                )
            },
        ),
        false,
    )
}
