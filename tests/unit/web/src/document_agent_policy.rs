//! The installation's document agent policy grants the Drive sync its
//! whole-file saves and nothing else: every other capability, and the same
//! capability on any other channel, stays denied as core's default denies it.

use systemprompt::files::documents::agents::{AgentCapability, AgentChannel};
use systemprompt_web_admin_governance::document_agent_policy::allows;

const CAPABILITIES: [AgentCapability; 9] = [
    AgentCapability::Share,
    AgentCapability::MentionGrant,
    AgentCapability::Trash,
    AgentCapability::Administer,
    AgentCapability::RestoreRevision,
    AgentCapability::ReplaceContent,
    AgentCapability::ReviewProposal,
    AgentCapability::CopyOut,
    AgentCapability::EditTemplate,
];

#[test]
fn only_drive_sync_replace_content_is_allowed() {
    for channel in [
        AgentChannel::Direct,
        AgentChannel::DriveSync,
        AgentChannel::LiveCheckpoint,
    ] {
        for capability in CAPABILITIES {
            let expected =
                capability == AgentCapability::ReplaceContent && channel == AgentChannel::DriveSync;
            assert_eq!(
                allows(capability, channel),
                expected,
                "{capability} on {channel:?}"
            );
        }
    }
}
