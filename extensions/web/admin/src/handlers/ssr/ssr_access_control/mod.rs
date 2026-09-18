//! `/admin/access-control` — every governed entity, who reaches it and why,
//! and whether the code agrees with the database.
//!
//! Four tabs, one kind of answer each. *Rules*: how a decision is made (the
//! ladder) and the entities table, grouped by kind. *Audience grid*: every
//! role, group and project against every entity, resolved by the real
//! resolver. *Check a person*: one account's resolved access, the same view
//! as that person's own Access tab. *Sync*: the `access_control` plane —
//! `rules.yaml` against this database, with the three directions. Editing
//! lives with the subject — a group's band on its Access tab, a person's
//! overrides on theirs. Reading here is the CONSOLE tier so a project
//! manager can audit.

mod audience;
mod bands;
mod data;
pub(crate) mod entities;
mod person;
mod summary;
mod view;

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::access_control::build_matrix_sections;
use crate::handlers::shared;
use crate::handlers::ssr::ssr_users::detail::AccessTabView;
use crate::handlers::ssr::sync_plane::{PlaneCardView, is_sync_tab, plane_card_by_id, sync_tab};
use crate::handlers::ssr::types::{BreadcrumbView, TabLinkView};
use crate::repositories::access_control::drift::DriftReport;
use crate::repositories::access_control::rules::{
    RULE_CAP, count_open_entities, list_ledger_rules,
};
use crate::repositories::config::gateway::{RouteLabels, get_route_labels_from_services};
use crate::repositories::sync::access_control::{declared_now, drift_now};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use entities::{AcQuery, BASE_URL, DOCS_URL, EntitiesInput, SYNC_URL};
use person::PersonCheckView;
use view::{AccessControlPageData, AudienceGridView, DriftBannerView};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AcTab {
    Rules,
    Audience,
    Person,
    Sync,
}

impl AcTab {
    fn parse(tab: Option<&str>) -> Self {
        match tab {
            Some("audience") => Self::Audience,
            Some("person") => Self::Person,
            t if is_sync_tab(t) => Self::Sync,
            _ => Self::Rules,
        }
    }
}

const AUDIENCE_URL: &str = "/admin/access-control?tab=audience";
pub(crate) const PERSON_URL: &str = "/admin/access-control?tab=person";

fn tabs(active: AcTab) -> Vec<TabLinkView> {
    let link = |slug, label, href: &str, tab| TabLinkView {
        slug,
        label,
        href: href.to_owned(),
        is_active: active == tab,
        count: None,
    };
    vec![
        link("rules", "Rules", BASE_URL, AcTab::Rules),
        link("audience", "Audience grid", AUDIENCE_URL, AcTab::Audience),
        link("person", "Check a person", PERSON_URL, AcTab::Person),
        sync_tab(BASE_URL, active == AcTab::Sync),
    ]
}

// Why: each tab pays only for what it shows. The audience grid runs the real
// resolver per entity and the person tab per account, and neither is built
// for a tab that never renders it.
#[derive(Default)]
struct TabBody {
    audience: AudienceGridView,
    audiences: usize,
    sync: Option<PlaneCardView>,
    person: Option<PersonCheckView>,
    access: Option<AccessTabView>,
}

// Why: labels are a courtesy, not a gate — an unreadable gateway file still
// leaves every route addressable by id, and is logged rather than 500ing
// the audit page.
fn route_labels() -> RouteLabels {
    get_route_labels_from_services()
        .inspect_err(|e| tracing::warn!(error = %e, "access-control: route labels unavailable"))
        .unwrap_or_default()
}

// Why: the file failing to load is reported on the page, not as a 500.
// The rules that ARE enforced are still worth showing while the operator
// fixes the declaration.
async fn declared_drift(pool: &PgPool) -> (Option<DriftReport>, usize, Option<String>) {
    match declared_now().await {
        Ok(declared) => match drift_now(pool, &declared).await {
            Ok(drift) => (Some(drift), declared.rule_count(), None),
            Err(e) => (None, declared.rule_count(), Some(e.to_string())),
        },
        Err(e) => (None, 0, Some(e.to_string())),
    }
}

async fn tab_body(
    pool: &PgPool,
    services_path: &std::path::Path,
    tab: AcTab,
    query: &AcQuery,
) -> AdminHtmlResult<TabBody> {
    let mut body = TabBody::default();
    match tab {
        AcTab::Sync => {
            let filter = query.entity.clone().unwrap_or_default();
            body.sync = plane_card_by_id(
                pool,
                crate::repositories::sync::access_control::PLANE_ID,
                &filter,
            )
            .await?;
        },
        AcTab::Audience => {
            let audience = audience::audience(pool).await;
            body.audiences = audience.columns.len();
            let sections = build_matrix_sections(services_path);
            body.audience = audience::grid(pool, audience, &sections).await;
        },
        AcTab::Person => {
            let selected = query.user.as_deref().filter(|u| !u.is_empty());
            let (person, access) = person::build(pool, selected).await;
            body.person = Some(person);
            body.access = access;
        },
        AcTab::Rules => {
            body.audiences = audience::audience(pool).await.columns.len();
        },
    }
    Ok(body)
}

pub(crate) async fn access_control_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<AcQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    let tab = AcTab::parse(query.tab.as_deref());
    // Why: the editor's old deep link, `?user=` with no tab, lands on the
    // person tab rather than bouncing to the user page — a temporary
    // redirect, since the old one was permanent and browsers cache those.
    if tab != AcTab::Person
        && let Some(user) = query.user.as_deref().filter(|u| !u.is_empty())
    {
        let target = format!("{PERSON_URL}&user={}", urlencoding::encode(user));
        return Ok(Redirect::to(&target).into_response());
    }
    let services_path = shared::get_services_path()?;

    let stats = data::load_stats(&pool).await;
    let ledger_rows = list_ledger_rules(&pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "access-control: rule listing failed"))
        .unwrap_or_default();
    let open_entities = count_open_entities(&pool).await?;

    let (drift, declared_count, declared_unreadable) = declared_drift(&pool).await;
    let body = tab_body(&pool, &services_path, tab, &query).await?;
    let labels = route_labels();

    let capped = i64::try_from(ledger_rows.len()).unwrap_or(i64::MAX) >= RULE_CAP;
    let entities = entities::build(
        &EntitiesInput {
            rows: &ledger_rows,
            drift: drift.as_ref(),
            open_entities,
            audiences: body.audiences,
            capped,
            labels: &labels,
        },
        &query,
    );

    let banner = drift
        .as_ref()
        .filter(|d| !d.is_clean())
        .map(|d| DriftBannerView {
            counts: d.counts(),
            url: SYNC_URL,
            declared: declared_count,
        });

    let page = AccessControlPageData {
        page: "access-control",
        title: "Access control",
        can_write: user_ctx.is_admin,
        stats,
        breadcrumbs: vec![
            BreadcrumbView::link("Admin", "/admin"),
            BreadcrumbView::link("People & access", "/admin/users"),
            BreadcrumbView::current("Access control"),
        ],
        tabs: tabs(tab),
        on_rules: tab == AcTab::Rules,
        on_audience: tab == AcTab::Audience,
        on_person: tab == AcTab::Person,
        sync: body.sync,
        docs_url: DOCS_URL,
        sync_url: SYNC_URL,
        drift: banner,
        declared_unreadable,
        entities,
        audience: body.audience,
        person: body.person,
        access: body.access,
    };

    Ok(super::render_typed_page(
        &engine,
        "access-control",
        &page,
        &user_ctx,
        &mkt_ctx,
    ))
}
