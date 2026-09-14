//! Custom UTC ranges use retained, idempotent asynchronous operations.

use crate::error::{AdminError, AdminResult};
use crate::routes::managed_state::ManagedState;
use chrono::{DateTime, NaiveDate, Utc};
use systemprompt::analytics::snapshots::{FeedbackSnapshot, SnapshotRangeRequest};
use systemprompt::identifiers::{ManagedResourceId, TaskId};

pub(super) struct Range {
    pub(super) from: NaiveDate,
    pub(super) to: NaiveDate,
    pub(super) window: u32,
    pub(super) custom: bool,
}
impl Range {
    pub(super) fn parse(query: &super::PortfolioQuery, today: NaiveDate) -> AdminResult<Self> {
        let window = query.days.unwrap_or(30);
        if ![1, 7, 30, 90, 365].contains(&window) {
            return Err(AdminError::BadRequest(
                "Unsupported snapshot window".to_owned(),
            ));
        }
        let to = query
            .end
            .as_deref()
            .map(date)
            .transpose()?
            .unwrap_or(today + chrono::Duration::days(1));
        let from = query
            .start
            .as_deref()
            .map(date)
            .transpose()?
            .unwrap_or(to - chrono::Duration::days(i64::from(window)));
        let days = (to - from).num_days();
        if !(1..=365).contains(&days)
            || from < today - chrono::Duration::days(364)
            || to > today + chrono::Duration::days(1)
        {
            return Err(AdminError::BadRequest(
                "Range must contain1–365 UTC days within retained history".to_owned(),
            ));
        }
        let standard =
            to == today + chrono::Duration::days(1) && [1, 7, 30, 90, 365].contains(&days);
        let window = if standard {
            u32::try_from(days).map_err(|error| AdminError::BadRequest(error.to_string()))?
        } else {
            30
        };
        Ok(Self {
            from,
            to,
            window,
            custom: !standard,
        })
    }
}
fn date(value: &str) -> AdminResult<NaiveDate> {
    value
        .parse::<NaiveDate>()
        .or_else(|_error| value.parse::<DateTime<Utc>>().map(|date| date.date_naive()))
        .map_err(|_error| AdminError::BadRequest("Dates must identify UTC days".to_owned()))
}
pub(super) async fn custom_snapshot(
    managed: &ManagedState,
    resource: Option<&ManagedResourceId>,
    range: &Range,
    now: DateTime<Utc>,
) -> AdminResult<Option<FeedbackSnapshot>> {
    let identity = format!(
        "{}:{}:{}:{}",
        managed.owner,
        resource.map_or("", ManagedResourceId::as_str),
        range.from,
        range.to
    );
    let operation_id = TaskId::new(format!(
        "portfolio-{}",
        systemprompt::marketplace::managed::AssetDigest::of(identity.as_bytes())
    ));
    let job = managed
        .snapshots
        .request_range(
            &managed.owner,
            &SnapshotRangeRequest {
                operation_id,
                resource_id: resource.cloned(),
                from_day: range.from,
                to_day: range.to,
            },
            now,
        )
        .await?;
    Ok(job.result)
}
