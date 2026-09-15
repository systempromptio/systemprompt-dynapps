//! Explicit binding and snapshot display helpers for one inventory row.
use super::rows::{Health, HealthInputs, health_of};
use crate::handlers::ssr::format::format_duration_ms;
use crate::repositories::analysis::snapshot_figures::SnapshotFigures;
use systemprompt::analytics::snapshots::FeedbackSnapshot;
use systemprompt::marketplace::inventory::InventoryEntry;
use systemprompt::marketplace::managed::ResourceSummary;
use systemprompt::models::feedback::inventory::InventoryAvailability;

/// How a configured entry relates to the managed resources that share its
/// name: the one it is bound to (or would bind to), and the candidates an
/// unbound entry can still choose between.
pub(super) struct Binding {
    pub(super) campaign_href: Option<String>,
    pub(super) candidates: Vec<ResourceSummary>,
    pub(super) more: bool,
}

pub(super) fn binding(entry: &InventoryEntry, managed: &[ResourceSummary]) -> Binding {
    let same_key =
        |r: &&ResourceSummary| r.kind == entry.kind && r.resource_key == entry.resource_key;
    let bound = entry
        .resource_id
        .as_ref()
        .and_then(|id| managed.iter().find(|r| &r.id == id));
    Binding {
        more: entry.resource_id.is_none() && managed.iter().filter(same_key).count() > 50,
        campaign_href: entry
            .resource_id
            .as_ref()
            .zip(entry.latest_revision_id.as_ref())
            .map(|(id, rev)| format!("/admin/analysis/campaigns?baseline={id}|{rev}"))
            .or_else(|| {
                bound.and_then(|r| {
                    r.latest_revision
                        .as_ref()
                        .map(|rev| format!("/admin/analysis/campaigns?baseline={}|{}", r.id, rev))
                })
            }),
        candidates: if entry.resource_id.is_none() {
            managed.iter().filter(same_key).take(50).cloned().collect()
        } else {
            Vec::new()
        },
    }
}

pub(super) fn health(
    entry: &InventoryEntry,
    usage: Option<&SnapshotFigures>,
    users_with_access: Option<usize>,
) -> (Health, &'static str) {
    let (health, health_reason) = health_of(&HealthInputs {
        availability: entry.availability,
        bound: entry.resource_id.is_some(),
        invocations: usage.map_or(0, |u| u.invocations),
        linked_requests: usage.map_or(0, |u| u.linked_requests),
        failed_requests: usage.map_or(0, |u| u.failed_requests),
        users_with_access,
    });
    let (health, health_reason) =
        if usage.is_none() && entry.availability == InventoryAvailability::Available {
            (
                Health::Muted,
                if entry.resource_id.is_none() {
                    "unbound"
                } else {
                    "usage unknown"
                },
            )
        } else {
            (health, health_reason)
        };
    (health, health_reason)
}
pub(super) fn attribution(snapshot: Option<&FeedbackSnapshot>) -> Option<i64> {
    snapshot.filter(|s| s.suppressed_days == 0).and_then(|s| {
        (s.metrics.invocations > 0)
            .then(|| {
                i128::from(s.metrics.verified_invocations) * 100 / i128::from(s.metrics.invocations)
            })
            .and_then(|percent| u8::try_from(percent).ok())
            .filter(|percent| *percent <= 100)
            .map(i64::from)
    })
}
pub(super) fn p95(snapshot: Option<&FeedbackSnapshot>) -> String {
    snapshot
        .filter(|s| s.suppressed_days == 0)
        .and_then(|s| s.histogram.percentile_upper_bound_micros(95).ok().flatten())
        .map_or_else(
            || "Unknown".to_owned(),
            |us| format_duration_ms(i64::try_from(us.div_ceil(1000)).unwrap_or(i64::MAX)),
        )
}

pub(super) fn tokens(snapshot: Option<&FeedbackSnapshot>, input: bool) -> String {
    snapshot
        .filter(|s| s.suppressed_days == 0 && s.metrics.token_measured_requests > 0)
        .map_or_else(
            || "Unknown".to_owned(),
            |s| {
                (if input {
                    s.metrics.input_tokens
                } else {
                    s.metrics.output_tokens
                })
                .to_string()
            },
        )
}
