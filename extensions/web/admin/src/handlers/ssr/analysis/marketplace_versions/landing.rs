//! One marketplace as the Versions landing lists it: its current version,
//! source, size, and the window's figures across every version it has had.

use serde::Serialize;
use systemprompt::identifiers::MarketplaceId;

use super::history::short;
use crate::handlers::ssr::analysis_urls::analysis_evaluation_url;
use crate::handlers::ssr::format::format_cost;
use crate::handlers::ssr::scorecard::{ScorecardView, Scores};
use crate::repositories::analysis::marketplace_versions::MarketplaceListRow;
use crate::repositories::evaluation::version_scores::MarketplaceScoreRollup;

#[derive(Serialize)]
pub(super) struct MarketplaceCard {
    marketplace_id: MarketplaceId,
    name: String,
    href: String,
    source: Option<String>,
    source_short: Option<String>,
    source_hash: Option<String>,
    hash: Option<String>,
    hash_short: Option<String>,
    plugin_count: Option<i32>,
    skill_count: Option<i32>,
    versions: i64,
    first_seen_at: String,
    last_changed_at: String,
    invocations: i64,
    failed_invocations: i64,
    users: i64,
    requests: i64,
    failed: i64,
    cost: String,
    scorecard: ScorecardView,
    evaluation_href: String,
}

pub(super) fn card(
    row: MarketplaceListRow,
    scores: Option<&MarketplaceScoreRollup>,
) -> MarketplaceCard {
    let current = scores.map(|s| s.current);
    let history = scores.map(|s| s.history);
    MarketplaceCard {
        evaluation_href: analysis_evaluation_url(&row.marketplace_id),
        scorecard: ScorecardView::with_prior(
            current.map_or_else(Scores::default, |c| Scores {
                intent: c.intent,
                quality: c.quality,
                speed: c.speed,
                cost: c.cost,
            }),
            history.map_or_else(Scores::default, |h| Scores {
                intent: h.intent,
                quality: h.quality,
                speed: h.speed,
                cost: h.cost,
            }),
            Some(analysis_evaluation_url(&row.marketplace_id)),
            format!(
                "Judge scores for the current version over {} judged sessions; the delta is against every earlier version",
                current.map_or(0, |c| c.scored_sessions)
            ),
        ),
        href: format!(
            "/admin/analysis/versions/{}",
            urlencoding::encode(row.marketplace_id.as_str())
        ),
        name: row
            .name
            .clone()
            .unwrap_or_else(|| row.marketplace_id.as_str().to_owned()),
        marketplace_id: row.marketplace_id,
        source_short: row.source_hash.as_deref().map(short),
        source: row.source,
        source_hash: row.source_hash,
        hash_short: row.content_hash.as_deref().map(short),
        hash: row.content_hash,
        plugin_count: row.plugin_count,
        skill_count: row.skill_count,
        versions: row.versions,
        first_seen_at: row.first_seen_at.to_rfc3339(),
        last_changed_at: row.last_changed_at.to_rfc3339(),
        invocations: row.invocations,
        failed_invocations: row.failed_invocations,
        users: row.users,
        requests: row.requests,
        failed: row.failed,
        cost: if row.cost > 0 {
            format_cost(row.cost)
        } else {
            "—".to_owned()
        },
    }
}
