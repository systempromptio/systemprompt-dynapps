//! The installation's document agent policy: core's deny-all default, except
//! the bridge Drive sync's whole-file saves.
//!
//! Core 0.75 caps every agent and denies each [`AgentCapability`] unless a
//! registered policy allows it. The Drive sync is an agent on the
//! [`AgentChannel::DriveSync`] channel, and its commit is a
//! `replace_content`, so without this policy every upload from a synced
//! folder is refused. Only that capability on that channel is granted; an
//! agent on any other path, and every other capability, stays denied.

use std::sync::Arc;

use systemprompt::database::DbPool;
use systemprompt::files::documents::agents::{
    AgentAccessPolicy, AgentActor, AgentCapability, AgentChannel, DocumentContext,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct DriveSyncSavesPolicy;

#[must_use]
pub const fn allows(capability: AgentCapability, channel: AgentChannel) -> bool {
    matches!(
        (capability, channel),
        (AgentCapability::ReplaceContent, AgentChannel::DriveSync)
    )
}

#[async_trait::async_trait]
impl AgentAccessPolicy for DriveSyncSavesPolicy {
    fn name(&self) -> &'static str {
        "dynapps-drive-sync-saves"
    }

    async fn allows(
        &self,
        capability: AgentCapability,
        _doc: &DocumentContext,
        agent: &AgentActor,
    ) -> bool {
        allows(capability, agent.channel)
    }
}

fn build(_db: &DbPool) -> Arc<dyn AgentAccessPolicy> {
    Arc::new(DriveSyncSavesPolicy)
}

systemprompt::files::register_agent_access_policy!("dynapps-drive-sync-saves", build);
