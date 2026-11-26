//! The `Extension` registration: the schema, its migrations and the Brain
//! router.
//!
//! Core asks an extension for its router more than once (to validate the
//! path, then to mount it), so the state is built on the first call and kept
//! in the extension value: one repository and one device check per process.
//!
//! The files of articles use core's file storage and a link key derived from
//! the instance's manifest signing seed; without that seed the Brain still
//! runs and its file operations answer `unavailable`.

use std::sync::{Arc, OnceLock};

use dynwork_relay::store::postgres::PgRelayStore;
use systemprompt::database::Database;
use systemprompt::extension::prelude::{
    Extension, ExtensionContext, ExtensionMetadata, ExtensionRouter, Migration, SchemaDefinition,
};

use crate::assets::AssetFiles;
use crate::assets::link::LinkSigner;
use crate::brain::native::NativeBrain;
use crate::repository::BrainRepository;
use crate::router::state::BrainState;
use crate::router::{EXTENSION_BASE_PATH, brain_router};
use crate::schema;

#[derive(Debug, Default)]
pub struct DynbrainExtension {
    state: OnceLock<BrainState>,
}

impl DynbrainExtension {
    pub const ID: &'static str = "dynbrain";

    fn state(&self, ctx: &dyn ExtensionContext) -> Option<BrainState> {
        if let Some(state) = self.state.get() {
            return Some(state.clone());
        }
        let handle = ctx.database();
        let Some(database) = handle.as_any().downcast_ref::<Database>() else {
            tracing::error!("Database handle is not Postgres; the Brain is not mounted");
            return None;
        };
        Some(
            self.state
                .get_or_init(|| build_state(ctx, database))
                .clone(),
        )
    }
}

fn build_state(ctx: &dyn ExtensionContext, database: &Database) -> BrainState {
    let pool = database.write_pool();
    BrainState {
        brain: Arc::new(
            NativeBrain::new(BrainRepository::new(Arc::clone(&pool))).with_files(asset_files(ctx)),
        ),
        authority: ctx.token_authority(),
        devices: Arc::new(PgRelayStore::new(pool)),
    }
}

fn asset_files(ctx: &dyn ExtensionContext) -> Option<AssetFiles> {
    let seed = ctx
        .secrets()
        .manifest_signing_secret_seed()
        .inspect_err(|error| {
            tracing::error!(%error, "no manifest signing seed; the Brain's files are off");
        })
        .ok()?;
    let signer = LinkSigner::new(&seed)?;
    Some(AssetFiles::new(
        ctx.file_storage(),
        signer,
        ctx.app_config().api_external_url.clone(),
    ))
}

impl Extension for DynbrainExtension {
    fn metadata(&self) -> ExtensionMetadata {
        ExtensionMetadata {
            id: Self::ID,
            name: "Brain",
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
        vec!["users", "web"]
    }

    fn router(&self, ctx: &dyn ExtensionContext) -> Option<ExtensionRouter> {
        let state = self.state(ctx)?;
        Some(ExtensionRouter::public(
            brain_router(state),
            EXTENSION_BASE_PATH,
        ))
    }
}
