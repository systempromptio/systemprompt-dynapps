//! One file per exportable table.
//!
//! Each declares its columns, maps a
//! repository row to typed cells, and reads through the same repository call
//! the page makes, so the export never drifts from the page.

pub(crate) mod people;
pub(crate) mod projects;
