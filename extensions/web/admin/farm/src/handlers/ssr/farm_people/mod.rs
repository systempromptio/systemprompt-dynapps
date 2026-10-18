//! `/admin/farm/people` — who launches farm runs, how often they succeed,
//! what they cost, and how they launch them. Each row drills into the run
//! explorer filtered to that person.

mod context;
pub(crate) mod rows;

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::response::Response;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::ssr::helpers::{Viewer, render_page_with_faults};
use systemprompt_web_admin_core::ssr::list_view::SelectOptionView;
use systemprompt_web_admin_core::ssr::page::Page;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;
use systemprompt_web_admin_core::util::time_range::{TimeRangeQuery, parse_time_range};

use context::{BASE_URL, PeopleKpi, PeopleTimeRange, farm_options, kpis, time_range};
use rows::{PersonRowView, person_rows};

use crate::repositories::farms::list_farms;
use crate::repositories::filters::FarmRunFilter;
use crate::repositories::people::list_runs_by_user;
use crate::repositories::runs::get_run_stats;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct PeopleQuery {
    pub preset: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub farm: Option<String>,
}

#[derive(Debug, Serialize)]
struct PeoplePage {
    page: &'static str,
    title: &'static str,
    breadcrumbs: Vec<BreadcrumbView>,
    base_url: &'static str,
    time_range: PeopleTimeRange,
    farms: Vec<SelectOptionView>,
    kpis: Vec<PeopleKpi>,
    people: Vec<PersonRowView>,
    has_people: bool,
    row_count: String,
}

pub(crate) async fn people_page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<PeopleQuery>,
) -> AdminHtmlResult<Response> {
    if !shell.user.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    let range = parse_time_range(&TimeRangeQuery {
        from: query.from.clone(),
        to: query.to.clone(),
        preset: query.preset.clone(),
    });
    let mut faults = Faults::new();
    let farms = faults.take("farms", list_farms(&pool).await);
    let farm = query
        .farm
        .as_deref()
        .and_then(|wanted| farms.iter().find(|f| f.id.as_str() == wanted))
        .map(|f| f.id.clone());
    let farm_key = farm.as_ref().map(|f| f.as_str().to_owned());

    let filter = FarmRunFilter {
        farm_id: farm.clone(),
        since: Some(range.from),
        until: Some(range.to),
        ..FarmRunFilter::default()
    };
    let stats = faults.take("run stats", get_run_stats(&pool, &filter).await);
    let leaderboard = faults.take(
        "runs by person",
        list_runs_by_user(&pool, range.from, farm.as_ref()).await,
    );
    let people = person_rows(&leaderboard);

    let ctx = PeoplePage {
        page: "farm-people",
        title: "By person",
        breadcrumbs: vec![
            BreadcrumbView::link("Agent farms", "/admin/farm"),
            BreadcrumbView::current("By person"),
        ],
        base_url: BASE_URL,
        time_range: time_range(query.preset.as_deref(), farm_key.as_deref(), range),
        farms: farm_options(&farms, farm_key.as_deref()),
        kpis: kpis(&stats, &people),
        has_people: !people.is_empty(),
        row_count: match people.len() {
            1 => "1 person".to_owned(),
            n => format!("{n} people"),
        },
        people,
    };
    Ok(render_page_with_faults(
        &shell.engine,
        "farm-people",
        &ctx,
        Viewer::new(&shell.user, &shell.marketplace),
        &faults,
    ))
}
