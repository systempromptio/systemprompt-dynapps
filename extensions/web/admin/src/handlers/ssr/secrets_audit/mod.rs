//! `/admin/governance/secrets` — every recorded touch of a stored credential.
//!
//! Secrets are sealed with a per-user key and plaintext never reaches the
//! database, so this append-only table is the entire record of what happened to
//! a credential. The question it answers is not "what is stored" but "who read
//! it, and were they its owner" — which is why the owner and the actor are two
//! columns and the count of rows where they differ is a KPI.
//!
//! Until now the trail was reachable only through the per-plugin JSON API. The
//! query here is the console-wide form of that read, paged and exportable.

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult, AdminResult};
use crate::handlers::ssr::csv::CsvBuilder;
use crate::handlers::ssr::list_view::{PageWindow, Pagination, SelectOptionView, TimeRangeContext};
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories::governance::secret_audit_log::{
    SecretAuditFilter, SecretAuditStats, get_secret_audit_stats, list_secret_audit_actions,
    list_secret_audit_paged,
};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use crate::util::time_range::{TimeRange, TimeRangeQuery, parse_time_range};

mod view;

const BASE_URL: &str = "/admin/governance/secrets";
const PAGE_SIZE: i64 = 50;
const CSV_LIMIT: i64 = 5_000;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct SecretsQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub preset: Option<String>,
    pub action: Option<String>,
    pub q: Option<String>,
    pub page: Option<i64>,
}

#[derive(Debug, Serialize)]
struct KpiView {
    label: &'static str,
    value: String,
    sub: &'static str,
    tone: &'static str,
    href: String,
    active: bool,
    hint: &'static str,
}

#[derive(Debug, Serialize)]
struct SecretsPageContext {
    page: &'static str,
    title: &'static str,
    subtitle: &'static str,
    breadcrumbs: Vec<BreadcrumbView>,
    time_range: TimeRangeContext,
    kpis: Vec<KpiView>,
    rows: Vec<view::SecretAuditRowView>,
    has_rows: bool,
    row_count: String,
    pagination: Pagination,
    actions: Vec<SelectOptionView>,
    search: String,
    csv_url: String,
    clear_url: &'static str,
    has_filters: bool,
    base_url: &'static str,
}

fn require_console(user_ctx: &UserContext) -> Result<(), AdminError> {
    if user_ctx.is_console {
        return Ok(());
    }
    Err(AdminError::Forbidden("Admin access required.".to_owned()))
}

fn range_of(query: &SecretsQuery) -> TimeRange {
    parse_time_range(&TimeRangeQuery {
        from: query.from.clone(),
        to: query.to.clone(),
        preset: query.preset.clone(),
    })
}

fn filter_of(query: &SecretsQuery) -> SecretAuditFilter {
    SecretAuditFilter {
        action: query
            .action
            .as_deref()
            .filter(|a| !a.is_empty())
            .map(str::to_owned),
        search: query
            .q
            .as_deref()
            .filter(|q| !q.trim().is_empty())
            .map(str::to_owned),
    }
}

fn url_for(query: &SecretsQuery, action: Option<&str>, page: i64) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(preset) = query.preset.as_deref().filter(|p| !p.is_empty()) {
        parts.push(format!("preset={}", urlencoding::encode(preset)));
    }
    if let Some(action) = action.filter(|a| !a.is_empty()) {
        parts.push(format!("action={}", urlencoding::encode(action)));
    }
    if let Some(search) = query.q.as_deref().filter(|q| !q.is_empty()) {
        parts.push(format!("q={}", urlencoding::encode(search)));
    }
    if page > 0 {
        parts.push(format!("page={page}"));
    }
    if parts.is_empty() {
        BASE_URL.to_owned()
    } else {
        format!("{BASE_URL}?{}", parts.join("&"))
    }
}

pub(crate) async fn secrets_audit_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<SecretsQuery>,
) -> AdminHtmlResult<Response> {
    require_console(&user_ctx)?;

    let range = range_of(&query);
    let filter = filter_of(&query);
    let page = query.page.unwrap_or(0).max(0);

    let stats = get_secret_audit_stats(&pool, range)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "secret audit stats failed");
            SecretAuditStats::default()
        });
    let (rows, total) = list_secret_audit_paged(&pool, range, &filter, PAGE_SIZE, page * PAGE_SIZE)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "secret audit log failed");
            (Vec::new(), 0)
        });
    let actions = list_secret_audit_actions(&pool, range)
        .await
        .unwrap_or_default();

    let shown = i64::try_from(rows.len()).unwrap_or(0);
    let window = PageWindow::new(page, PAGE_SIZE, total, shown, "entries");
    let (first_row, last_row) = window.bounds();

    let ctx = SecretsPageContext {
        page: "governance-secrets",
        title: "Secrets audit",
        subtitle: "Every recorded action against a stored credential. Plaintext never \
                   reaches the database, so this trail is the whole record of who read \
                   what, and whether they owned it.",
        breadcrumbs: vec![
            BreadcrumbView::link("Admin", "/admin"),
            BreadcrumbView::link("Governance", "/admin/governance"),
            BreadcrumbView::current("Secrets audit"),
        ],
        time_range: TimeRangeContext {
            preset: query.preset.clone().unwrap_or_else(|| "7d".to_owned()),
            from: range.from.format("%Y-%m-%dT%H:%M").to_string(),
            to: range.to.format("%Y-%m-%dT%H:%M").to_string(),
            base_url: BASE_URL,
            query: "",
        },
        kpis: kpis(&stats, &query),
        rows: view::rows(&rows),
        has_rows: !rows.is_empty(),
        row_count: format!("{total} entries"),
        pagination: Pagination {
            current_page: page + 1,
            total_pages: window.total_pages,
            first_row,
            last_row,
            total_rows: total,
            noun: "entries",
            has_prev: page > 0,
            has_next: page + 1 < window.total_pages,
            prev_url: (page > 0).then(|| url_for(&query, filter.action.as_deref(), page - 1)),
            next_url: (page + 1 < window.total_pages)
                .then(|| url_for(&query, filter.action.as_deref(), page + 1)),
        },
        actions: action_options(&actions, filter.action.as_deref()),
        search: query.q.clone().unwrap_or_default(),
        csv_url: format!(
            "{BASE_URL}.csv?preset={}",
            query.preset.as_deref().unwrap_or("7d")
        ),
        clear_url: BASE_URL,
        has_filters: filter.action.is_some() || filter.search.is_some(),
        base_url: BASE_URL,
    };

    Ok(super::render_typed_page(
        &engine,
        "governance-secrets",
        &ctx,
        &user_ctx,
        &mkt_ctx,
    ))
}

pub(crate) async fn secrets_audit_csv(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<SecretsQuery>,
) -> AdminResult<Response> {
    require_console(&user_ctx)?;

    let range = range_of(&query);
    let (rows, _) = list_secret_audit_paged(&pool, range, &filter_of(&query), CSV_LIMIT, 0).await?;

    let mut csv = CsvBuilder::new(&[
        "at",
        "action",
        "variable",
        "plugin",
        "owner",
        "actor",
        "third_party",
        "ip",
    ]);
    for row in &rows {
        csv.row(&[
            &row.created_at.to_rfc3339(),
            &row.action,
            &row.var_name,
            &row.plugin_id,
            row.user_id.as_str(),
            row.actor_id.as_str(),
            if row.actor_id == row.user_id {
                "no"
            } else {
                "yes"
            },
            row.ip_address.as_deref().unwrap_or(""),
        ]);
    }

    Ok(csv.into_response(&format!(
        "secrets-audit-{}.csv",
        range.from.format("%Y%m%d")
    )))
}

// Why: `third_party` is the tile that earns the page. Every other number can be
// counted off the table; "an administrator read someone else's credential"
// cannot, and it is the row an auditor came here to find.
fn kpis(stats: &SecretAuditStats, query: &SecretsQuery) -> Vec<KpiView> {
    vec![
        KpiView {
            label: "Entries",
            value: stats.entries.to_string(),
            sub: "recorded actions",
            tone: "",
            href: url_for(query, None, 0),
            active: query.action.is_none(),
            hint: "Rows in the window, before any filter",
        },
        KpiView {
            label: "Variables",
            value: stats.variables.to_string(),
            sub: "distinct secrets touched",
            tone: "",
            href: url_for(query, None, 0),
            active: false,
            hint: "Distinct variable names appearing in the window",
        },
        KpiView {
            label: "Actors",
            value: stats.actors.to_string(),
            sub: "people who acted",
            tone: "",
            href: url_for(query, None, 0),
            active: false,
            hint: "Distinct actors, which is not the same as distinct owners",
        },
        KpiView {
            label: "Reads",
            value: stats.accesses.to_string(),
            sub: "decrypt events",
            tone: "warn",
            href: url_for(query, Some("accessed"), 0),
            active: query.action.as_deref() == Some("accessed"),
            hint: "Every time a stored secret was unsealed",
        },
        KpiView {
            label: "Rotations",
            value: stats.rotations.to_string(),
            sub: "credentials replaced",
            tone: "ok",
            href: url_for(query, Some("rotated"), 0),
            active: query.action.as_deref() == Some("rotated"),
            hint: "Rotation is the healthy signal on this page",
        },
        KpiView {
            label: "Third party",
            value: stats.third_party.to_string(),
            sub: "actor was not the owner",
            tone: if stats.third_party > 0 { "err" } else { "ok" },
            href: url_for(query, None, 0),
            active: false,
            hint: "Rows where an administrator acted on someone else's secret",
        },
    ]
}

fn action_options(actions: &[String], selected: Option<&str>) -> Vec<SelectOptionView> {
    let mut out = vec![SelectOptionView {
        value: String::new(),
        label: "All actions".to_owned(),
        selected: selected.is_none(),
    }];
    out.extend(actions.iter().map(|action| SelectOptionView {
        selected: selected == Some(action.as_str()),
        value: action.clone(),
        label: action.clone(),
    }));
    out
}
