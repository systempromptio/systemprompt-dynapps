//! The Analysis section: how the instance's skills, conversations and
//! marketplace versions perform, and what the judge makes of them.
//!
//! Every page lives at `/admin/analysis/<noun>` with the same identity in
//! the path as the Platform page for the same noun (`analysis_urls`).
pub(crate) mod conversations;
pub(crate) mod evaluation;
pub(crate) mod lifecycle;
pub(crate) mod marketplace_versions;
pub(crate) mod portfolio;
mod query;
pub(crate) mod revisions;
pub(crate) mod skill_detail;
pub(crate) mod time;
pub(crate) mod views;
pub use query::{AnalysisQuery, drilldown_url, parse_skill_key};
pub(crate) use skill_detail::skill_page;
pub use views::RetainedConversationView;
pub(crate) use views::dollars;
