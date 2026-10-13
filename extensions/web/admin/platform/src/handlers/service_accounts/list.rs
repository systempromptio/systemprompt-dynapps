//! The service-account listing and its create dialog.

use std::sync::Arc;

use axum::extract::{Extension, State};
use axum::response::Response;
use serde::Serialize;
use sqlx::PgPool;

use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::ssr::helpers::{Viewer, render_page_with_faults};
use systemprompt_web_admin_core::ssr::page::Page;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;

use super::{BASE_URL, CredentialView};
use crate::repositories::connectors::machine_credentials::list_machine_credentials;
use crate::services::service_accounts::ServiceAccounts;

#[derive(Debug, Serialize)]
struct AccountRow {
    name: String,
    email: String,
    created_at: String,
    href: String,
    credentials: Vec<CredentialView>,
}

#[derive(Debug, Serialize)]
struct ServiceAccountsPage {
    page: &'static str,
    title: &'static str,
    breadcrumbs: Vec<BreadcrumbView>,
    rows: Vec<AccountRow>,
    count_label: String,
    can_manage: bool,
}

pub(crate) async fn service_accounts_page(
    shell: Page,
    Extension(accounts): Extension<ServiceAccounts>,
    State(pool): State<Arc<PgPool>>,
) -> AdminHtmlResult<Response> {
    if !shell.user.is_admin {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    let mut faults = Faults::new();
    let accounts = faults.take("service accounts", accounts.list().await);
    let credentials = faults.take("machine credentials", list_machine_credentials(&pool).await);
    let rows: Vec<AccountRow> = accounts
        .into_iter()
        .map(|account| AccountRow {
            href: format!(
                "{BASE_URL}/{}",
                urlencoding::encode(account.user_id.as_str())
            ),
            credentials: credentials
                .iter()
                .filter(|c| c.user_id == account.user_id)
                .map(CredentialView::from)
                .collect(),
            name: account.name,
            email: account.email,
            created_at: account.created_at.format("%Y-%m-%d").to_string(),
        })
        .collect();
    let page = ServiceAccountsPage {
        page: "service-accounts",
        title: "Service accounts",
        breadcrumbs: vec![
            BreadcrumbView::link("Users", "/admin/users"),
            BreadcrumbView::current("Service accounts".to_owned()),
        ],
        count_label: format!("{} service accounts", rows.len()),
        rows,
        can_manage: shell.user.is_admin,
    };
    Ok(render_page_with_faults(
        &shell.engine,
        "service-accounts",
        &page,
        Viewer::new(&shell.user, &shell.marketplace),
        &faults,
    ))
}
