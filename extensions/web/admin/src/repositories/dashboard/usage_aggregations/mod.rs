//! Rollups derived from hook events as sessions progress.

mod daily;
pub(crate) mod ingestion;
mod session_titles;
pub(crate) mod session_updates;
mod statusline;

pub use daily::{DailyAggregationParams, refresh_session_summary, upsert_daily_aggregation};
pub use session_titles::update_session_title;
pub use session_updates::{
    update_session_metadata, update_session_permission_mode, update_session_title_if_empty,
};
pub use statusline::{
    SessionCostSnapshot, set_session_summary_tokens, upsert_session_cost_snapshot,
};
