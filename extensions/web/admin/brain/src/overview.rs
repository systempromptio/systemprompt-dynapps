//! `/admin/brain`: the mode of the instance, what is indexed, the providers,
//! the searches that found nothing and what the month cost.

use axum::response::Response;
use chrono::{TimeDelta, Utc};
use dynbrain::config::Mode;
use dynbrain::console::{Console, Native};
use dynbrain::ingest::context::{SYNOPSIS_DOC_CHARS, SYNOPSIS_MAX_WORDS};
use dynbrain::ingest::core_ai::EMBEDDING_MODEL;
use dynbrain::ingest::provider::DIMENSIONS;
use dynbrain::search::cohere;
use serde::Serialize;
use systemprompt_web_admin_core::error::AdminHtmlResult;
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::ssr::helpers::{Viewer, render_page_with_faults};
use systemprompt_web_admin_core::ssr::page::Page;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;

use crate::read::{self, Space, Who};
use crate::source::{Reach, require_admin};
use crate::views::{AT_SOURCE, NONE, NOT_MOUNTED, Tile, keep, tile, usd};

#[derive(Debug, Serialize)]
pub struct ProviderRow {
    pub step: &'static str,
    pub provider: &'static str,
    pub detail: String,
}

#[derive(Debug, Serialize)]
pub struct GapRow {
    pub query: String,
    pub space: String,
    pub times: i64,
}

#[derive(Debug, Serialize)]
pub struct SpendRow {
    pub provider: String,
    pub calls: i64,
    pub usd: String,
}

#[derive(Debug, Serialize)]
pub struct SpaceRow {
    pub name: String,
    pub slug: String,
    pub articles: usize,
    pub langs: String,
}

#[derive(Debug, Default, Serialize)]
pub struct OverviewPage {
    pub page: &'static str,
    pub title: &'static str,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub unavailable: Option<&'static str>,
    pub mode: &'static str,
    pub proxy_note: Option<&'static str>,
    pub tiles: Vec<Tile>,
    pub providers: Vec<ProviderRow>,
    pub threshold: String,
    pub gaps: Vec<GapRow>,
    pub spend: Vec<SpendRow>,
    pub spaces: Vec<SpaceRow>,
}

pub async fn overview_page(shell: Page, reach: Reach) -> AdminHtmlResult<Response> {
    require_admin(&shell)?;
    let mut faults = Faults::new();
    let mut page = OverviewPage {
        page: "brain",
        title: "Brain",
        breadcrumbs: vec![BreadcrumbView::current("Brain")],
        ..OverviewPage::default()
    };
    let who = Who {
        id: &shell.user.user_id,
        token: reach.token.as_ref(),
    };
    match reach.source.console() {
        Some(console) => fill(&mut page, &console, who, &mut faults).await,
        None => page.unavailable = Some(NOT_MOUNTED),
    }
    Ok(render_page_with_faults(
        &shell.engine,
        "brain",
        &page,
        Viewer::new(&shell.user, &shell.marketplace),
        &faults,
    ))
}

async fn fill(page: &mut OverviewPage, console: &Console, who: Who<'_>, faults: &mut Faults) {
    let spaces = faults.take("Spaces", read::read_spaces(console, who).await);
    let articles: usize = spaces.iter().map(|space| space.articles.len()).sum();
    page.spaces = space_rows(&spaces);
    match (&console.mode, &console.native) {
        (Mode::Native, Some(native)) => {
            page.mode = "Native";
            page.providers = providers();
            page.threshold = format!("{:.2}", native.retriever.settings().weak_match_threshold);
            native_blocks(page, native, articles, faults).await;
        },
        _ => {
            page.mode = "Proxy";
            page.proxy_note = Some(AT_SOURCE);
            page.tiles = vec![
                tile("Articles", articles, None),
                tile("Chunks", NONE, None),
                tile("Errors", NONE, None),
            ];
        },
    }
}

async fn native_blocks(
    page: &mut OverviewPage,
    native: &Native,
    articles: usize,
    faults: &mut Faults,
) {
    let week = Utc::now() - TimeDelta::days(7);
    let month = Utc::now() - TimeDelta::days(30);
    if let Some(counts) = keep(faults, "Index", native.repo.count_index().await) {
        let tone = |n: i64, tone: &'static str| (n > 0).then_some(tone);
        page.tiles = vec![
            tile("Articles", articles, None),
            tile("Documents indexed", counts.documents, None),
            tile("Chunks", counts.chunks, None),
            tile(
                "Embeddings pending",
                counts.pending_embeddings,
                tone(counts.pending_embeddings, "warn"),
            ),
            tile(
                "Synopses pending",
                counts.pending_synopses,
                tone(counts.pending_synopses, "warn"),
            ),
            tile(
                "Errors",
                counts.chunk_errors + counts.failed_jobs,
                tone(counts.chunk_errors + counts.failed_jobs, "err"),
            ),
        ];
    }
    if let Some(searches) = keep(faults, "Searches", native.repo.count_searches(week).await) {
        page.tiles.push(tile("Searches · 7 days", searches, None));
    }
    let gaps = keep(
        faults,
        "Searches",
        native.repo.list_empty_searches(week, 8).await,
    );
    page.gaps = gaps
        .unwrap_or_default()
        .into_iter()
        .map(|gap| GapRow {
            query: gap.query,
            space: gap.space.unwrap_or_else(|| "all".to_owned()),
            times: gap.times,
        })
        .collect();
    let spend = keep(
        faults,
        "Cost",
        native.repo.list_embedding_spend(month).await,
    );
    page.spend = spend
        .unwrap_or_default()
        .into_iter()
        .map(|row| SpendRow {
            provider: row.fingerprint.replace(':', " · "),
            calls: row.calls,
            usd: usd(row.micros),
        })
        .collect();
}

fn providers() -> Vec<ProviderRow> {
    vec![
        ProviderRow {
            step: "Embeddings",
            provider: "OpenAI",
            detail: format!("{EMBEDDING_MODEL} · {DIMENSIONS}"),
        },
        ProviderRow {
            step: "Synopsis",
            provider: "Anthropic",
            detail: format!(
                "claude-haiku-5-5 · up to {SYNOPSIS_MAX_WORDS} words over {SYNOPSIS_DOC_CHARS} characters"
            ),
        },
        ProviderRow {
            step: "Reranker",
            provider: "Cohere",
            detail: format!("{} · fails open behind a circuit breaker", cohere::MODEL),
        },
        ProviderRow {
            step: "Keyword",
            provider: "Postgres",
            detail: "tsvector in each chunk's language".to_owned(),
        },
        ProviderRow {
            step: "Chunking",
            provider: "—",
            detail: "300 words, 50 of overlap, with heading_path".to_owned(),
        },
    ]
}

fn space_rows(spaces: &[Space]) -> Vec<SpaceRow> {
    spaces
        .iter()
        .map(|space| SpaceRow {
            name: space.item.name.clone(),
            slug: space.item.slug.clone(),
            articles: space.articles.len(),
            langs: space.item.langs.join(", "),
        })
        .collect()
}
