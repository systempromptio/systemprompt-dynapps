//! `/admin/brain/usage`: the documents most retrieved and cited, the searches
//! that found nothing and the embedding cost. Counts only: the log holds no
//! person and no place.

use axum::Extension;
use axum::response::Response;
use chrono::{TimeDelta, Utc};
use serde::Serialize;
use systemprompt_web_admin_core::error::AdminHtmlResult;
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::ssr::helpers::{Viewer, render_page_with_faults};
use systemprompt_web_admin_core::ssr::page::Page;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;

use crate::overview::{GapRow, SpendRow};
use crate::source::{Source, require_admin};
use crate::views::{AT_SOURCE, NOT_MOUNTED, crumbs, keep, usd};

const SHOWN: i64 = 20;

#[derive(Debug, Serialize)]
pub struct UsedRow {
    pub article: i64,
    pub title: String,
    pub retrieved: i64,
    pub cited: i64,
}

#[derive(Debug, Default, Serialize)]
pub struct UsagePage {
    pub page: &'static str,
    pub title: &'static str,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub unavailable: Option<&'static str>,
    pub proxy_note: Option<&'static str>,
    pub used: Vec<UsedRow>,
    pub gaps: Vec<GapRow>,
    pub spend: Vec<SpendRow>,
}

pub async fn usage_page(
    shell: Page,
    Extension(source): Extension<Source>,
) -> AdminHtmlResult<Response> {
    require_admin(&shell)?;
    let mut faults = Faults::new();
    let mut page = UsagePage {
        page: "brain-usage",
        title: "Usage",
        breadcrumbs: crumbs("Usage"),
        ..UsagePage::default()
    };
    match source.console() {
        None => page.unavailable = Some(NOT_MOUNTED),
        Some(console) => match &console.native {
            None => page.proxy_note = Some(AT_SOURCE),
            Some(native) => {
                let since = Utc::now() - TimeDelta::days(30);
                let used = keep(&mut faults, "Usage", native.repo.list_used_articles(since, SHOWN).await);
                page.used = used
                    .unwrap_or_default()
                    .into_iter()
                    .map(|row| UsedRow {
                        article: row.article_id,
                        title: row.title,
                        retrieved: row.retrieved,
                        cited: row.cited,
                    })
                    .collect();
                let gaps = keep(&mut faults, "Searches", native.repo.list_empty_searches(since, SHOWN).await);
                page.gaps = gaps
                    .unwrap_or_default()
                    .into_iter()
                    .map(|gap| GapRow {
                        query: gap.query,
                        space: gap.space.unwrap_or_else(|| "all".to_owned()),
                        times: gap.times,
                    })
                    .collect();
                let spend = keep(&mut faults, "Cost", native.repo.list_embedding_spend(since).await);
                page.spend = spend
                    .unwrap_or_default()
                    .into_iter()
                    .map(|row| SpendRow {
                        provider: row.fingerprint.replace(':', " · "),
                        calls: row.calls,
                        usd: usd(row.micros),
                    })
                    .collect();
            },
        },
    }
    Ok(render_page_with_faults(
        &shell.engine,
        "brain-usage",
        &page,
        Viewer::new(&shell.user, &shell.marketplace),
        &faults,
    ))
}
