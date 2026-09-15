//! Borrowed catalog grouping and display values; snapshot bodies remain on the
//! JSON API only.
use super::groups::{MarketplaceGroup, PluginGroup};
use super::html_snapshot::SnapshotMetadata;
use super::rows::PortfolioResourceRow;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

pub(super) struct HtmlMarketplaceGroup<'a>(pub(super) &'a MarketplaceGroup);
impl Serialize for HtmlMarketplaceGroup<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut output = serializer.serialize_struct("HtmlMarketplaceGroup", 5)?;
        output.serialize_field("marketplace_id", &self.0.marketplace_id)?;
        output.serialize_field("marketplace_name", &self.0.marketplace_name)?;
        output.serialize_field("users_with_access", &self.0.users_with_access)?;
        output.serialize_field(
            "plugins",
            &self
                .0
                .plugins
                .iter()
                .map(HtmlPluginGroup)
                .collect::<Vec<_>>(),
        )?;
        output.serialize_field("rollup", &self.0.rollup)?;
        output.end()
    }
}

pub(super) struct HtmlPluginGroup<'a>(pub(super) &'a PluginGroup);
impl Serialize for HtmlPluginGroup<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut output = serializer.serialize_struct("HtmlPluginGroup", 4)?;
        output.serialize_field("plugin_id", &self.0.plugin_id)?;
        output.serialize_field("plugin_name", &self.0.plugin_name)?;
        output.serialize_field(
            "rows",
            &self.0.rows.iter().map(HtmlResourceRow).collect::<Vec<_>>(),
        )?;
        output.serialize_field("rollup", &self.0.rollup)?;
        output.end()
    }
}

pub(super) struct HtmlResourceRow<'a>(pub(super) &'a PortfolioResourceRow);
impl Serialize for HtmlResourceRow<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut output = serializer.serialize_struct("HtmlResourceRow", 26)?;
        output.serialize_field("entry", &self.0.entry)?;
        output.serialize_field("resource_key", &self.0.resource_key)?;
        output.serialize_field("plugin_id", &self.0.plugin_id)?;
        output.serialize_field("plugin_name", &self.0.plugin_name)?;
        output.serialize_field("marketplaces", &self.0.marketplaces)?;
        output.serialize_field("catalog_href", &self.0.catalog_href)?;
        output.serialize_field("drilldown_href", &self.0.drilldown_href)?;
        output.serialize_field("campaign_href", &self.0.campaign_href)?;
        output.serialize_field("usage", &self.0.usage)?;
        output.serialize_field("users_with_access", &self.0.users_with_access)?;
        output.serialize_field("cost", &self.0.cost)?;
        output.serialize_field("p95", &self.0.p95)?;
        output.serialize_field("avg_latency", &self.0.avg_latency)?;
        output.serialize_field("tokens_in", &self.0.tokens_in)?;
        output.serialize_field("tokens_out", &self.0.tokens_out)?;
        output.serialize_field("tokens_cache", &self.0.tokens_cache)?;
        output.serialize_field("last_used", &self.0.last_used)?;
        output.serialize_field("attributed_pct", &self.0.attributed_pct)?;
        output.serialize_field(
            "snapshot",
            &self.0.snapshot.as_ref().map(SnapshotMetadata::from),
        )?;
        output.serialize_field("installation", &self.0.installation)?;
        output.serialize_field("health", &self.0.health)?;
        output.serialize_field("health_tone", &self.0.health_tone)?;
        output.serialize_field("health_reason", &self.0.health_reason)?;
        output.serialize_field("usage_status", &self.0.usage_status)?;
        output.serialize_field("bind_candidates", &self.0.bind_candidates)?;
        output.serialize_field("more_bind_candidates", &self.0.more_bind_candidates)?;
        output.end()
    }
}
