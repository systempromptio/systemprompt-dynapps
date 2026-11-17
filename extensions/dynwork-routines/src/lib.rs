//! `DynWork` routines, a systemprompt extension.
//!
//! Governance is the source of truth for each person's routine definitions:
//! every `DynWork` desktop of that person reads them from a change feed and
//! writes them with optimistic concurrency (`If-Match` on a revision). A
//! routine runs on the computer that created it, its owner device, which also
//! reports a summary of each run; the conversation of a run never leaves that
//! computer. After every write the user's other desktops get the relay's
//! change hint, `{"t":"routines.changed"}`.
//!
//! The extension also serves the "My routines" page, mounted by the admin
//! router ([`page::routes`]).

pub mod error;
pub mod extension;
pub mod http;
pub mod ids;
pub mod page;
pub mod router;
pub mod schema;
pub mod state;
pub mod store;
pub mod view;

pub use extension::RoutinesExtension;
pub use router::{BASE_PATH, routines_router};
pub use state::RoutinesState;

systemprompt::extension::register_extension!(RoutinesExtension);
