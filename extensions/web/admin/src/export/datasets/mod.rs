//! One file per exportable table. Each declares its columns, maps a
//! repository row to typed cells, and reads through the page module's own
//! `export_rows`, so the export never drifts from the page.

pub(crate) mod analysis_conversations;
pub(crate) mod analysis_skills;
pub(crate) mod artifacts;
pub(crate) mod contexts;
pub(crate) mod dashboard;
pub(crate) mod dashboard_cost;
pub(crate) mod evaluation;
pub(crate) mod governance;
pub(crate) mod history;
pub(crate) mod paged;
pub(crate) mod portfolio;
pub(crate) mod reports;
pub(crate) mod requests;
pub(crate) mod sessions;
pub(crate) mod traces;
pub(crate) mod versions;
