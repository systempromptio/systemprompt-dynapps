//! Data-collection orchestration for the Inference Requests page.
//!
//! Resolves the effective time range (honouring an explicit user pick, else
//! auto-widening 24h -> 7d -> 30d until a window has rows), then runs only the
//! queries the active tab actually renders. Every failed `Result` is handed to
//! the page's fault collector, so a single failed query degrades its own panel
//! and is named on the page rather than taking the whole page down.

use std::sync::Arc;

use sqlx::PgPool;

use crate::repositories::analytics::request_stats::{
    LatencyBucket, RequestStats, TimeBucket, get_request_stats, list_latency_histogram,
    list_request_timeseries,
};
use crate::repositories::analytics::requests::{
    BreakdownRow, FacetValue, RequestFilter, RequestKpis, RequestPage, RequestRow, RequestSortSpec,
    find_request_billing_split, get_request_kpis, list_request_facets, list_requests_by_model,
    list_requests_by_provider, list_requests_by_status, list_requests_paged,
};
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::util::time_range::{
    TimeRange, TimeRangePreset, TimeRangeQuery, count_requests_in_range, parse_time_range,
    preset_to_range,
};

use super::RequestsQuery;
use super::context::RequestsTab;

pub(super) async fn resolve_range(
    pool: &PgPool,
    query: &RequestsQuery,
    faults: &mut Faults,
) -> (TimeRange, Option<&'static str>) {
    let user_picked_range = query.preset.is_some() || (query.from.is_some() && query.to.is_some());
    let initial_range = parse_time_range(&TimeRangeQuery {
        from: query.from.clone(),
        to: query.to.clone(),
        preset: query.preset.clone(),
    });

    if user_picked_range {
        return (initial_range, None);
    }

    let mut chosen = initial_range;
    let mut widened: Option<&'static str> = None;
    for (label, preset) in [
        ("24h", TimeRangePreset::Hours24),
        ("7d", TimeRangePreset::Days7),
        ("30d", TimeRangePreset::Days30),
    ] {
        let candidate = preset_to_range(preset);
        match count_requests_in_range(pool, candidate).await {
            Ok(count) if count > 0 => {
                chosen = candidate;
                widened = if label == "24h" { None } else { Some(label) };
                break;
            },
            Ok(_) => {},
            Err(e) => {
                faults.record("request count for the time range", &e);
                break;
            },
        }
    }
    (chosen, widened)
}

#[derive(Default)]
pub(super) struct RequestsData {
    pub rows: Vec<RequestRow>,
    pub total_count: i64,
    pub rows_unavailable: bool,
    pub kpis: RequestKpis,
    pub facets: Vec<FacetValue>,
    pub stats: RequestStats,
    pub hist: Vec<LatencyBucket>,
    pub series: Vec<TimeBucket>,
    pub breakdown: Vec<BreakdownRow>,
}

#[derive(Clone, Copy)]
pub(super) struct RequestsPageQuery<'a> {
    pub tab: RequestsTab,
    pub filter: &'a RequestFilter,
    pub range: TimeRange,
    pub sort: RequestSortSpec,
    pub page_size: i64,
    pub offset: i64,
}

pub(super) async fn load_requests_data(
    pool: &Arc<PgPool>,
    query: RequestsPageQuery<'_>,
    faults: &mut Faults,
) -> RequestsData {
    let page = RequestPage {
        sort: query.sort,
        limit: query.page_size,
        offset: query.offset,
    };
    let mut data = load_every_tab(pool, query.filter, query.range, page, faults).await;
    load_active_tab_only(pool, query, &mut data, faults).await;
    data
}

async fn load_every_tab(
    pool: &PgPool,
    filter: &RequestFilter,
    range: TimeRange,
    page: RequestPage,
    faults: &mut Faults,
) -> RequestsData {
    let (paged, stats_res, kpis_res, facets_res, billing_res) = tokio::join!(
        list_requests_paged(pool, filter, range, page),
        get_request_stats(pool, range, &filter.scope),
        get_request_kpis(pool, filter, range),
        list_request_facets(pool, range, &filter.scope),
        find_request_billing_split(pool, filter, range),
    );

    let rows_unavailable = paged.is_err();
    let (rows, total_count) = faults.take("request rows", paged);
    let stats = faults.take("request stats", stats_res);
    let mut kpis = faults.take("request kpis", kpis_res);
    if let Some(split) = faults.take("request billing split", billing_res) {
        kpis.cost_microdollars -= split.user_key_cost_microdollars;
        kpis.user_key_cost_microdollars = split.user_key_cost_microdollars;
        kpis.user_key_requests = split.user_key_requests;
    }
    let facets = faults.take("request facets", facets_res);

    RequestsData {
        rows,
        total_count,
        rows_unavailable,
        kpis,
        facets,
        stats,
        ..RequestsData::default()
    }
}

async fn load_active_tab_only(
    pool: &PgPool,
    query: RequestsPageQuery<'_>,
    data: &mut RequestsData,
    faults: &mut Faults,
) {
    let RequestsPageQuery {
        tab, filter, range, ..
    } = query;
    match tab {
        RequestsTab::Overview => {
            let (hist_res, series_res) = tokio::join!(
                list_latency_histogram(pool, range, &filter.scope),
                list_request_timeseries(pool, range, &filter.scope),
            );
            data.hist = faults.take("latency histogram", hist_res);
            data.series = faults.take("request timeseries", series_res);
        },
        RequestsTab::Models => {
            data.breakdown = faults.take(
                "requests by model",
                list_requests_by_model(pool, range, &filter.scope).await,
            );
        },
        RequestsTab::Providers => {
            data.breakdown = faults.take(
                "requests by provider",
                list_requests_by_provider(pool, range, &filter.scope).await,
            );
        },
        RequestsTab::Status => {
            data.breakdown = faults.take(
                "requests by status",
                list_requests_by_status(pool, range, &filter.scope).await,
            );
        },
        RequestsTab::Log => {},
    }
}
