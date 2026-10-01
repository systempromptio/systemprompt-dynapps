//! Library surface for exporting one configured marketplace as a portable kit.
//!
//! The CLI owns argument parsing and output-directory policy. This crate
//! exposes the export, the strict round-trip proof and the per-skill
//! `SKILL.md` projection so callers can test or embed the exact same
//! filesystem transformation.

mod export;
mod skill_md;
mod verify;

pub use export::{ExportReport, export_kit};
pub use skill_md::render as render_skill_md;
pub use verify::{Diff, round_trip};
