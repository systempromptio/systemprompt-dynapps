//! Database → `policies.yaml`: the rows rendered as the file the loader
//! reads.
//!
//! The header is regenerated with the export date; the long
//! operational commentary in the committed file is the operator's to keep
//! when they merge the result.

use chrono::{DateTime, Utc};
use systemprompt::gateway::{GatewayPolicyConfig, GatewayPolicyEntry};

use super::rows::PolicyRow;

fn header(exported_at: DateTime<Utc>) -> String {
    format!(
        "# Gateway policy baseline. Seeds `ai_gateway_policies` when the table is empty\n\
         # at boot (the governance_bootstrap job); otherwise the table is edited live at\n\
         # /admin/gateway/policies and /admin/sync compares the two and applies a\n\
         # direction on request.\n\
         #\n\
         # Exported from the database on {} by the console. A quota window with\n\
         # `calendar: month` resets on the UTC calendar month.\n\
         #\n\
         # Spec fields (systemprompt_ai::GatewayPolicySpec):\n\
         #   quota_mode, quota_windows, safety {{ mode, scanners, block_categories,\n\
         #                                       block_response_categories, history }}\n\n",
        exported_at.format("%Y-%m-%d")
    )
}

#[must_use]
pub fn render_policies_export(rows: &[PolicyRow], exported_at: DateTime<Utc>) -> String {
    let cfg = GatewayPolicyConfig {
        policies: rows
            .iter()
            .map(|r| GatewayPolicyEntry {
                name: r.name.clone(),
                enabled: r.enabled,
                priority: r.priority,
                spec: r.spec.clone(),
            })
            .collect(),
    };
    let body = serde_yaml::to_string(&cfg).unwrap_or_default();
    format!("{}{body}", header(exported_at))
}
