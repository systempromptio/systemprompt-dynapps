//! The transcript side of the export dialog: which list a JSON Lines export
//! of whole conversations is drawn from, how many it may hold, and the
//! download links of one conversation's full record.

use serde::Serialize;
use systemprompt::identifiers::ContextId;

use super::model::Window;

pub const MAX_CONVERSATIONS: i64 = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranscriptSource {
    Sessions,
    Analysis,
    Conversations,
    History,
}

impl TranscriptSource {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sessions => "sessions",
            Self::Analysis => "analysis",
            Self::Conversations => "conversations",
            Self::History => "history",
        }
    }

    #[must_use]
    pub const fn window(self) -> Window {
        match self {
            Self::Sessions | Self::Analysis => Window::Live,
            Self::Conversations | Self::History => Window::Retained,
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        [
            Self::Sessions,
            Self::Analysis,
            Self::Conversations,
            Self::History,
        ]
        .into_iter()
        .find(|s| s.as_str() == value)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DocumentExportView {
    pub json_href: String,
    pub markdown_href: String,
}

impl DocumentExportView {
    #[must_use]
    pub fn conversation(context_id: &ContextId) -> Self {
        let id = urlencoding::encode(context_id.as_str());
        Self {
            json_href: format!("/admin/export/transcripts/{id}?format=json"),
            markdown_href: format!("/admin/export/transcripts/{id}?format=markdown"),
        }
    }
}
