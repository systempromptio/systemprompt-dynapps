//! `DynWork` cloud chats, a systemprompt extension.
//!
//! Chat threads are stored, and answered, in the user's own systemprompt
//! instance, so `DynWork` Mobile and every `DynWork` desktop read the same
//! threads whether a desktop is on or not. The crate holds the store (threads,
//! messages and the live events of a running reply, behind one trait with a
//! memory and a Postgres implementation) and the HTTP API under
//! `/api/dynwork/chats/v1`, authenticated with the desktop or the mobile JWT:
//! the threads, the changes feed and the message history. Replies, the `/log`
//! stream and the models list come in later phases of the plan.

pub mod auth;
pub mod clock;
pub mod error;
pub mod extension;
pub mod http;
pub mod ids;
pub mod router;
pub mod schema;
pub mod state;
pub mod store;

pub use extension::ChatsExtension;
pub use router::{EXTENSION_BASE_PATH, chats_router};
pub use state::{ChatsState, Shared};

systemprompt::extension::register_extension!(ChatsExtension);
