//! Super-admin publication, distribution, installation and rollback review.

mod review;
mod view;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::render_typed_page;
use crate::repositories::analysis::publications::list_publication_lifecycle;
use crate::routes::managed_state::ManagedState;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::extract::{Extension, State};
use axum::response::Response;
use serde::Serialize;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::marketplace::managed::{WithdrawalProposal, WithdrawalStatus};

pub(crate) use review::{decide_withdrawal, review};

#[derive(Serialize)]
struct LifecycleContext {
    page: &'static str,
    title: &'static str,
    publications: Vec<view::PublicationView>,
    kpis: view::PublicationKpis,
    withdrawals: Vec<WithdrawalProposal>,
    distributions: Vec<view::DistributionView>,
    receipts: Vec<view::ReceiptView>,
    can_manage: bool,
}

pub(crate) async fn page(
    Extension(user): Extension<UserContext>,
    Extension(marketplace): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    Extension(managed): Extension<Arc<ManagedState>>,
    State(pool): State<Arc<PgPool>>,
) -> AdminHtmlResult<Response> {
    if !user.is_admin {
        return Err(AdminError::Forbidden(
            "Administrator access required for shared publication evidence".to_owned(),
        )
        .into());
    }
    let rows = list_publication_lifecycle(&pool, managed.owner.as_str()).await?;
    let latest_generation = rows.iter().map(|row| row.generation).max().unwrap_or(0);
    let resources: std::collections::BTreeSet<&str> =
        rows.iter().map(|row| row.resource_id.as_str()).collect();
    let resource_count = resources.len();
    let publications = view::list_publication_views(rows);
    let withdrawals = managed
        .repository
        .list_withdrawal_proposals(&managed.owner)
        .await?;
    let distributions: Vec<view::DistributionView> = managed
        .repository
        .list_distribution_status(&managed.owner)
        .await?
        .into_iter()
        .map(view::distribution_view)
        .collect();
    let receipts: Vec<view::ReceiptView> = managed
        .repository
        .list_installation_receipts(&managed.owner, None)
        .await?
        .into_iter()
        .map(view::receipt_view)
        .collect();
    let kpis = view::PublicationKpis {
        resources: resource_count,
        latest_generation,
        pending_withdrawals: withdrawals
            .iter()
            .filter(|proposal| proposal.status == WithdrawalStatus::Pending)
            .count(),
        failed_deliveries: distributions
            .iter()
            .filter(|row| row.status == "failed")
            .count(),
        verified_receipts: receipts.iter().filter(|row| row.is_verified()).count(),
    };
    Ok(render_typed_page(
        &engine,
        "analysis-publications",
        &LifecycleContext {
            page: "analysis-publications",
            title: "Publication",
            publications,
            kpis,
            withdrawals,
            distributions,
            receipts,
            can_manage: user.is_admin,
        },
        &user,
        &marketplace,
    ))
}
