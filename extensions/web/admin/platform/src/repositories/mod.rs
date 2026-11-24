//! Data access owned by the platform domain.
//!
//! Device credentials and bridge sessions (`devices`), per-user connector
//! state (`connectors`),
//! the scheduled-jobs listing (`jobs`), service-account group membership
//! (`service_account_group`), the names drive pages show (`drive_people`), the
//! document class rows (`document_classes`), and the artifacts broker's audit
//! rows (`artifact_tool_calls`).
//!
//! Callers path-qualify (`repositories::devices::pats::…`); nothing is
//! re-exported, so the module path is the only name a symbol has.

pub mod artifact_tool_calls;
pub mod connectors;
pub mod devices;
pub mod document_classes;
pub mod drive_people;
pub mod jobs;
pub mod service_account_group;

pub mod data_console;
