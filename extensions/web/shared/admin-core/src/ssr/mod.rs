//! The server-rendered page kit every admin domain renders through.
//!
//! [`page`] and [`context`] are the per-request values and the shell a page
//! renders inside; [`helpers`] hands a page's data to the template engine;
//! [`mod@format`], [`entity_urls`], [`analysis_urls`] and [`entity_kind`] are
//! the one spelling of each value and link a page shows; [`types`] holds the
//! shared view-models (breadcrumbs, tables, tabs, charts); [`time_window`]
//! parses and renders the window bounds a page carries in its query;
//! [`demo_help`] is the per-page help strip; [`sync_plane`] is the one
//! sync-plane card every owner page and the import preview render.
//! [`list_view`] is the filter, time-range and pagination contract every
//! filtered list page binds to; [`pickable_users`] is the person picker;
//! [`history_rows`] is one conversation-history row wherever it is listed;
//! [`tone`] is the one colour vocabulary for every figure; [`retention`] is
//! the retention-window block the lifecycle and configuration pages share;
//! [`faults`] is what a page could not load, drawn above its content, and
//! [`flash`] the one outcome notice a form post leaves on the page it returns
//! to; [`viewer_tz`] is the zone every wall-clock time renders in.

pub mod analysis_urls;
pub mod context;
pub mod core_plane;
pub mod demo_help;
pub mod entity_kind;
pub mod entity_panel;
pub mod entity_urls;
pub mod faults;
pub mod flash;
pub mod format;
pub mod helpers;
pub mod history_rows;
pub mod list_view;
pub mod page;
pub mod pickable_users;
pub mod retention;
pub mod sync_plane;
pub mod time_window;
pub mod tone;
pub mod types;
pub mod viewer_tz;
