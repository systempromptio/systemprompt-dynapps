//! The `Extension` registration: the schema, its migrations, the config block
//! and the Brain router.
//!
//! Core asks an extension for its router more than once (to validate the
//! path, then to mount it), so the state is built on the first call and kept
//! in the extension value: one Brain and one device check per process.
//!
//! [`compose`] builds the Brain the instance's config asks for, native or
//! proxy, for the API and for the `brain` MCP server alike. A config that
//! cannot be used (an unknown mode, a proxy with no source, a source server
//! that is not declared) leaves the Brain unmounted in the API and stops the
//! MCP server from starting.
//!
//! The files of articles use core's file storage and links signed with the
//! instance secret `dynbrain_links`, which the API and the `brain` server
//! both hold, so a link either signs opens the API's download route; without
//! it the Brain still runs and its file operations answer `unavailable`. The
//! search reranks with Cohere when the instance has the `cohere` secret, and
//! keeps the fused order without it.

use std::sync::{Arc, OnceLock};

use dynwork_relay::store::postgres::PgRelayStore;
use systemprompt::database::Database;
use systemprompt::extension::prelude::{
    Extension, ExtensionConfigError, ExtensionContext, ExtensionMetadata, ExtensionRouter,
    Migration, SchemaDefinition,
};
use systemprompt::mcp::RegistryService;
use systemprompt::mcp::services::client::{McpClient, McpClientLimits};
use systemprompt::security::credential::TokenCache;

use crate::assets::AssetFiles;
use crate::assets::link::{LINKS_SECRET, LinkSigner};
use crate::brain::Brain;
use crate::brain::native::NativeBrain;
use crate::brain::proxy::ProxyBrain;
use crate::brain::proxy::source::{McpSource, StoreSecrets};
use crate::config::{BrainConfig, CONFIG_PREFIX, ConfigError, Mode, ProxyConfig, check_server};
use crate::console::{self, Console, Native};
use crate::repository::BrainRepository;
use crate::router::state::BrainState;
use crate::router::{EXTENSION_BASE_PATH, brain_router};
use crate::schema;
use crate::search::Retriever;
use crate::search::cohere::{self, CohereReranker};
use crate::search::query::EmbedderSlot;
use crate::search::rerank::Reranker;

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
        let embedder = EmbedderSlot::default();
        let brain = compose(ctx, embedder.clone())
            .inspect_err(|error| {
                tracing::error!(%error, "the Brain's config cannot be used; it is not mounted");
            })
            .ok()?;
        Some(
            self.state
                .get_or_init(|| BrainState {
                    brain,
                    authority: ctx.token_authority(),
                    devices: Arc::new(PgRelayStore::new(database.write_pool())),
                    embedder,
                })
                .clone(),
        )
    }
}

pub fn compose(
    ctx: &dyn ExtensionContext,
    embedder: EmbedderSlot,
) -> Result<Arc<dyn Brain>, ConfigError> {
    let config = BrainConfig::load(&ctx.services_root().path)?;
    // Why: an instance is either native or proxy, never both. Merging results from
    // Governance's own index and a source's index needs one ranking across two
    // systems that score differently, and that is not solved; a customer who
    // needs several sources must send their documentation into Governance (an
    // ingestion integration, to be evaluated) instead of using the proxy.
    // Decided by David, 24-11-2026.
    match (config.mode, config.proxy) {
        (Mode::Proxy, Some(proxy)) => {
            let brain: Arc<dyn Brain> = Arc::new(proxy_brain(ctx, proxy)?);
            console::publish(Console {
                mode: Mode::Proxy,
                brain: Arc::clone(&brain),
                native: None,
            });
            Ok(brain)
        },
        (Mode::Proxy, None) => Err(ConfigError::NoSource),
        (Mode::Native, _) => {
            let (brain, native) = native_brain(ctx, embedder)?;
            let shared: Arc<dyn Brain> = brain;
            console::publish(Console {
                mode: Mode::Native,
                brain: Arc::clone(&shared),
                native: Some(native),
            });
            Ok(shared)
        },
    }
}

fn native_brain(
    ctx: &dyn ExtensionContext,
    embedder: EmbedderSlot,
) -> Result<(Arc<NativeBrain>, Native), ConfigError> {
    let handle = ctx.database();
    let database = handle
        .as_any()
        .downcast_ref::<Database>()
        .ok_or(ConfigError::NotPostgres)?;
    let repo = BrainRepository::new(database.write_pool());
    let search = Retriever::new(repo.clone())
        .with_embedder(embedder)
        .with_reranker(reranker(ctx));
    let brain = Arc::new(
        NativeBrain::new(repo.clone())
            .with_files(asset_files(ctx))
            .with_search(search.clone()),
    );
    let native = Native {
        brain: Arc::clone(&brain),
        repo,
        retriever: search,
    };
    Ok((brain, native))
}

fn proxy_brain(ctx: &dyn ExtensionContext, proxy: ProxyConfig) -> Result<ProxyBrain, ConfigError> {
    let services = ctx.services();
    check_server(&services.config(), &proxy.server)?;
    let registry = RegistryService::new(ctx.system_owner_id(), services, ctx.app_config());
    let server = registry
        .get_server(&proxy.server)
        .map_err(|error| ConfigError::Registry(Box::new(error)))?;
    let client = McpClient::new(
        Arc::new(TokenCache::new()),
        registry.deployments(),
        Arc::clone(registry.config()),
        Arc::new(StoreSecrets(ctx.secrets())),
        McpClientLimits::default(),
    );
    tracing::info!(server = %proxy.server, "the Brain answers through its source");
    Ok(ProxyBrain::new(
        Arc::new(McpSource::new(client, server)),
        proxy,
    ))
}

fn reranker(ctx: &dyn ExtensionContext) -> Option<Arc<dyn Reranker>> {
    let Some(key) = ctx.secrets().secret(cohere::PROVIDER) else {
        tracing::info!("no cohere key; searches keep the fused order, unrated");
        return None;
    };
    CohereReranker::new(key.expose().to_owned(), cohere::ENDPOINT, cohere::MODEL)
        .inspect_err(|error| {
            tracing::error!(%error, "the cohere reranker is off");
        })
        .ok()
        .map(|reranker| -> Arc<dyn Reranker> { Arc::new(reranker) })
}

fn asset_files(ctx: &dyn ExtensionContext) -> Option<AssetFiles> {
    let Some(secret) = ctx.secrets().secret(LINKS_SECRET) else {
        tracing::warn!("no dynbrain_links secret; the Brain's files are off");
        return None;
    };
    let signer = LinkSigner::from_secret(secret.expose())?;
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

    // Why: only `users` is a foreign key of the schema. The grants read the
    // web extension's `user_groups` at query time, but declaring `web` here
    // stops the `brain` MCP binary, which does not register that extension.
    fn dependencies(&self) -> Vec<&'static str> {
        vec!["users"]
    }

    fn config_prefix(&self) -> Option<&str> {
        Some(CONFIG_PREFIX)
    }

    // JSON: core hands an extension its config block as a JSON document.
    fn validate_config(&self, config: &serde_json::Value) -> Result<(), ExtensionConfigError> {
        BrainConfig::from_json(config).map(|_| ()).map_err(|error| {
            ExtensionConfigError::ParseError {
                source: Box::new(error),
            }
        })
    }

    fn router(&self, ctx: &dyn ExtensionContext) -> Option<ExtensionRouter> {
        let state = self.state(ctx)?;
        Some(ExtensionRouter::public(
            brain_router(state),
            EXTENSION_BASE_PATH,
        ))
    }
}
