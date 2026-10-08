//! Rollups derived from hook events as sessions progress.

mod daily;
mod session_titles;
pub(crate) mod session_updates;

pub use daily::{DailyAggregationParams, upsert_daily_aggregation};
pub use session_titles::update_session_title;
pub use session_updates::{
    update_session_metadata, update_session_permission_mode, update_session_title_if_empty,
    update_unique_files_touched,
};
