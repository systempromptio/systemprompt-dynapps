//! Pure view-model functions re-exported for the tests workspace, which
//! cannot reach `handlers` directly. Each page module exposes its pure row
//! and figure builders as `pub mod` and is listed here once.

pub mod people {
    pub use crate::handlers::ssr::farm_people::rows::{
        person_rows, share_pct, success_label, total_runs, trigger_split,
    };
}

pub mod status {
    pub use crate::handlers::ssr::shared::status::{is_live, status_badge, trigger_badge};
}

pub mod runs_query {
    pub use crate::handlers::ssr::farm_runs::query::{
        RunSortColumn, RunsQuery, run_filter, run_sort,
    };
}

pub mod curl {
    pub use crate::handlers::ssr::curl::{launch_body, remote_commands};
}

pub mod eval_history {
    pub use crate::handlers::ssr::shared::eval_history::{
        EvalRunItemView, EvalTrendRowView, eval_run_items, eval_trend_rows,
    };
}

pub mod eval_matrix {
    pub use crate::handlers::ssr::farm_evals::matrix::{eval_matrix, score_tone};
}

pub mod run_events {
    pub use crate::handlers::ssr::run_events_view::pairing::{PairKey, pair_seqs};
    pub use crate::handlers::ssr::run_events_view::text::FOLD_CHARS;
    pub use crate::handlers::ssr::run_events_view::timeline::{
        MIN_MARK_WIDTH, VIEW_WIDTH, scale, timeline_view,
    };
    pub use crate::handlers::ssr::run_events_view::{
        FRAGMENT_EVENT_CAP, RunEventView, event_views, events_after, transcript_views,
    };
}

pub mod sync_card {
    pub use crate::handlers::ssr::sync_card::{
        BundleBadge, BundleFacts, FarmSyncFacts, SyncState, bundle_for, commits_match, short_sha,
        sync_state,
    };
}
