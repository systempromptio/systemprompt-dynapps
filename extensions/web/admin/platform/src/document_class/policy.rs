//! `AgentAccessPolicy` for this installation: an agent's cap on a document
//! is the cap its document class grants an attended or unattended actor.
//!
//! The class is resolved through the folder chain core hands over (nearest
//! first), against the class rows in `document_classes`. The hard-rule
//! capabilities stay denied everywhere except `agent-workspace`, where an
//! agent may trash, and restore from trash, the drafts it created itself. A
//! lookup that fails answers `none` and denies: an unreadable class never
//! widens a cap.
//!
//! Core hands the folder chain nearest first; for a folder target the first
//! folder is the target itself, so its class is the target's own.
//!
//! Trash (which core's `Trash` capability pairs with restore-from-trash) is
//! an agent's own business only in a draft that same agent created inside
//! `agent-workspace`, or a file its person deleted from a synced Drive folder
//! where the class lets that device edit. Core names an agent that sent no name
//! `agent`, so unnamed agents share their drafts. `ReplaceContent` is allowed
//! only on core's Drive-sync and live-checkpoint channels, whose whole-file
//! saves and checkpoints core files under it, and only where the class lets
//! that agent edit: a checkpoint persists edits already authorized one by one.
//! Under `suggest` core writes the save as a proposal instead. Direct agent
//! calls stay denied, as does every other hard-rule capability.

use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;
use systemprompt::database::DbPool;
use systemprompt::files::documents::agents::{
    AgentAccessPolicy, AgentActor, AgentCap, AgentCapability, AgentChannel, DocumentContext,
    MoveActor, MoveRefused,
};
use systemprompt::identifiers::{DocumentFolderId, DocumentId};
use uuid::Uuid;

use super::catalog::{ClassCatalog, DeclaredClasses};
use super::resolve::{ClassChain, ClassSource, resolve};
use crate::repositories::document_classes::ClassRow;
use crate::services::document_classes::{
    ancestors_nearest_first, find_document_class, list_folder_classes,
};

pub const POLICY_NAME: &str = "dynapps-document-classes";
pub const AGENT_WORKSPACE: &str = "agent-workspace";

#[derive(Debug, Clone)]
pub struct DocumentClassPolicy {
    pub(super) pool: Arc<PgPool>,
    pub(super) classes: DeclaredClasses,
}

impl DocumentClassPolicy {
    #[must_use]
    pub const fn new(pool: Arc<PgPool>, classes: DeclaredClasses) -> Self {
        Self { pool, classes }
    }

    pub(super) async fn chain(&self, doc: &DocumentContext) -> Result<ClassChain, sqlx::Error> {
        let folders: Vec<Uuid> = doc
            .folders
            .iter()
            .filter_map(|f| Uuid::parse_str(f.as_str()).ok())
            .collect();
        let classed = list_folder_classes(&self.pool, &folders).await?;
        let document = doc
            .document_id
            .as_ref()
            .and_then(|d| Uuid::parse_str(d.as_str()).ok());
        let own = match document {
            Some(id) => find_document_class(&self.pool, id).await?,
            None => None,
        };
        Ok(chain_from(own, &folders, &classed, doc))
    }
}

#[must_use]
pub fn chain_from(
    own: Option<String>,
    folders: &[Uuid],
    classed: &[ClassRow],
    doc: &DocumentContext,
) -> ClassChain {
    let root_first: Vec<Uuid> = folders.iter().rev().copied().collect();
    let mut ancestors = ancestors_nearest_first(&root_first, classed);
    let own = if doc.document_id.is_none() && !ancestors.is_empty() {
        ancestors.remove(0).1
    } else {
        own
    };
    ClassChain {
        own,
        ancestors,
        created_by_agent: doc
            .created_by_agent
            .as_ref()
            .map(|agent| agent.as_str().to_owned()),
    }
}

#[must_use]
pub fn cap_in(catalog: &ClassCatalog, chain: &ClassChain, agent: &AgentActor) -> AgentCap {
    match resolve(catalog, chain) {
        Some(r) if !matches!(r.source, ClassSource::Unknown(_)) => r.class.cap_for(agent.attended),
        _ => AgentCap::None,
    }
}

#[must_use]
pub fn allows_in(
    catalog: &ClassCatalog,
    chain: &ClassChain,
    capability: AgentCapability,
    agent: &AgentActor,
) -> bool {
    match capability {
        AgentCapability::Trash => {
            let in_workspace = resolve(catalog, chain).is_some_and(|r| {
                r.class.id == AGENT_WORKSPACE && !matches!(r.source, ClassSource::Unknown(_))
            });
            let own_draft = chain.created_by_agent.as_deref() == Some(agent.name.as_str());
            let device_delete = agent.channel == AgentChannel::DriveSync
                && cap_in(catalog, chain, agent) == AgentCap::Edit;
            (in_workspace && own_draft) || device_delete
        },
        AgentCapability::ReplaceContent => {
            matches!(
                agent.channel,
                AgentChannel::DriveSync | AgentChannel::LiveCheckpoint
            ) && cap_in(catalog, chain, agent) == AgentCap::Edit
        },
        AgentCapability::BroadRewrite => {
            agent.attended && cap_in(catalog, chain, agent) == AgentCap::Edit
        },
        _ => false,
    }
}

#[async_trait]
impl AgentAccessPolicy for DocumentClassPolicy {
    fn name(&self) -> &'static str {
        POLICY_NAME
    }

    async fn cap(&self, doc: &DocumentContext, agent: &AgentActor) -> AgentCap {
        let Ok(catalog) = self.classes.get() else {
            return AgentCap::None;
        };
        match self.chain(doc).await {
            Ok(chain) => cap_in(catalog, &chain, agent),
            Err(error) => {
                tracing::error!(%error, "document class lookup failed; agent capped at none");
                AgentCap::None
            },
        }
    }

    async fn allows(
        &self,
        capability: AgentCapability,
        doc: &DocumentContext,
        agent: &AgentActor,
    ) -> bool {
        let Ok(catalog) = self.classes.get() else {
            return false;
        };
        if !matches!(
            capability,
            AgentCapability::Trash
                | AgentCapability::ReplaceContent
                | AgentCapability::BroadRewrite
        ) {
            return false;
        }
        match self.chain(doc).await {
            Ok(chain) => allows_in(catalog, &chain, capability, agent),
            Err(error) => {
                tracing::error!(%error, "document class lookup failed; capability denied");
                false
            },
        }
    }

    async fn on_move(
        &self,
        doc: &DocumentContext,
        to: &[DocumentFolderId],
        mover: &MoveActor,
    ) -> Result<(), MoveRefused> {
        self.move_hook(doc, to, mover)
            .await
            .map_err(MoveRefused::from)
    }

    async fn on_copy(&self, source: &DocumentContext, copy: &DocumentId) {
        self.copy_hook(source, copy).await;
    }
}

fn build(db: &DbPool) -> Arc<dyn AgentAccessPolicy> {
    Arc::new(DocumentClassPolicy::new(db.pool(), DeclaredClasses::load()))
}

systemprompt::register_agent_access_policy!(POLICY_NAME, build);
