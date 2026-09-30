//! Analytics domain of the admin dashboard.
//!
//! What the AI on this instance did, and what it cost.
//!
//! - [`repositories`] — requests, sessions, traces, artifacts and tools; the
//!   conversation and skill analysis facts and the reports built from them; the
//!   dashboard, overview and customer/internal report rollups; the
//!   observability exporter state; and the session state every activity page
//!   derives.
//! - [`routes`] — the route fragments the admin composition root merges: the
//!   overview, the AI-activity and analysis pages, the account history, the
//!   table and transcript exports, the observability page, the dashboard and
//!   events API, and the `/hooks/track` intake.
//! - [`test_support`] — the transcript view and the skill-key parser, read by
//!   the conversation judge job and the tests.
//!
//! The export datasets register with core through `inventory`, so a binary
//! that serves exports must link this crate.

pub(crate) mod export;
pub(crate) mod handlers;
pub mod repositories;
pub mod routes;
pub(crate) mod services;

pub mod test_support {
    pub use crate::handlers::ssr::analysis::{SkillRef, parse_skill_key};
    pub use crate::handlers::ssr::transcript_view::{
        ConversationView, EmptyReason, ParsedAssistant, SideCallRowView, SideCallsView, StepView,
        ThreadView, ToolChipView, ToolUseMarker, TranscriptMetaView, TranscriptOptions,
        TranscriptRequestIds, TurnView, build_conversation, meta_view, parse_assistant, preview,
        short_id, strip_system_reminders, tidy_lines, transcript_request_ids,
    };
}
