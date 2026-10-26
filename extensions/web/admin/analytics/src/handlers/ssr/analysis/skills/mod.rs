//! `/admin/analysis/skills` — every skill as the record shows it, in three
//! views under one scope. The scope is the window and, optionally, one
//! marketplace; every tab honours both. *Overview* is the zoomed-out view:
//! one row per marketplace, entitled → installed → active with the window's
//! activity and spend. *Activity* is invocations per day over the window.
//! *Skills* is the table: one row per skill, grouped under the marketplace
//! and plugin that serve it, with a fourteen-day sparkline per row.
//!
//! Invocations are hook events; everything else is read from the
//! `conversation_facts` rollup through the harness session, so a figure here
//! is the same figure the Conversations page shows for the same conversation.
//! Only the active tab's data is read.

mod activity;
mod adoption;
mod context;
mod query;
mod rows;
mod summary;
mod tabs;
mod views;

use std::sync::Arc;
use systemprompt_web_admin_core::extension_boot::ExtensionBoot;

use axum::Extension;
use axum::extract::{Query, State};
use axum::response::Response;
use chrono::{Duration, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::MarketplaceId;

use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::ssr::page::Page;

use crate::repositories::analysis::skills::{
    SkillListFilter, SkillWindow, list_marketplace_adoption, list_skill_facts,
};
use systemprompt_web_admin_core::catalog_index::inventory_index::{
    InventoryIndex, get_marketplace_audience,
};
use systemprompt_web_admin_core::repositories::scope::ScopeRequest;

use context::{SkillsPageInputs, SkillsRead, build_context};
use query::{SkillsQuery, SkillsTab};
pub(crate) use rows::skill_row_view;
pub(crate) use views::SkillFactView;

pub(super) const SPARK_DAYS: i64 = 14;
const ROW_LIMIT: i64 = 500;

fn list_filter(query: &SkillsQuery, tab: SkillsTab) -> SkillListFilter {
    SkillListFilter {
        marketplace_id: query.marketplace().map(MarketplaceId::new),
        search: (tab == SkillsTab::Skills).then(|| query.search()).flatten(),
        client_kind: (tab == SkillsTab::Skills)
            .then(|| SkillsQuery::trimmed(query.client.as_deref()))
            .flatten(),
        sort: query.sort(),
        limit: ROW_LIMIT,
        offset: 0,
        skills: None,
    }
}

pub(crate) async fn page(
    shell: Page,
    Extension(boot): Extension<ExtensionBoot>,
    State(pool): State<Arc<PgPool>>,
    Extension(services): Extension<
        systemprompt_web_admin_core::extension_services::ExtensionServices,
    >,
    Query(query): Query<SkillsQuery>,
) -> AdminHtmlResult<Response> {
    if !shell.user.is_console {
        return Err(AdminError::Forbidden("Console access required".into()).into());
    }
    let tab = query.tab()?;
    let request = ScopeRequest::from_query(
        &shell.user,
        query.group.as_deref(),
        query.project.as_deref(),
    );
    let scope = systemprompt_web_admin_core::repositories::scope::membership::get_subject_scope(
        &pool, &request,
    )
    .await?;
    let end = Utc::now();
    let window = SkillWindow {
        start: end - Duration::days(query.days()),
        end,
        subject_ids: scope.as_sql().map(<[String]>::to_vec),
    };
    let filter = list_filter(&query, tab);
    let index = InventoryIndex::build(boot.services_root());
    let (rows, adoption, audience) = tokio::join!(
        list_skill_facts(&pool, &window, &filter),
        async {
            if tab == SkillsTab::Overview {
                list_marketplace_adoption(&pool, &window).await
            } else {
                Ok(Vec::new())
            }
        },
        get_marketplace_audience(&pool, &services, &index.marketplaces),
    );
    let (rows, adoption, audience) = (rows?, adoption?, audience?);
    let report_banner = crate::handlers::ssr::analysis::reports::banner::report_banner(
        &pool,
        "global",
        None,
        query
            .link(tab, None, None)
            .split_once('?')
            .map_or("", |(_, q)| q),
        "Skills in view",
    )
    .await;
    let context = build_context(
        &query,
        tab,
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
    Ok(
        systemprompt_web_admin_core::ssr::helpers::render_typed_page(
            &shell.engine,
            "analysis-skills",
            &context,
            &shell.user,
            &shell.marketplace,
        ),
    )
}
