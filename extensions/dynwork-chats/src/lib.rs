//! `DynWork` cloud chats, a systemprompt extension.
//!
//! Chat threads are stored, and answered, in the user's own systemprompt
//! instance, so `DynWork` Mobile and every `DynWork` desktop read the same
//! threads whether a desktop is on or not. The crate holds the store (threads,
//! messages and the live events of a running reply, behind one trait with a
//! memory and a Postgres implementation) and the HTTP API under
//! `/api/dynwork/chats/v1`, authenticated with the desktop or the mobile JWT:
//! the threads, the changes feed, the message history, prompts that start a
//! server-side reply, `interrupt`, and the `/log` stream of a thread's live
//! events. `GET /models` lists the models the instance can serve, and the
//! retention job tombstones threads idle longer than the instance's
//! `inactive_days`. Every change to a thread is announced to the person's other
//! desktops with the relay's change hint (`hint`).

pub mod auth;
pub mod catalog;
pub mod clock;
pub mod config;
pub mod error;
pub mod extension;
pub mod hint;
pub mod http;
pub mod ids;
pub mod reply;
pub mod retention;
pub mod router;
pub mod schema;
pub mod state;
pub mod store;

pub use extension::ChatsExtension;
pub use router::{EXTENSION_BASE_PATH, chats_router};
pub use state::{ChatsState, Shared};

systemprompt::extension::register_extension!(ChatsExtension);
