//! Marketplace adoption as the Skills page draws it: entitled → installed →
//! active, by host, with the window's activity and spend.

use super::rows::marketplace_name;
use super::views::AdoptionView;
use crate::handlers::ssr::analysis::tone::{completion_tone, percent, score_display};
use crate::handlers::ssr::analysis_urls::analysis_version_url;
use crate::handlers::ssr::format::format_cost;
use crate::repositories::analysis::inventory_index::{InventoryIndex, MarketplaceAudience};
use crate::repositories::analysis::skills::MarketplaceAdoptionRow;

pub(super) fn adoption_view(
    a: &MarketplaceAdoptionRow,
    audience: &MarketplaceAudience,
    index: &InventoryIndex,
) -> AdoptionView {
    let entitled = audience.users_reaching_marketplace(a.marketplace_id.as_str());
    let entitled_i = i64::try_from(entitled).unwrap_or(0);
    AdoptionView {
        marketplace_name: marketplace_name(index, a.marketplace_id.as_str()),
        versions_href: analysis_version_url(&a.marketplace_id, None),
        marketplace: a.marketplace_id.as_str().to_owned(),
        entitled,
        installed: a.installed,
        active: a.active_users,
        install_rate: percent(a.installed, entitled_i),
        install_rate_pct: if entitled_i > 0 {
            (a.installed * 100 / entitled_i).min(100)
        } else {
            0
        },
        activation_rate: percent(a.active_users, a.installed),
        activation_rate_pct: if a.installed > 0 {
            (a.active_users * 100 / a.installed).min(100)
        } else {
            0
        },
        installed_claude_code: a.installed_claude_code,
        installed_opencode: a.installed_opencode,
        installed_other: a.installed_other,
        skills: a.skills,
        skills_used: a.skills_used,
        plugins: a.plugins,
        invocations: a.invocations,
        conversations: a.conversations,
        cost_display: format_cost(a.cost_microdollars),
        completion_display: score_display(a.completion_avg),
        completion_tone: completion_tone(a.completion_avg),
        version_short: a
            .current_hash
            .as_deref()
            .map_or_else(|| "—".to_owned(), |h| h[..h.len().min(12)].to_owned()),
        versions: a.versions,
        last_install: a
            .last_install_at
            .map_or_else(|| "never".to_owned(), |t| t.format("%b %-d").to_string()),
    }
}
