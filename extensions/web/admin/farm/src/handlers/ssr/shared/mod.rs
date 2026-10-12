//! View pieces every farm page shares: status and trigger badges, the run
//! table row, and the small formatting helpers the pages compose them from.

pub(crate) mod run_row;
pub(crate) mod status;

pub(crate) use run_row::{RunRowView, run_row_views};
pub(crate) use status::{BadgeView, is_live, status_badge, trigger_badge};
