//! `/admin/identity/service-accounts` — the non-interactive accounts workflow
//! write-backs act as, and the machine credential each holds per provider.
//!
//! Pages render here; the JSON mutations they post to live in [`api`]. A
//! secret is accepted on attach and rotate and never rendered or returned.

pub(crate) mod api;
mod detail;
mod list;

pub(crate) use detail::service_account_page;
pub(crate) use list::service_accounts_page;

use serde::Serialize;

use crate::repositories::connectors::machine_credentials::MachineCredentialSummary;
use crate::services::machine_credentials::MachineKind;

pub(crate) const BASE_URL: &str = "/admin/identity/service-accounts";

#[derive(Debug, Serialize)]
pub(crate) struct CredentialView {
    provider: String,
    kind_label: &'static str,
    identifiers: String,
    created_by: String,
    created_at: String,
    rotated_at: Option<String>,
}

impl From<&MachineCredentialSummary> for CredentialView {
    fn from(row: &MachineCredentialSummary) -> Self {
        let identifiers = row
            .config
            .as_object()
            .map(|map| {
                map.iter()
                    .filter_map(|(k, v)| v.as_str().map(|v| format!("{k}={v}")))
                    .collect::<Vec<_>>()
                    .join(" · ")
            })
            .unwrap_or_default();
        Self {
            provider: row.provider.clone(),
            kind_label: MachineKind::parse(&row.kind).map_or("Unknown", MachineKind::label),
            identifiers,
            created_by: row.created_by.clone(),
            created_at: row.created_at.format("%Y-%m-%d %H:%M").to_string(),
            rotated_at: row
                .rotated_at
                .map(|at| at.format("%Y-%m-%d %H:%M").to_string()),
        }
    }
}
