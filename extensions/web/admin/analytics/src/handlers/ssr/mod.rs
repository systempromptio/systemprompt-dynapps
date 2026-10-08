//! Server-rendered admin pages.
//!
//! Each module owns one page: it builds a typed template context and renders a
//! `.hbs` template from `storage/files/admin/templates/` at request time.

mod attention;
pub(crate) mod conversation_header;
mod overview;
pub(crate) mod ssr_analytics_dashboard;
pub(crate) mod ssr_analytics_requests;
pub(crate) mod ssr_artifacts;
mod ssr_context_detail;
pub(crate) mod ssr_history;
mod ssr_observability;
mod ssr_perf_trace_detail;
pub(crate) mod ssr_perf_traces;
mod ssr_session_detail;
pub(crate) mod ssr_sessions_list;
pub(crate) mod ssr_skills_contexts;
pub(crate) mod ssr_tools;
pub(crate) mod transcript_view;
pub(crate) mod types;

pub(crate) use attention::attention_counts;
pub(crate) use overview::{overview_fragment, overview_page};
pub(crate) use ssr_analytics_dashboard::analytics_dashboard_page;
pub(crate) use ssr_analytics_requests::analytics_requests_page;
pub(crate) use ssr_artifacts::{artifact_detail_page, artifact_preview, artifacts_page};
pub(crate) use ssr_context_detail::context_detail_page;
pub(crate) use ssr_history::{
    conversations_page, history_conversation_page, history_page, history_search,
};
pub(crate) use ssr_observability::{
    export_now as observability_export_now, observability_page,
    test_connection as observability_test_connection,
};
pub(crate) use ssr_perf_trace_detail::perf_trace_detail_page;
pub(crate) use ssr_perf_traces::perf_traces_page;
pub(crate) use ssr_session_detail::session_detail_page;
pub(crate) use ssr_sessions_list::sessions_list_page;
pub(crate) use ssr_skills_contexts::skills_contexts_page;
pub(crate) use ssr_tools::tools_page;

pub(crate) mod analysis;
