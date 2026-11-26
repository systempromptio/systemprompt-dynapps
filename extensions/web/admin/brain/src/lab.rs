//! `/admin/brain/search`: the search laboratory.
//!
//! It runs the search the operations run, through the retriever's `explain`,
//! and shows each candidate with its rank and score in each arm, its fused
//! and reranked scores and what the deduplication did with it, then the
//! context `answer` would give the agent. "Run as" searches with another
//! person's permissions, and every run leaves a row in the audit.

use std::collections::HashSet;

use axum::Extension;
use axum::extract::Query;
use axum::response::Response;
use dynbrain::brain::args::AnswerArgs;
use dynbrain::console::Native;
use dynbrain::ids::SpaceId;
use dynbrain::search::filter::PropertyFilter;
use dynbrain::search::rerank::ScoreSource;
use dynbrain::search::{
    ArmScore, ExplainRow, Explained, Outcome, RetrievalMode, RetrievalRequest, SearchScope,
};
use serde::{Deserialize, Serialize};
use systemprompt::identifiers::UserId;
use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::ssr::helpers::{Viewer, render_page_with_faults};
use systemprompt_web_admin_core::ssr::page::Page;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;

use crate::source::{Source, require_admin};
use crate::views::{AT_SOURCE, NONE, NOT_MOUNTED, clip, crumbs};

const SHOWN: usize = 10;
const QUOTA: usize = 25_000;

#[derive(Debug, Default, Deserialize)]
pub struct LabQuery {
    #[serde(default)]
    pub q: String,
    #[serde(default)]
    pub lang: String,
    #[serde(default)]
    pub space: String,
    #[serde(default)]
    pub run_as: String,
    #[serde(default)]
    pub rerank: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LabRow {
    pub n: String,
    pub title: String,
    pub snippet: String,
    pub vector: String,
    pub text: String,
    pub rrf: String,
    pub rerank: String,
    pub outcome: String,
    pub dropped: bool,
    pub low: bool,
}

#[derive(Debug, Default, Serialize)]
pub struct LabPage {
    pub page: &'static str,
    pub title: &'static str,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub unavailable: Option<&'static str>,
    pub proxy_note: Option<&'static str>,
    pub q: String,
    pub lang: String,
    pub space: String,
    pub run_as: String,
    pub rerank: bool,
    pub ran: bool,
    pub run_as_name: String,
    pub scoring: String,
    pub mode: &'static str,
    pub timings: String,
    pub rows: Vec<LabRow>,
    pub context: String,
    pub context_chars: usize,
    pub quota: usize,
}

pub async fn search_page(
    shell: Page,
    Extension(source): Extension<Source>,
    Query(query): Query<LabQuery>,
) -> AdminHtmlResult<Response> {
    require_admin(&shell)?;
    let mut faults = Faults::new();
    let mut page = LabPage {
        page: "brain-search",
        title: "Search lab",
        breadcrumbs: crumbs("Search lab"),
        rerank: query.rerank.is_some() || query.q.is_empty(),
        q: query.q.clone(),
        lang: query.lang.clone(),
        space: query.space.clone(),
        run_as: query.run_as.clone(),
        quota: QUOTA,
        ..LabPage::default()
    };
    match source.console() {
        None => page.unavailable = Some(NOT_MOUNTED),
        Some(console) => match &console.native {
            None => page.proxy_note = Some(AT_SOURCE),
            Some(native) if !query.q.trim().is_empty() => {
                if let Err(error) = run(&mut page, native, &shell.user.user_id, &query).await {
                    faults.record("Search lab", &error);
                }
            },
            Some(_) => {},
        },
    }
    Ok(render_page_with_faults(
        &shell.engine,
        "brain-search",
        &page,
        Viewer::new(&shell.user, &shell.marketplace),
        &faults,
    ))
}

async fn run(
    page: &mut LabPage,
    native: &Native,
    admin: &UserId,
    query: &LabQuery,
) -> Result<(), AdminError> {
    let acting = !query.run_as.trim().is_empty();
    let reader = if acting {
        UserId::new(query.run_as.trim().to_owned())
    } else {
        admin.clone()
    };
    let action = if acting { "run_as" } else { "search_lab" };
    native
        .repo
        .record_admin_action(
            admin.as_str(),
            action,
            acting.then_some(reader.as_str()),
            query.q.trim(),
        )
        .await
        .map_err(AdminError::internal)?;
    let lang = Some(query.lang.trim().to_owned()).filter(|lang| !lang.is_empty());
    let space = query.space.trim().parse::<i64>().ok().map(SpaceId);
    let scope = SearchScope {
        reader: reader.clone(),
        space,
        lang: lang.clone(),
        filter: PropertyFilter::default(),
    };
    let request = RetrievalRequest {
        query: query.q.trim(),
        scope: &scope,
        limit: SHOWN,
        prefer_lang: lang.as_deref(),
    };
    let explained = native
        .retriever
        .explain(&request, page.rerank)
        .await
        .map_err(AdminError::internal)?;
    show(page, &explained, native.retriever.settings().weak_match_threshold);
    if acting {
        reader.as_str().clone_into(&mut page.run_as_name);
        let denied = denied_rows(native, admin, &reader, &request).await?;
        page.rows.extend(denied);
    }
    let args = AnswerArgs {
        question: query.q.trim().to_owned(),
        lang,
        space,
        max_sources: None,
    };
    let answer = native
        .brain
        .context(&reader, &args)
        .await
        .map_err(AdminError::internal)?;
    page.context_chars = answer.context.chars().count();
    page.context = answer.context;
    Ok(())
}

fn show(page: &mut LabPage, explained: &Explained, threshold: f64) {
    page.ran = true;
    page.mode = match explained.mode {
        RetrievalMode::Hybrid => "hybrid",
        RetrievalMode::Keyword => "keyword",
    };
    page.scoring = match explained.trace.scoring {
        ScoreSource::Fusion => "fusion".to_owned(),
        ScoreSource::CrossEncoder => "cross-encoder".to_owned(),
    };
    page.timings = timings(explained);
    page.rows = explained
        .rows
        .iter()
        .map(|row| lab_row(row, explained, threshold))
        .collect();
}

async fn denied_rows(
    native: &Native,
    admin: &UserId,
    reader: &UserId,
    request: &RetrievalRequest<'_>,
) -> Result<Vec<LabRow>, AdminError> {
    let reach: HashSet<i64> = native
        .repo
        .list_article_levels(reader, None)
        .await
        .map_err(AdminError::internal)?
        .into_iter()
        .map(|reached| reached.article.0)
        .collect();
    let scope = SearchScope {
        reader: admin.clone(),
        ..request.scope.clone()
    };
    let wide = RetrievalRequest {
        scope: &scope,
        ..*request
    };
    let all = native
        .retriever
        .explain(&wide, false)
        .await
        .map_err(AdminError::internal)?;
    Ok(all
        .kept
        .iter()
        .filter(|passage| !reach.contains(&passage.article.0))
        .map(|passage| LabRow {
            n: String::new(),
            title: passage.title.clone(),
            snippet: clip(&passage.content, 80),
            vector: NONE.to_owned(),
            text: NONE.to_owned(),
            rrf: NONE.to_owned(),
            rerank: NONE.to_owned(),
            outcome: format!("No access for {} (filtered in SQL)", reader.as_str()),
            dropped: true,
            low: false,
        })
        .collect())
}

fn lab_row(row: &ExplainRow, explained: &Explained, threshold: f64) -> LabRow {
    let place = explained
        .kept
        .iter()
        .position(|kept| kept.chunk == row.passage.chunk);
    let arm = |arm: Option<ArmScore>, digits: usize| {
        arm.map_or_else(
            || NONE.to_owned(),
            |arm| format!("{} · {:.digits$}", arm.rank, arm.value),
        )
    };
    LabRow {
        n: place.map(|at| format!("[{}]", at + 1)).unwrap_or_default(),
        title: row.passage.title.clone(),
        snippet: clip(&row.passage.content, 80),
        vector: arm(row.vector, 2),
        text: arm(row.text, 3),
        rrf: format!("{:.4}", row.fused),
        rerank: row
            .rerank
            .map_or_else(|| NONE.to_owned(), |score| format!("{score:.2}")),
        outcome: match row.outcome {
            Outcome::Kept => "Kept".to_owned(),
            Outcome::SameDocument => "Same document as a better chunk".to_owned(),
            Outcome::NearDuplicate => "Jaccard ≥ 0.85 with a kept chunk".to_owned(),
            Outcome::OverLimit => format!("Beyond the first {SHOWN} results"),
        },
        dropped: row.outcome != Outcome::Kept,
        low: row.rerank.is_some_and(|score| score < threshold),
    }
}

fn timings(explained: &Explained) -> String {
    let at = explained.timings;
    let cache = if explained.cached {
        "cache hit"
    } else {
        "embedded"
    };
    format!(
        "query {cache} {} ms · arms {} ms · rerank {} ms · RRF k=60 · {} candidates, {} kept",
        at.embedding,
        at.arms,
        at.rerank,
        explained.rows.len(),
        explained.kept.len()
    )
}
