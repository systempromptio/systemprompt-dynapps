//! Retained resource-wide figures for campaign baseline selection.
use crate::error::AdminError;
use crate::repositories::analysis::snapshot_figures::SnapshotFigures;
use crate::routes::managed_state::ManagedState;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
pub(super) struct ResourceOption {
    #[serde(flatten)]
    resource: systemprompt::marketplace::managed::ResourceSummary,
    value: String,
    selected: bool,
    measured: Option<MeasuredUsage>,
}

#[derive(Serialize, Clone)]
pub(super) struct MeasuredUsage {
    invocations: i64,
    users: Option<i64>,
    cost: String,
    p95: String,
    failed_requests: i64,
    linked_requests: i64,
}

impl MeasuredUsage {
    fn of(figures: &SnapshotFigures) -> Self {
        Self {
            invocations: figures.invocations,
            users: figures.distinct_users,
            cost: if figures.priced_requests > 0 {
                crate::handlers::ssr::format::format_cost(figures.cost_microdollars)
            } else {
                "—".to_owned()
            },
            p95: figures.p95_micros.map_or_else(
                || "—".to_owned(),
                |micros| {
                    format!(
                        "≤ {}",
                        crate::handlers::ssr::format::format_duration_ms((micros / 1000) as i64)
                    )
                },
            ),
            failed_requests: figures.failed_requests,
            linked_requests: figures.requests,
        }
    }
}

pub(super) async fn measured_by_resource(
    managed: &ManagedState,
    resources: &[systemprompt::marketplace::managed::ResourceSummary],
) -> Result<HashMap<String, MeasuredUsage>, AdminError> {
    let mut measured = HashMap::new();
    for resource in resources {
        if let Some(snapshot) = managed
            .snapshots
            .snapshot(&managed.owner, Some(&resource.id), 30)
            .await?
            && let Some(figures) = SnapshotFigures::of(&snapshot)?
        {
            measured.insert(resource.id.to_string(), MeasuredUsage::of(&figures));
        }
    }
    Ok(measured)
}

pub(super) fn resource_options(
    resources: &[systemprompt::marketplace::managed::ResourceSummary],
    measured: &HashMap<String, MeasuredUsage>,
    preselected: &str,
) -> Vec<ResourceOption> {
    resources
        .iter()
        .filter(|r| r.latest_revision.is_some())
        .map(|r| {
            let value = format!(
                "{}|{}",
                r.id,
                r.latest_revision.as_ref().map_or("", |v| v.as_str())
            );
            ResourceOption {
                selected: value == preselected,
                value,
                measured: measured.get(r.id.as_str()).cloned(),
                resource: r.clone(),
            }
        })
        .collect()
}
