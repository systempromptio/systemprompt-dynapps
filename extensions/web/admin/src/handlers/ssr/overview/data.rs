//! Loading the overview's widgets.
//!
//! Every widget keeps its own `Result` all the way to the view, which renders
//! the failed ones as a notice in place and draws the rest. The landing page
//! is where an operator finds out something is wrong; a page that blanks
//! because one aggregate could not be read is the one failure mode it must not
//! have.

use std::sync::Arc;

use sqlx::PgPool;

use crate::repositories::analytics::site::SiteScope;
use crate::repositories::analytics::site::models::{ModelStatsRow, list_model_stats};
use crate::repositories::overview::dimension_series::{DimensionBucket, OverviewDimension};
use crate::repositories::overview::kpis::OverviewKpis;
use crate::repositories::overview::queues::UsageAnomalyRow;
use crate::repositories::overview::scopes::ScopeCostRow;
use crate::repositories::overview::series::OverviewSeriesPoint;
use crate::repositories::overview::{dimension_series, kpis, queues, scopes, series};
use systemprompt_web_admin_core::repositories::scope::{ScopeKind, SubjectScope};

use super::view::OverviewRange;

pub(super) const TOP_SCOPES: i64 = 5;

pub(super) const TOP_MODELS: usize = 5;

const TOP_ANOMALIES: i64 = 5;

type Loaded<T> = Result<T, sqlx::Error>;

pub(super) struct OverviewData {
    pub kpis: Loaded<OverviewKpis>,
    pub series: Loaded<Vec<OverviewSeriesPoint>>,
    pub model_series: Loaded<Vec<DimensionBucket>>,
    pub client_series: Loaded<Vec<DimensionBucket>>,
    pub pending_approvals: Loaded<i64>,
    pub anomalies: Loaded<Vec<UsageAnomalyRow>>,
    pub projects: Loaded<Vec<ScopeCostRow>>,
    pub groups: Loaded<Vec<ScopeCostRow>>,
    pub models: Loaded<Vec<ModelStatsRow>>,
}

pub(super) async fn load_overview(pool: &Arc<PgPool>, range: OverviewRange) -> OverviewData {
    let window = range.time_range();
    let scope = SiteScope::new(SubjectScope::All);
    let (
        kpis,
        series,
        model_series,
        client_series,
        pending_approvals,
        anomalies,
        projects,
        groups,
        models,
    ) = tokio::join!(
        kpis::get_overview_kpis(pool, window),
        series::list_overview_series(pool, window),
        dimension_series::list_dimension_series(pool, window, OverviewDimension::Model),
        dimension_series::list_dimension_series(pool, window, OverviewDimension::Client),
        queues::count_pending_approvals(pool),
        queues::list_open_anomalies(pool, TOP_ANOMALIES),
        scopes::list_top_scopes_by_cost(pool, ScopeKind::Project, window, TOP_SCOPES),
        scopes::list_top_scopes_by_cost(pool, ScopeKind::Group, window, TOP_SCOPES),
        list_model_stats(pool, window, &scope),
    );

    OverviewData {
        kpis,
        series,
        model_series,
        client_series,
        pending_approvals,
        anomalies,
        projects,
        groups,
        models,
    }
}
