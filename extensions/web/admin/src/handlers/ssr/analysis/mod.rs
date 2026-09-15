//! Skill drill-down over retained reporting data: organisation totals, one row
//! per managed resource, and the conversations behind a selected resource.
//! Nothing here is computed live; every number is read from the snapshot
//! pipeline's retained tables through `repositories::analysis::retained`.
use crate::error::{AdminError, AdminHtmlResult, AdminResult};
use crate::handlers::ssr::list_view::Pagination;
use crate::repositories::analysis::inventory_index::InventoryIndex;
use crate::repositories::analysis::retained::{
    self, PAGE_SIZE, RetainedResourceRef, RetainedTotals, RetainedWindow,
};
use crate::routes::managed_state::ManagedState;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::extract::{Extension, Query, State};
use axum::response::Response;
use chrono::{DateTime, Duration, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::identifiers::ManagedResourceId;
pub(crate) mod campaign_completion;
pub(crate) mod campaigns;
pub(crate) mod candidates;
pub(crate) mod experiments;
pub(crate) mod impact;
pub(crate) mod lifecycle;
pub(crate) mod portfolio;
pub(crate) mod time;
pub(crate) mod versions;
pub(crate) mod views;
pub(crate) use views::dollars;
pub use views::{RetainedConversationView, RetainedResourceView};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisQuery {
    #[serde(
        default,
        deserialize_with = "time::deserialize",
        serialize_with = "time::serialize"
    )]
    pub start: Option<DateTime<Utc>>,
    #[serde(
        default,
        deserialize_with = "time::deserialize",
        serialize_with = "time::serialize"
    )]
    pub end: Option<DateTime<Utc>>,
    pub resource: Option<String>,
    pub page: Option<u32>,
}

impl AnalysisQuery {
    // Why: the default window ends at the next UTC midnight so today's drained
    // facts are inside it, and starts 30 days earlier — the same day
    // boundaries the daily snapshot buckets facts under.
    pub fn window(&self, now: DateTime<Utc>) -> Result<RetainedWindow, AdminError> {
        let end = self.end.unwrap_or_else(|| {
            (now.date_naive() + Duration::days(1))
                .and_time(NaiveTime::MIN)
                .and_utc()
        });
        let start = self.start.unwrap_or(end - Duration::days(30));
        if start >= end || end - start > Duration::days(366) {
            return Err(AdminError::BadRequest(
                "Choose a window between 1 day and 366 days".into(),
            ));
        }
        Ok(RetainedWindow { start, end })
    }

    pub fn resource(&self) -> Result<Option<ManagedResourceId>, AdminError> {
        let Some(raw) = self.resource.as_deref().map(str::trim) else {
            return Ok(None);
        };
        if raw.is_empty() {
            return Ok(None);
        }
        if raw.len() > 200 || raw.chars().any(|c| c.is_control() || c.is_whitespace()) {
            return Err(AdminError::BadRequest("Invalid resource id".into()));
        }
        Ok(Some(ManagedResourceId::new(raw)))
    }

    #[must_use]
    pub fn page(&self) -> u32 {
        self.page.unwrap_or(1).max(1)
    }

    #[must_use]
    pub fn url(&self, page: u32, resource: Option<&str>) -> String {
        let mut pairs = Vec::new();
        for (key, value) in [
            ("start", self.start.map(time::render)),
            ("end", self.end.map(time::render)),
            ("resource", resource.map(str::to_owned)),
        ] {
            if let Some(value) = value {
                pairs.push((key, value));
            }
        }
        if page > 1 {
            pairs.push(("page", page.to_string()));
        }
        let query = pairs
            .into_iter()
            .map(|(k, v)| format!("{k}={}", urlencoding::encode(&v)))
            .collect::<Vec<_>>()
            .join("&");
        if query.is_empty() {
            "/admin/analysis/skills".to_owned()
        } else {
            format!("/admin/analysis/skills?{query}")
        }
    }
}

#[must_use]
pub fn drilldown_url(resource: &ManagedResourceId, window: (&str, &str)) -> String {
    format!(
        "/admin/analysis/skills?resource={}&start={}&end={}",
        urlencoding::encode(resource.as_str()),
        urlencoding::encode(window.0),
        urlencoding::encode(window.1)
    )
}

fn paginate(query: &AnalysisQuery, resource: Option<&str>, total: i64, shown: usize) -> Pagination {
    let page = i64::from(query.page());
    let size = i64::from(PAGE_SIZE);
    let total_pages = ((total + size - 1) / size).max(1);
    let offset = (page - 1) * size;
    let shown = i64::try_from(shown).unwrap_or(i64::MAX);
    let link = |target: i64| query.url(u32::try_from(target).unwrap_or(u32::MAX), resource);
    Pagination {
        current_page: page.min(total_pages),
        total_pages,
        first_row: if shown == 0 { 0 } else { offset + 1 },
        last_row: offset + shown,
        total_rows: total,
        noun: if resource.is_some() {
            "conversations"
        } else {
            "resources"
        },
        has_prev: page > 1,
        has_next: page < total_pages,
        prev_url: (page > 1).then(|| link(page - 1)),
        next_url: (page < total_pages).then(|| link(page + 1)),
    }
}

#[derive(Serialize)]
struct AnalysisPage {
    page: &'static str,
    title: &'static str,
    totals: RetainedTotals,
    cost: String,
    filter: AnalysisQuery,
    all_href: String,
    resources: Vec<RetainedResourceView>,
    selected: Option<RetainedResourceRef>,
    conversations: Vec<RetainedConversationView>,
    detail: bool,
    pagination: Pagination,
}

async fn load(
    managed: &ManagedState,
    pool: &PgPool,
    mut query: AnalysisQuery,
) -> AdminResult<AnalysisPage> {
    let window = query.window(Utc::now())?;
    query.start = Some(window.start);
    query.end = Some(window.end);
    let resource = query.resource()?;
    query.resource = resource.as_ref().map(|r| r.as_str().to_owned());
    let page = query.page();
    let owner = &managed.owner;
    let totals = retained::get_retained_totals(pool, owner, window).await?;
    let mut resources = Vec::new();
    let mut conversations = Vec::new();
    let mut selected = None;
    let total;
    if let Some(resource) = resource.as_ref() {
        selected = retained::find_retained_resource(pool, owner, resource).await?;
        if selected.is_none() {
            return Err(AdminError::NotFound("Unknown managed resource".into()));
        }
        let (rows, count) =
            retained::list_retained_conversations(pool, owner, resource, window, page).await?;
        total = count;
        conversations.extend(rows.into_iter().map(RetainedConversationView::new));
    } else {
        let index = InventoryIndex::build(&crate::handlers::shared::get_services_path()?);
        let (rows, count) = retained::list_retained_resources(pool, owner, window, page).await?;
        total = count;
        for row in rows {
            let href = query.url(1, Some(row.resource_id.as_str()));
            resources.push(RetainedResourceView::new(row, href, &index));
        }
    }
    let shown = resources.len().max(conversations.len());
    let pagination = paginate(&query, query.resource.as_deref(), total, shown);
    Ok(AnalysisPage {
        page: "analysis-skills",
        title: "Skill conversations",
        cost: dollars(totals.cost),
        totals,
        all_href: query.url(1, None),
        filter: query,
        resources,
        selected,
        conversations,
        detail: resource.is_some(),
        pagination,
    })
}

#[expect(
    clippy::too_many_arguments,
    reason = "axum extractor list; the router decides the arity, not this signature"
)]
pub(crate) async fn skills_page(
    Extension(user): Extension<UserContext>,
    Extension(marketplace): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    Extension(managed): Extension<Arc<ManagedState>>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<AnalysisQuery>,
) -> AdminHtmlResult<Response> {
    if !user.is_console {
        return Err(AdminError::Forbidden("Console access required".into()).into());
    }
    let context = load(&managed, &pool, query).await?;
    Ok(super::render_typed_page(
        &engine,
        "analysis-skills",
        &context,
        &user,
        &marketplace,
    ))
}
