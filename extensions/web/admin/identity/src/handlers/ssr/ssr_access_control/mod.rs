//! `/admin/access-control` — every governed entity, who reaches it and why,
//! and whether the code agrees with the database.
//!
//! Three tabs, one kind of answer each, and a link. *Rules*: how a decision
//! is made (the ladder) and the entities table, grouped by kind. *Audience
//! grid*: every role, group and project against every entity, resolved by
//! the real resolver. *Find a person*: a search that links to that person's
//! own Access tab. *Sync* is a link to Code sync's Access review, carrying the
//! count still waiting on a person — the plane lives there alone. Editing
//! lives where it is shown: an entity's rules in its "Who gets this" panel, a
//! group's band on its Access tab, a person's overrides on theirs. Reading
//! here is the CONSOLE tier so a project manager can audit.
//!
//! Each tab builds only what it shows: the audience grid runs the resolver
//! per entity and the person tab per account. An unreadable declaration is
//! reported on the page, and unreadable route labels are logged, so neither
//! turns the audit page into a 500.

mod audience;
mod audience_filter;
mod audience_focus;
mod audience_rows;
mod bands;
mod data;
pub(crate) mod entities;
mod entity_row;
mod page;
mod person;
mod summary;
mod view;

use std::sync::Arc;
use systemprompt_web_admin_core::ssr::page::Page;

use axum::extract::{Extension, Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use sqlx::PgPool;

use crate::handlers::access_control::build_matrix_sections;
use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::repositories::access_rules::drift::DriftReport;
use systemprompt_web_admin_core::repositories::access_rules::rules::{
    RULE_CAP, count_open_entities, list_ledger_rules,
};
use systemprompt_web_admin_core::repositories::gateway_catalog::{RouteLabels, get_route_labels};
use systemprompt_web_admin_core::repositories::sync::access_control::{declared_now, drift_now};
use systemprompt_web_admin_core::repositories::sync::attention::access_attention;
use systemprompt_web_admin_core::request as shared;
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::ssr::helpers::Viewer;
use systemprompt_web_admin_core::ssr::types::tabs::TabLinkView;

use audience::GridRead;
use entities::{AcQuery, BASE_URL, EntitiesInput, SYNC_URL};
use page::AcPageParts;
use person::PersonCheckView;
use view::AudienceGridView;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AcTab {
    Rules,
    Audience,
    Person,
}

impl AcTab {
    fn parse(tab: Option<&str>) -> Self {
        match tab {
            Some("audience") => Self::Audience,
            Some("person") => Self::Person,
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
        link("person", "Find a person", PERSON_URL, AcTab::Person),
        TabLinkView {
            slug: "sync",
            label: "Sync",
            href: SYNC_URL.to_owned(),
            is_active: false,
            count: i64::try_from(access_attention()).ok().filter(|n| *n > 0),
        },
    ]
}

#[derive(Default)]
struct TabBody {
    audience: AudienceGridView,
    audiences: usize,
    person: Option<PersonCheckView>,
}

fn route_labels(
    services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    faults: &mut Faults,
) -> RouteLabels {
    faults.take("route labels", get_route_labels(&services.config()))
}

async fn declared_drift(
    services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    pool: &PgPool,
) -> (Option<DriftReport>, usize, Option<String>) {
    match declared_now(&services.config()).await {
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
    read: GridRead<'_>,
    faults: &mut Faults,
) -> AdminHtmlResult<TabBody> {
    let mut body = TabBody::default();
    match tab {
        AcTab::Audience => {
            let audience = audience::audience(pool, faults).await;
            body.audiences = audience.columns.len();
            let sections = build_matrix_sections(read.services, services_path);
            body.audience = audience::grid(pool, audience, &sections, read, faults).await;
        },
        AcTab::Person => {
            body.person = Some(person::build(pool, read.query.q.as_deref(), faults).await);
        },
        AcTab::Rules => {
            body.audiences = audience::audience(pool, faults).await.columns.len();
        },
    }
    Ok(body)
}

fn legacy_link_redirect(query: &AcQuery) -> Option<Redirect> {
    if query.tab.as_deref() == Some("sync") {
        return Some(Redirect::to(SYNC_URL));
    }
    query.user.as_deref().filter(|u| !u.is_empty()).map(|user| {
        Redirect::to(&format!(
            "/admin/users/{}?tab=access",
            urlencoding::encode(user)
        ))
    })
}

pub(crate) async fn access_control_page(
    shell: Page,
    Extension(services): Extension<
        systemprompt_web_admin_core::extension_services::ExtensionServices,
    >,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<AcQuery>,
) -> AdminHtmlResult<Response> {
    let Page {
        engine,
        user: user_ctx,
        marketplace: mkt_ctx,
    } = shell;
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    if let Some(redirect) = legacy_link_redirect(&query) {
        return Ok(redirect.into_response());
    }
    let tab = AcTab::parse(query.tab.as_deref());
    let services_path = shared::get_services_path()?;

    let mut faults = Faults::new();
    let stats = data::load_stats(&pool, &mut faults).await;
    let ledger_rows = faults.take("access rules", list_ledger_rules(&pool).await);
    let open_entities = count_open_entities(&pool).await?;

    let (drift, declared_count, declared_unreadable) = declared_drift(&services, &pool).await;
    let body = tab_body(
        &pool,
        &services_path,
        tab,
        GridRead {
            services: &services,
            query: &query,
            ledger: &ledger_rows,
        },
        &mut faults,
    )
    .await?;
    let labels = route_labels(&services, &mut faults);

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

    let page = page::page_data(AcPageParts {
        tab,
        can_write: user_ctx.is_admin,
        stats,
        drift: page::drift_banner(drift.as_ref(), declared_count),
        declared_unreadable,
        entities,
        body,
    });

    Ok(
        systemprompt_web_admin_core::ssr::helpers::render_page_with_faults(
            &engine,
            "access-control",
            &page,
            Viewer::new(&user_ctx, &mkt_ctx),
            &faults,
        ),
    )
}
