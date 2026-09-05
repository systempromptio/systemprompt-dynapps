//! Loading the overview's widgets.
//!
//! Every widget keeps its own `Result` all the way to the view, which renders
//! the failed ones as a notice in place and draws the rest. The landing page
//! is where an operator finds out something is wrong; a page that blanks
//! because one aggregate could not be read is the one failure mode it must not
//! have.

use std::path::Path;
use std::sync::Arc;

use sqlx::PgPool;

use crate::repositories::mcp::mcp_servers;
use crate::repositories::overview::kpis::OverviewKpis;
use crate::repositories::overview::liveness::McpHeartbeatRow;
use crate::repositories::overview::queues::UsageAnomalyRow;
use crate::repositories::overview::scopes::ScopeCostRow;
use crate::repositories::overview::{kpis, liveness, queues, scopes, series};
use crate::repositories::scope::ScopeKind;
use crate::types::McpServerDetail;

use super::view::OverviewRange;

// Why: how many containers each leaderboard carries. Five is what fits beside
// the other board without either one scrolling.
pub(super) const TOP_SCOPES: i64 = 5;

// Why: how many open anomalies the page lists before pointing at the page that
// owns them.
const TOP_ANOMALIES: i64 = 5;

type Loaded<T> = Result<T, sqlx::Error>;

pub(super) struct OverviewData {
    pub kpis: Loaded<OverviewKpis>,
    pub buckets: Loaded<Vec<i64>>,
    pub heartbeats: Loaded<Vec<McpHeartbeatRow>>,
    pub configured: Vec<McpServerDetail>,
    pub pending_approvals: Loaded<i64>,
    pub anomalies: Loaded<Vec<UsageAnomalyRow>>,
    pub projects: Loaded<Vec<ScopeCostRow>>,
    pub groups: Loaded<Vec<ScopeCostRow>>,
}

pub(super) async fn load_overview(
    pool: &Arc<PgPool>,
    range: OverviewRange,
    services_path: Option<&Path>,
) -> OverviewData {
    let window = range.time_range();
    let (kpis, buckets, heartbeats, pending_approvals, anomalies, projects, groups) = tokio::join!(
        kpis::get_overview_kpis(pool, window),
        series::list_request_buckets(pool, window),
        liveness::list_mcp_server_liveness(pool),
        queues::count_pending_approvals(pool),
        queues::list_open_anomalies(pool, TOP_ANOMALIES),
        scopes::list_top_scopes_by_cost(pool, ScopeKind::Project, window, TOP_SCOPES),
        scopes::list_top_scopes_by_cost(pool, ScopeKind::Group, window, TOP_SCOPES),
    );

    OverviewData {
        kpis,
        buckets,
        heartbeats,
        configured: configured_servers(services_path),
        pending_approvals,
        anomalies,
        projects,
        groups,
    }
}

// Why: the strip lists what `services/mcp/*.yaml` declares, not what has a
// heartbeat row. A server that was configured and has never been reached is
// the finding the page exists to surface, and it has no row to be found by.
fn configured_servers(services_path: Option<&Path>) -> Vec<McpServerDetail> {
    services_path.map_or_else(Vec::new, |path| {
        mcp_servers::list_mcp_servers(path).unwrap_or_else(|e| {
            tracing::warn!(error = %e, "overview: MCP catalog listing failed");
            Vec::new()
        })
    })
}
