//! The Teams family's surface — teams and channels, chats, presence and
//! online meetings: each tool's title and behaviour class from the Belgian
//! `_annotations.py`, and one definition list and dispatch over the family's
//! domain tables, so the server's handler takes the family in one step.

use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Tool};
use systemprompt::identifiers::McpServerId;

use super::{ToolCall, ToolKind, ToolSpec};

pub(super) const TEAMS_SPECS: &[ToolSpec] = &[
    ToolSpec::new("d365_list_teams", "List teams", ToolKind::Read),
    ToolSpec::new("d365_list_channels", "List channels", ToolKind::Read),
    ToolSpec::new("d365_get_channel", "Get channel", ToolKind::Read),
    ToolSpec::new(
        "d365_list_channel_messages",
        "List channel messages",
        ToolKind::Read,
    ),
    ToolSpec::new(
        "d365_get_channel_message",
        "Get channel message",
        ToolKind::Read,
    ),
    ToolSpec::new(
        "d365_list_channel_message_replies",
        "List channel message replies",
        ToolKind::Read,
    ),
    ToolSpec::new(
        "d365_list_channel_members",
        "List channel members",
        ToolKind::Read,
    ),
    ToolSpec::new(
        "d365_list_team_members",
        "List team members",
        ToolKind::Read,
    ),
    ToolSpec::new(
        "d365_send_channel_message",
        "Send channel message",
        ToolKind::Destructive,
    ),
    ToolSpec::new(
        "d365_reply_to_channel_message",
        "Reply to channel message",
        ToolKind::Destructive,
    ),
    ToolSpec::new("d365_create_channel", "Create channel", ToolKind::Write),
    ToolSpec::new(
        "d365_update_channel",
        "Update channel",
        ToolKind::IdempotentWrite,
    ),
    ToolSpec::new("d365_list_chats", "List chats", ToolKind::Read),
    ToolSpec::new("d365_get_chat", "Get chat", ToolKind::Read),
    ToolSpec::new(
        "d365_list_chat_messages",
        "List chat messages",
        ToolKind::Read,
    ),
    ToolSpec::new(
        "d365_list_chat_members",
        "List chat members",
        ToolKind::Read,
    ),
    ToolSpec::new(
        "d365_send_chat_message",
        "Send chat message",
        ToolKind::Destructive,
    ),
    ToolSpec::new(
        "d365_create_one_on_one_chat",
        "Create one-on-one chat",
        ToolKind::Write,
    ),
    ToolSpec::new(
        "d365_create_group_chat",
        "Create group chat",
        ToolKind::Write,
    ),
    ToolSpec::new("d365_pin_chat_message", "Pin chat message", ToolKind::Write),
    ToolSpec::new(
        "d365_unpin_chat_message",
        "Unpin chat message",
        ToolKind::Write,
    ),
    ToolSpec::new(
        "d365_set_chat_message_reaction",
        "React to chat message",
        ToolKind::Write,
    ),
    ToolSpec::new("d365_get_my_presence", "Get my presence", ToolKind::Read),
    ToolSpec::new(
        "d365_get_user_presence",
        "Get user presence",
        ToolKind::Read,
    ),
    ToolSpec::new(
        "d365_set_my_presence",
        "Set my presence",
        ToolKind::IdempotentWrite,
    ),
    ToolSpec::new(
        "d365_list_online_meetings",
        "List online meetings",
        ToolKind::Read,
    ),
    ToolSpec::new(
        "d365_get_online_meeting",
        "Get online meeting",
        ToolKind::Read,
    ),
    ToolSpec::new(
        "d365_get_meeting_by_join_url",
        "Get meeting by join URL",
        ToolKind::Read,
    ),
    ToolSpec::new(
        "d365_create_online_meeting",
        "Create online meeting",
        ToolKind::Write,
    ),
    ToolSpec::new(
        "d365_delete_online_meeting",
        "Delete online meeting",
        ToolKind::Destructive,
    ),
    ToolSpec::new(
        "d365_list_meeting_transcripts",
        "List meeting transcripts",
        ToolKind::Read,
    ),
    ToolSpec::new(
        "d365_get_meeting_transcript_content",
        "Get meeting transcript",
        ToolKind::Read,
    ),
    ToolSpec::new(
        "d365_list_meeting_recordings",
        "List meeting recordings",
        ToolKind::Read,
    ),
    ToolSpec::new(
        "d365_get_meeting_recording",
        "Get meeting recording",
        ToolKind::Read,
    ),
    ToolSpec::new(
        "d365_list_meeting_attendance_reports",
        "List meeting attendance reports",
        ToolKind::Read,
    ),
];

pub(super) fn definitions(server: &McpServerId) -> Vec<Tool> {
    let mut tools = super::teams::definitions(server);
    tools.extend(super::teams_write::definitions(server));
    tools.extend(super::chat::definitions(server));
    tools.extend(super::chat_write::definitions(server));
    tools.extend(super::presence::definitions(server));
    tools.extend(super::meetings::definitions(server));
    tools.extend(super::meetings_media::definitions(server));
    tools.extend(super::meetings_media::file_definitions(server));
    tools
}

pub(super) async fn dispatch(call: &ToolCall<'_>) -> Option<Result<CallToolResult, McpError>> {
    if let Some(result) = super::teams::dispatch(call).await {
        return Some(result);
    }
    if let Some(result) = super::teams_write::dispatch(call).await {
        return Some(result);
    }
    if let Some(result) = super::chat::dispatch(call).await {
        return Some(result);
    }
    if let Some(result) = super::chat_write::dispatch(call).await {
        return Some(result);
    }
    if let Some(result) = super::presence::dispatch(call).await {
        return Some(result);
    }
    if let Some(result) = super::meetings::dispatch(call).await {
        return Some(result);
    }
    if let Some(result) = super::meetings_media::dispatch(call).await {
        return Some(result);
    }
    super::meetings_media::file_dispatch(call).await
}
