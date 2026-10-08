//! Conversations & history data layer.
//!
//! The `/admin/history` listing lives in `unified`, which reads the gateway's
//! own `ai_requests` rows grouped by context; `redact` masks secrets and PII
//! before any body reaches a page, and `scope` decides whose conversations a
//! viewer may see. [`rows`] is the per-user conversation roll-up the people
//! and analytics pages page through.

mod gateway_text;
mod redact;
pub mod rows;
pub mod scope;
mod unified;

pub use gateway_text::strip_gateway_markers;
pub use redact::redact_text;
pub use scope::{
    HistoryScope, has_full_history_view, history_scope_for, own_history_scope,
    resolve_history_scope,
};
pub use unified::{HistoryFilter, HistoryItem, list_history_items};
