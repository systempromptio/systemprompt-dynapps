//! View-models shared by every server-rendered admin page.
//!
//! Breadcrumbs, sortable tables, tab strips, the person picker, and the SVG
//! pie, line and sparkline charts. Callers path-qualify; nothing is re-exported
//! here.

pub mod breadcrumb;
pub mod pickable_user;
pub mod pie;
pub mod sparkline;
pub mod svg_line;
pub mod table;
pub mod tabs;
