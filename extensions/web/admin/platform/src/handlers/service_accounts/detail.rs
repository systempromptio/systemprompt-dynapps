//! One service account: its credentials by provider, the attach and rotate
//! forms (secret fields write-only), and the delete action.

use std::sync::Arc;

use axum::extract::{Extension, Path, State};
use axum::response::Response;
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::ssr::helpers::{Viewer, render_page_with_faults};
use systemprompt_web_admin_core::ssr::page::Page;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;

use super::{BASE_URL, CredentialView};
use crate::repositories::connectors::machine_credentials::list_machine_credentials_for_user;
use crate::services::machine_credentials::MachineKind;
use crate::services::service_accounts::ServiceAccounts;

#[derive(Debug, Serialize)]
struct KindOption {
    value: &'static str,
    label: &'static str,
}

#[derive(Debug, Serialize)]
struct ServiceAccountPage {
    page: &'static str,
    title: String,
    breadcrumbs: Vec<BreadcrumbView>,
    user_id: UserId,
    email: String,
    created_at: String,
    credentials: Vec<CredentialView>,
    has_credentials: bool,
    kinds: Vec<KindOption>,
}

pub(crate) async fn service_account_page(
    shell: Page,
    Extension(accounts): Extension<ServiceAccounts>,
    State(pool): State<Arc<PgPool>>,
    Path(user_id): Path<UserId>,
) -> AdminHtmlResult<Response> {
    if !shell.user.is_admin {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    let account = accounts
        .find(&user_id)
        .await?
        .ok_or_else(|| AdminError::NotFound("No such service account.".to_owned()))?;
    let mut faults = Faults::new();
    let credentials: Vec<CredentialView> = faults
        .take(
            "machine credentials",
            list_machine_credentials_for_user(&pool, &user_id).await,
        )
        .iter()
        .map(CredentialView::from)
        .collect();
    let page = ServiceAccountPage {
        page: "service-accounts",
        breadcrumbs: vec![
            BreadcrumbView::link("Service accounts", BASE_URL),
            BreadcrumbView::current(account.name.clone()),
        ],
        title: account.name,
        user_id: user_id.clone(),
        email: account.email,
        created_at: account.created_at.format("%Y-%m-%d %H:%M").to_string(),
        has_credentials: !credentials.is_empty(),
        credentials,
        kinds: MachineKind::ALL
            .into_iter()
            .map(|kind| KindOption {
                value: kind.as_str(),
                label: kind.label(),
            })
            .collect(),
    };
    Ok(render_page_with_faults(
        &shell.engine,
        "service-account-detail",
        &page,
        Viewer::new(&shell.user, &shell.marketplace),
        &faults,
    ))
}
