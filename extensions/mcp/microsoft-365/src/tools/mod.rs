//! The tool surface of the `microsoft-365` server.
//!
//! Each tool's name, title and behaviour hints follow the Belgian server's
//! `tools/_annotations.py`, so clients see the same `d365_*` contract; the
//! dispatch goes from a tool name to its handler.

pub mod calendar;
pub mod calendar_event;
pub mod chat;
pub mod chat_write;
pub mod contacts;
pub mod drive;
pub mod drive_content;
mod drive_tools;
pub mod drive_write;
mod file_tool;
pub mod graph_tool;
pub mod groups;
mod handlers;
pub mod mail;
pub mod mail_attach;
pub mod mail_folders;
pub mod mail_read;
pub mod mail_send;
mod mail_tools;
pub mod mail_upload;
pub mod meetings;
pub mod meetings_media;
pub mod onenote;
pub mod planner;
pub mod planner_bucket;
pub mod planner_details;
mod planner_hub;
pub mod planner_roster;
pub mod planner_task;
pub mod presence;
pub mod reply;
mod rest_hub;
pub mod search;
pub mod sharepoint;
pub mod teams;
mod teams_hub;
pub mod teams_write;
pub mod todo;
pub mod users;

use std::sync::LazyLock;

use rmcp::model::{Tool, ToolAnnotations};
use systemprompt::identifiers::McpServerId;

pub use handlers::{ToolCall, dispatch};

use crate::SERVER_NAME;

pub const INSTRUCTIONS: &str = "Microsoft 365 through Microsoft Graph, acting as the signed-in \
    user on their own tenant. Connect Microsoft 365 on the Connectors page first; every tool \
    then reads or changes only what that user may reach in Graph. List tools page with `top` \
    and `skip`: when a reply says `has_more`, call again with `skip` increased by `top`.";

/// The four behaviour classes of `_annotations.py`.
///
/// `Read` for list, get, search and downloads; `Write` for creates (twice
/// creates twice); `IdempotentWrite` for updates and responses; `Destructive`
/// for deletes and sends. Every tool is open-world (it reaches Microsoft's
/// service).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolKind {
    Read,
    Write,
    IdempotentWrite,
    Destructive,
}

impl ToolKind {
    pub fn annotations(self, title: &str) -> ToolAnnotations {
        let (read_only, destructive, idempotent) = match self {
            Self::Read => (true, false, true),
            Self::Write => (false, false, false),
            Self::IdempotentWrite => (false, false, true),
            Self::Destructive => (false, true, false),
        };
        ToolAnnotations::with_title(title)
            .read_only(read_only)
            .destructive(destructive)
            .idempotent(idempotent)
            .open_world(true)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolSpec {
    pub name: &'static str,
    pub title: &'static str,
    pub kind: ToolKind,
}

impl ToolSpec {
    pub const fn new(name: &'static str, title: &'static str, kind: ToolKind) -> Self {
        Self { name, title, kind }
    }
}

// Why: check-file-size.sh caps a source at 300 lines, so each later domain
// keeps its specs beside its tools; this joins them after the mail,
// calendar, contacts and To Do specs below.
pub static TOOLS: LazyLock<Vec<ToolSpec>> = LazyLock::new(|| {
    [
        BASE_SPECS,
        drive_tools::DRIVE_SPECS,
        sharepoint::SHAREPOINT_SPECS,
        teams_hub::TEAMS_SPECS,
        planner_hub::PLANNER_SPECS,
        rest_hub::REST_SPECS,
    ]
    .concat()
});

const BASE_SPECS: &[ToolSpec] = &[
    ToolSpec::new("d365_list_mail", "List emails", ToolKind::Read),
    ToolSpec::new("d365_get_mail", "Get email", ToolKind::Read),
    ToolSpec::new("d365_search_mail", "Search emails", ToolKind::Read),
    ToolSpec::new(
        "d365_list_mail_folders",
        "List mail folders",
        ToolKind::Read,
    ),
    ToolSpec::new(
        "d365_list_mail_attachments",
        "List mail attachments",
        ToolKind::Read,
    ),
    ToolSpec::new(
        "d365_get_mail_attachment",
        "Get mail attachment",
        ToolKind::Read,
    ),
    ToolSpec::new("d365_list_mail_rules", "List mail rules", ToolKind::Read),
    ToolSpec::new(
        "d365_list_shared_mailbox_messages",
        "List shared mailbox messages",
        ToolKind::Read,
    ),
    ToolSpec::new("d365_send_mail", "Send email", ToolKind::Destructive),
    ToolSpec::new("d365_reply_mail", "Reply to email", ToolKind::Destructive),
    ToolSpec::new("d365_forward_mail", "Forward email", ToolKind::Destructive),
    ToolSpec::new("d365_create_draft", "Create email draft", ToolKind::Write),
    ToolSpec::new("d365_send_draft", "Send email draft", ToolKind::Destructive),
    ToolSpec::new(
        "d365_update_mail",
        "Update email",
        ToolKind::IdempotentWrite,
    ),
    ToolSpec::new("d365_delete_mail", "Delete email", ToolKind::Destructive),
    ToolSpec::new("d365_move_mail", "Move email", ToolKind::Write),
    ToolSpec::new(
        "d365_add_mail_attachment",
        "Add mail attachment",
        ToolKind::Write,
    ),
    ToolSpec::new(
        "d365_create_mail_attachment_upload_session",
        "Create attachment upload session",
        ToolKind::Write,
    ),
    ToolSpec::new(
        "d365_upload_attachment_chunk",
        "Upload attachment chunk",
        ToolKind::Write,
    ),
    ToolSpec::new(
        "d365_delete_mail_attachment",
        "Delete mail attachment",
        ToolKind::Destructive,
    ),
    ToolSpec::new(
        "d365_create_mail_folder",
        "Create mail folder",
        ToolKind::Write,
    ),
    ToolSpec::new(
        "d365_delete_mail_folder",
        "Delete mail folder",
        ToolKind::Destructive,
    ),
    ToolSpec::new("d365_create_mail_rule", "Create mail rule", ToolKind::Write),
    ToolSpec::new(
        "d365_create_outlook_category",
        "Create Outlook category",
        ToolKind::Write,
    ),
    ToolSpec::new("d365_list_calendars", "List calendars", ToolKind::Read),
    ToolSpec::new("d365_list_events", "List calendar events", ToolKind::Read),
    ToolSpec::new("d365_get_event", "Get calendar event", ToolKind::Read),
    ToolSpec::new(
        "d365_list_calendar_view",
        "List calendar view",
        ToolKind::Read,
    ),
    ToolSpec::new(
        "d365_list_shared_calendar_events",
        "List shared calendar events",
        ToolKind::Read,
    ),
    ToolSpec::new(
        "d365_create_event",
        "Create calendar event",
        ToolKind::Write,
    ),
    ToolSpec::new(
        "d365_update_event",
        "Update calendar event",
        ToolKind::IdempotentWrite,
    ),
    ToolSpec::new(
        "d365_delete_event",
        "Delete calendar event",
        ToolKind::Destructive,
    ),
    ToolSpec::new(
        "d365_accept_event",
        "Accept calendar event",
        ToolKind::IdempotentWrite,
    ),
    ToolSpec::new(
        "d365_decline_event",
        "Decline calendar event",
        ToolKind::IdempotentWrite,
    ),
    ToolSpec::new(
        "d365_tentatively_accept_event",
        "Tentatively accept event",
        ToolKind::IdempotentWrite,
    ),
    ToolSpec::new("d365_list_contacts", "List contacts", ToolKind::Read),
    ToolSpec::new("d365_get_contact", "Get contact", ToolKind::Read),
    ToolSpec::new("d365_create_contact", "Create contact", ToolKind::Write),
    ToolSpec::new(
        "d365_update_contact",
        "Update contact",
        ToolKind::IdempotentWrite,
    ),
    ToolSpec::new(
        "d365_delete_contact",
        "Delete contact",
        ToolKind::Destructive,
    ),
    ToolSpec::new("d365_list_todo_lists", "List to-do lists", ToolKind::Read),
    ToolSpec::new("d365_list_todo_tasks", "List to-do tasks", ToolKind::Read),
    ToolSpec::new("d365_get_todo_task", "Get to-do task", ToolKind::Read),
    ToolSpec::new(
        "d365_create_todo_task",
        "Create to-do task",
        ToolKind::Write,
    ),
    ToolSpec::new(
        "d365_update_todo_task",
        "Update to-do task",
        ToolKind::IdempotentWrite,
    ),
    ToolSpec::new(
        "d365_delete_todo_task",
        "Delete to-do task",
        ToolKind::Destructive,
    ),
    ToolSpec::new(
        "d365_create_todo_list",
        "Create to-do list",
        ToolKind::Write,
    ),
];

pub fn spec(name: &str) -> Option<&'static ToolSpec> {
    TOOLS.iter().find(|spec| spec.name == name)
}

pub fn tool_names() -> Vec<&'static str> {
    TOOLS.iter().map(|spec| spec.name).collect()
}

pub fn list_tools() -> Vec<Tool> {
    handlers::definitions(&McpServerId::new(SERVER_NAME))
        .into_iter()
        .map(|mut tool| {
            tool.annotations = spec(&tool.name).map(|spec| spec.kind.annotations(spec.title));
            tool
        })
        .collect()
}
