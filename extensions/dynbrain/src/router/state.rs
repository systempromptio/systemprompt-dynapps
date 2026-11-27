//! What every route shares.
//!
//! It is built once per process by the extension, or by a suite with its own
//! authority.
//!
//! `embedder` is the query embedder of the search: core hands its AI service
//! to the routes with each request, and the first search fills the slot with
//! the catalog's embedding model, the one the indexer uses.

use std::sync::Arc;

use dynwork_relay::store::postgres::PgRelayStore;
use systemprompt::ai::AiService;
use systemprompt::models::auth::TokenAuthority;

use crate::brain::Brain;
use crate::ingest::core_ai::{CoreEmbedder, EMBEDDING_MODEL, EMBEDDING_PROVIDER};
use crate::search::query::EmbedderSlot;

#[derive(Debug, Clone)]
pub struct BrainState {
    pub brain: Arc<dyn Brain>,
    pub authority: Arc<dyn TokenAuthority>,
    pub devices: Arc<PgRelayStore>,
    pub embedder: EmbedderSlot,
}

impl BrainState {
    pub fn fill_embedder(&self, ai: Option<Arc<AiService>>) {
        let Some(ai) = ai else {
            return;
        };
        self.embedder.fill(move || {
            CoreEmbedder::new(
                ai,
                EMBEDDING_PROVIDER.to_owned(),
                EMBEDDING_MODEL.to_owned(),
            )
        });
    }
}
