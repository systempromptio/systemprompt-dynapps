//! Snapshot presence and provenance for HTML, without raw numeric aggregate
//! payloads.
use chrono::{DateTime, NaiveDate, Utc};
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use systemprompt::analytics::snapshots::{FeedbackSnapshot, SnapshotRangeJob};
use systemprompt::identifiers::ManagedResourceId;

#[derive(Serialize)]
pub(super) struct SnapshotMetadata<'a> {
    resource_id: Option<&'a ManagedResourceId>,
    generation: i64,
    fact_generation: i64,
    from_day: NaiveDate,
    to_day: NaiveDate,
    generated_at: DateTime<Utc>,
    related_spend_non_additive: bool,
    suppressed_days: i64,
    historical_identity_available: bool,
}
impl<'a> From<&'a FeedbackSnapshot> for SnapshotMetadata<'a> {
    fn from(snapshot: &'a FeedbackSnapshot) -> Self {
        Self {
            resource_id: snapshot.resource_id.as_ref(),
            generation: snapshot.generation,
            fact_generation: snapshot.fact_generation,
            from_day: snapshot.from_day,
            to_day: snapshot.to_day,
            generated_at: snapshot.generated_at,
            related_spend_non_additive: snapshot.related_spend_non_additive,
            suppressed_days: snapshot.suppressed_days,
            historical_identity_available: snapshot.historical_identity_available,
        }
    }
}

pub(super) struct HtmlRangeJob<'a>(pub(super) &'a SnapshotRangeJob);
impl Serialize for HtmlRangeJob<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut output = serializer.serialize_struct("HtmlRangeJob", 4)?;
        output.serialize_field("operation_id", &self.0.operation_id)?;
        output.serialize_field("state", &self.0.state)?;
        output.serialize_field("diagnostic", &self.0.diagnostic)?;
        output.serialize_field(
            "result",
            &self.0.result.as_ref().map(SnapshotMetadata::from),
        )?;
        output.end()
    }
}
