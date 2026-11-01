//! The `Extension` registration: schema, the router and the revocation
//! watch.
//!
//! Core asks an extension for its router more than once (to validate the
//! path, then to mount it), so the relay state is built on the first call
//! and kept in the extension value: one hub per process, one watch task.

use std::sync::{Arc, OnceLock};

use systemprompt::database::Database;
use systemprompt::extension::prelude::*;

use crate::router::{BASE_PATH, relay_router};
use crate::state::RelayState;
use crate::store::postgres::PgRelayStore;
use crate::{schema, sweeper};

#[derive(Debug, Default)]
pub struct RelayExtension {
    state: OnceLock<RelayState>,
}

impl RelayExtension {
    pub const ID: &'static str = "dynwork-relay";

    fn state(&self, ctx: &dyn ExtensionContext) -> Option<RelayState> {
        if let Some(state) = self.state.get() {
            return Some(state.clone());
        }
        if tokio::runtime::Handle::try_current().is_err() {
            tracing::error!("No async runtime at router build; the dynwork relay is not mounted");
            return None;
        }
        let handle = ctx.database();
        let Some(database) = handle.as_any().downcast_ref::<Database>() else {
            tracing::error!("Database handle is not Postgres; the dynwork relay is not mounted");
            return None;
        };
        let store = Arc::new(PgRelayStore::new(database.write_pool()));
        let built = RelayState::builder(store, ctx.token_authority(), ctx.background_tasks())
            .build()
            .inspect_err(|error| tracing::error!(%error, "dynwork relay limits refused"))
            .ok()?;
        match self.state.set(built) {
            Ok(()) => {
                let state = self.state.get()?;
                sweeper::spawn(state);
                Some(state.clone())
            },
            Err(_raced) => self.state.get().cloned(),
        }
    }
}

impl Extension for RelayExtension {
    fn metadata(&self) -> ExtensionMetadata {
        ExtensionMetadata {
            id: Self::ID,
            name: "DynWork Relay",
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
        Some(ExtensionRouter::public(relay_router(state), BASE_PATH))
    }
}
