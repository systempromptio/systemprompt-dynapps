//! `/admin/analysis/skills` — every skill as the record shows it: who
//! invoked it and how often, the conversations behind that, the tokens and
//! cost those conversations ran up, tool calls, errors and denials, latency,
//! installs, and the judge's mean completion — grouped under the marketplace
//! that serves it, with a fourteen-day sparkline per row. Above the table,
//! marketplace adoption: entitled → installed → active, by host.
//!
//! Invocations are hook events; everything else is read from the
//! `conversation_facts` rollup through the harness session, so a figure here
//! is the same figure the Conversations page shows for the same conversation.


mod adoption;
mod context;
mod rows;
mod summary;
mod views;

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::response::Response;
use chrono::{Duration, Utc};
use serde::Deserialize;
use sqlx::PgPool;
use systemprompt::identifiers::MarketplaceId;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::analysis_urls::ANALYSIS_SKILLS_URL;
use crate::handlers::ssr::page::Page;
use crate::repositories;
use crate::repositories::analysis::inventory_index::{InventoryIndex, get_marketplace_audience};
use crate::repositories::analysis::skills::{
    SkillListFilter, SkillSort, SkillWindow, list_marketplace_adoption, list_skill_facts,
};
use crate::repositories::scope::ScopeRequest;
use crate::types::Role;

use context::{SkillsPageInputs, SkillsRead, build_context};
pub(crate) use rows::skill_row_view;
pub(crate) use views::SkillFactView;

pub(super) const SPARK_DAYS: i64 = 14;
const ROW_LIMIT: i64 = 500;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SkillsQuery {
    pub days: Option<i64>,
    pub marketplace: Option<String>,
    pub client: Option<String>,
    pub search: Option<String>,
    pub sort: Option<String>,
    pub group: Option<String>,
    pub project: Option<String>,
}

impl SkillsQuery {
    pub(super) const fn days(&self) -> i64 {
        match self.days {
            Some(7) => 7,
            Some(90) => 90,
            Some(365) => 365,
            _ => 30,
        }
    }

    pub(super) fn trimmed(value: Option<&str>) -> Option<String> {
        value
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_owned)
    }

    pub(super) fn sort(&self) -> SkillSort {
        SkillSort::parse_skill_sort(self.sort.as_deref())
    }

    // Why: the page minus the named filters — a chip's remove link and the
    // ribbon's Clear.
    pub(super) fn link_without(&self, drop: &[&str]) -> String {
        let cleared = Self {
            days: self.days,
            marketplace: (!drop.contains(&"marketplace"))
                .then(|| self.marketplace.clone())
                .flatten(),
            client: (!drop.contains(&"client"))
                .then(|| self.client.clone())
                .flatten(),
            search: (!drop.contains(&"search"))
                .then(|| self.search.clone())
                .flatten(),
            sort: (!drop.contains(&"sort"))
                .then(|| self.sort.clone())
                .flatten(),
            group: self.group.clone(),
            project: self.project.clone(),
        };
        cleared.link(None, None)
    }

    pub(super) fn link(&self, days: Option<i64>, sort: Option<SkillSort>) -> String {
        let mut pairs: Vec<(&str, String)> = Vec::new();
        let days = days.unwrap_or_else(|| self.days());
        if days != 30 {
            pairs.push(("days", days.to_string()));
        }
        let sort = sort.unwrap_or_else(|| self.sort());
        if sort != SkillSort::Invocations {
            pairs.push(("sort", sort.as_str().to_owned()));
        }
        for (key, value) in [
            ("marketplace", Self::trimmed(self.marketplace.as_deref())),
            ("client", Self::trimmed(self.client.as_deref())),
            ("search", Self::trimmed(self.search.as_deref())),
            ("group", Self::trimmed(self.group.as_deref())),
            ("project", Self::trimmed(self.project.as_deref())),
        ] {
            if let Some(value) = value {
                pairs.push((key, value));
            }
        }
        let qs = pairs
            .iter()
            .map(|(k, v)| format!("{k}={}", urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");
        if qs.is_empty() {
            ANALYSIS_SKILLS_URL.to_owned()
        } else {
            format!("{ANALYSIS_SKILLS_URL}?{qs}")
        }
    }
}

pub(crate) async fn page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<SkillsQuery>,
) -> AdminHtmlResult<Response> {
    if !shell.user.is_console {
        return Err(AdminError::Forbidden("Console access required".into()).into());
    }
    let request = ScopeRequest::from_query(
        &shell.user,
        query.group.as_deref(),
        query.project.as_deref(),
    );
    let scope = repositories::scope::membership::get_subject_scope(&pool, &request).await?;
    let end = Utc::now();
    let window = SkillWindow {
        start: end - Duration::days(query.days()),
        end,
        subject_ids: scope.as_sql().map(<[String]>::to_vec),
    };
    let filter = SkillListFilter {
        marketplace_id: SkillsQuery::trimmed(query.marketplace.as_deref()).map(MarketplaceId::new),
        search: SkillsQuery::trimmed(query.search.as_deref()),
        client_kind: SkillsQuery::trimmed(query.client.as_deref()),
        sort: query.sort(),
        limit: ROW_LIMIT,
        offset: 0,
        skills: None,
    };
    let index = InventoryIndex::build(&crate::handlers::shared::get_services_path()?);
    let roles: Vec<String> = Role::ALL.iter().map(|r| r.as_str().to_owned()).collect();
    let (rows, adoption, audience) = tokio::join!(
        list_skill_facts(&pool, &window, &filter),
        list_marketplace_adoption(&pool, &window),
        get_marketplace_audience(&pool, &index.marketplaces, &roles),
    );
    let (rows, adoption, audience) = (rows?, adoption?, audience?);
    let report_banner = crate::handlers::ssr::analysis::reports::banner::report_banner(
        &pool,
        "global",
        None,
        query
            .link(None, None)
            .split_once('?')
            .map_or("", |(_, q)| q),
        "Skills in view",
    )
    .await;
    let context = build_context(
        &query,
        &SkillsRead {
            rows: &rows,
            adoption: &adoption,
            audience: &audience,
            index: &index,
        },
        SkillsPageInputs {
            is_admin: shell.user.is_admin,
            report_banner,
        },
    );
    Ok(crate::handlers::ssr::render_typed_page(
        &shell.engine,
        "analysis-skills",
        &context,
        &shell.user,
        &shell.marketplace,
    ))
}
