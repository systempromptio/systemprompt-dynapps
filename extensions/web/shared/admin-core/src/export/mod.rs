//! The shared half of CSV/JSON export: the [`model`] every dataset fills,
//! the [`format`] it is written in, the [`window`] it covers, the
//! [`registry`] datasets join with `inventory::submit!`, the [`handler`] that
//! serves any registered dataset, and the [`view`] a page hands its export
//! dialog. The datasets themselves and the conversation document export stay
//! with the domains that own the data.

pub mod format;
pub mod handler;
pub mod model;
pub mod registry;
pub mod transcripts;
pub mod view;
pub mod window;
