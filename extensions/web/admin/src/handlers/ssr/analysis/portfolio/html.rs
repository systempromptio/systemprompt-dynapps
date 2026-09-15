//! Borrowed HTML page view avoids converting raw wide snapshot integers into
//! JSON Values.
use super::PortfolioPage;
use super::html_rows::HtmlMarketplaceGroup;
use super::html_snapshot::{HtmlRangeJob, SnapshotMetadata};
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

pub(super) struct HtmlPortfolioPage<'a>(pub(super) &'a PortfolioPage);
impl Serialize for HtmlPortfolioPage<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut output = serializer.serialize_struct("HtmlPortfolioPage", 27)?;
        output.serialize_field("page", &self.0.page)?;
        output.serialize_field("title", &self.0.title)?;
        output.serialize_field("kind", &self.0.kind)?;
        output.serialize_field(
            "groups",
            &self
                .0
                .groups
                .iter()
                .map(HtmlMarketplaceGroup)
                .collect::<Vec<_>>(),
        )?;
        output.serialize_field("kpis", &self.0.kpis)?;
        output.serialize_field("marketplaces", &self.0.marketplaces)?;
        output.serialize_field("user_groups", &self.0.user_groups)?;
        output.serialize_field("filter_marketplace", &self.0.filter_marketplace)?;
        output.serialize_field("filter_group", &self.0.filter_group)?;
        output.serialize_field("start", &self.0.start)?;
        output.serialize_field("end", &self.0.end)?;
        output.serialize_field("search", &self.0.search)?;
        output.serialize_field("days", &self.0.days)?;
        output.serialize_field("generated_at", &self.0.generated_at)?;
        output.serialize_field(
            "totals",
            &self.0.totals.as_ref().map(SnapshotMetadata::from),
        )?;
        output.serialize_field("health", &self.0.health)?;
        output.serialize_field("inventory", &self.0.inventory)?;
        output.serialize_field("installations", &self.0.installations)?;
        output.serialize_field("pending_jobs", &self.0.pending_jobs)?;
        output.serialize_field(
            "range_jobs",
            &self
                .0
                .range_jobs
                .iter()
                .map(HtmlRangeJob)
                .collect::<Vec<_>>(),
        )?;
        output.serialize_field("unknown_usage", &self.0.unknown_usage)?;
        output.serialize_field("unbound", &self.0.unbound)?;
        output.serialize_field("conflicting", &self.0.conflicting)?;
        output.serialize_field("operation_key", &self.0.operation_key)?;
        output.serialize_field("next_cursor", &self.0.next_cursor)?;
        output.serialize_field("next_href", &self.0.next_href)?;
        output.serialize_field("metrics_notice", &self.0.metrics_notice)?;
        output.end()
    }
}

#[cfg(test)]
#[path = "html_tests.rs"]
mod tests;
