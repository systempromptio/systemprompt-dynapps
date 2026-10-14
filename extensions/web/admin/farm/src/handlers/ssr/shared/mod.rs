//! View pieces every farm page shares: status and trigger badges, the run
//! table row, the eval history rows, and the small formatting helpers the
//! pages compose them from.

pub(crate) mod eval_history;
pub(crate) mod run_row;
pub(crate) mod status;

pub(crate) use eval_history::{EvalRunItemView, EvalTrendRowView, eval_run_items, eval_trend_rows};
pub(crate) use run_row::{RunRowView, run_row_views};
pub(crate) use status::{BadgeView, is_live, status_badge, trigger_badge};
