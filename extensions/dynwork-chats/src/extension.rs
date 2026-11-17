//! The `Extension` registration: schema, migrations and the chats router.
//!
//! Core asks an extension for its router more than once (to validate the
//! path, then to mount it), so the state is built on the first call and kept
//! in the extension value: one store, one device check and one registry of
//! running replies per process.

use std::sync::{Arc, OnceLock};

use systemprompt::database::Database;
use systemprompt::extension::prelude::*;

use crate::auth::RelayDevices;
use crate::clock::SystemClock;
use crate::reply::model::Gateway;
use crate::reply::registry::Replies;
use crate::router::{EXTENSION_BASE_PATH, chats_router};
use crate::schema;
use crate::state::{ChatsState, Shared};
use crate::store::postgres::PgChatStore;

#[derive(Debug, Default)]
pub struct ChatsExtension {
    state: OnceLock<ChatsState>,
}

impl ChatsExtension {
    pub const ID: &'static str = "dynwork-chats";

    fn state(&self, ctx: &dyn ExtensionContext) -> Option<ChatsState> {
        if let Some(state) = self.state.get() {
            return Some(state.clone());
        }
        let handle = ctx.database();
        let Some(database) = handle.as_any().downcast_ref::<Database>() else {
            tracing::error!("Database handle is not Postgres; dynwork chats is not mounted");
            return None;
        };
        let pool = database.write_pool();
        let built = ChatsState::new(Shared {
            store: Arc::new(PgChatStore::new(Arc::clone(&pool))),
            authority: ctx.token_authority(),
            devices: Arc::new(RelayDevices::new(pool)),
            clock: Arc::new(SystemClock),
            models: Arc::new(Gateway),
            replies: Replies::default(),
        });
        Some(self.state.get_or_init(|| built).clone())
    }
}

impl Extension for ChatsExtension {
    fn metadata(&self) -> ExtensionMetadata {
        ExtensionMetadata {
            id: Self::ID,
            name: "DynWork Chats",
            version: env!("CARGO_PKG_VERSION"),
        }
    }

    fn schemas(&self) -> Vec<SchemaDefinition> {
        schema::schema_definitions()
    }

    fn migrations(&self) -> Vec<Migration> {
        schema::migrations()
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec!["users"]
    }

    fn router(&self, ctx: &dyn ExtensionContext) -> Option<ExtensionRouter> {
        let state = self.state(ctx)?;
        Some(ExtensionRouter::public(
            chats_router(state),
            EXTENSION_BASE_PATH,
        ))
    }
}
