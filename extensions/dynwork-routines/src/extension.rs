//! The `Extension` registration: schema and the desktop API.
//!
//! Core asks an extension for its router more than once, so the state is
//! built on the first call and kept in the extension value.

use std::sync::OnceLock;

use systemprompt::database::Database;
use systemprompt::extension::prelude::*;

use crate::router::{BASE_PATH, routines_router};
use crate::schema;
use crate::state::RoutinesState;

#[derive(Debug, Default)]
pub struct RoutinesExtension {
    state: OnceLock<RoutinesState>,
}

impl RoutinesExtension {
    pub const ID: &'static str = "dynwork-routines";

    fn state(&self, ctx: &dyn ExtensionContext) -> Option<RoutinesState> {
        if let Some(state) = self.state.get() {
            return Some(state.clone());
        }
        let handle = ctx.database();
        let Some(database) = handle.as_any().downcast_ref::<Database>() else {
            tracing::error!("Database handle is not Postgres; dynwork routines are not mounted");
            return None;
        };
        let built = RoutinesState {
            pool: database.write_pool(),
            authority: ctx.token_authority(),
        };
        Some(self.state.get_or_init(|| built).clone())
    }
}

impl Extension for RoutinesExtension {
    fn metadata(&self) -> ExtensionMetadata {
        ExtensionMetadata {
            id: Self::ID,
            name: "DynWork Routines",
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
        Some(ExtensionRouter::public(routines_router(state), BASE_PATH))
    }
}
