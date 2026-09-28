//! Template context types for the SSR pages.

mod charts;
mod groups;
mod groups_listing;
mod project_report;
mod projects_page;
mod users;

pub(crate) use charts::*;
pub(crate) use groups::*;
pub(crate) use groups_listing::*;

pub(crate) use project_report::*;
pub(crate) use projects_page::*;

pub(crate) use users::*;
