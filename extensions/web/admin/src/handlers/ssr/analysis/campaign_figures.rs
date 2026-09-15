//! Retained resource-wide figures for campaign baseline selection.
use crate::error::AdminError;
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
    fn of(row: &systemprompt::analytics::snapshots::FeedbackSnapshot) -> Self {
        Self {
            invocations: row.metrics.invocations,
            users: row.distinct_users,
            cost: crate::repositories::analysis::snapshot_figures::cost(row),
            p95: row
                .histogram
                .percentile_upper_bound_micros(95)
                .ok()
                .flatten()
                .map_or_else(
                    || "Unknown".to_owned(),
                    |us| {
                        crate::handlers::ssr::format::format_duration_ms(
                            i64::try_from(us.div_ceil(1000)).unwrap_or(i64::MAX),
                        )
                    },
                ),
            failed_requests: row.metrics.failed_requests,
            linked_requests: row.metrics.requests,
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
            && snapshot.suppressed_days == 0
        {
            measured.insert(resource.id.to_string(), MeasuredUsage::of(&snapshot));
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
