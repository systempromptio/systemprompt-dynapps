//! Assembles the slices of a session a criterion asks to see.
//!
//! Each `InputSlice` maps to one existing reader; the judge prompt is built
//! from these labelled blocks in the order the criterion lists them, so the
//! dashboard's description of a criterion's inputs is also the order the
//! model reads them in.

use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};

use crate::repositories::analytics::conversations::{RawTurnBody, find_raw_turns};
use crate::repositories::dashboard::{conversation_analytics, hooks_track};
use crate::types::ENTITY_SKILL;
use crate::types::conversation_analytics::SessionEntityLink;

use super::metrics::{SessionMetrics, get_session_metrics};
use super::{InputSlice, activity};

const TRANSCRIPT_TAIL_TURNS: usize = 12;
const TRANSCRIPT_TURN_CHARS: usize = 2_000;

// Why: one labelled block of the user prompt, in criterion order.
#[derive(Debug, Clone)]
pub struct InputBlock {
    pub slice: InputSlice,
    pub label: &'static str,
    pub body: String,
}

#[derive(Debug, Clone, Default)]
pub struct AssembledInputs {
    pub blocks: Vec<InputBlock>,
    pub metrics: Option<SessionMetrics>,
}

pub async fn assemble_inputs(
    pool: &PgPool,
    user_id: &UserId,
    session: &SessionId,
    slices: &[InputSlice],
) -> Result<AssembledInputs, sqlx::Error> {
    let mut out = AssembledInputs::default();
    for slice in slices {
        let (label, body) = match slice {
            InputSlice::UserMessages => (
                "USER MESSAGES (chronological)",
                format_user_messages(
                    &hooks_track::list_user_messages(pool, session, user_id).await,
                ),
            ),
            InputSlice::LastAssistantMessage => (
                "LAST ASSISTANT MESSAGE",
                hooks_track::get_last_message(pool, session, user_id).await,
            ),
            InputSlice::TranscriptTail => (
                "TRANSCRIPT TAIL (most recent turns)",
                format_transcript_tail(find_raw_turns(pool, session).await?),
            ),
            InputSlice::Skills => (
                "SKILLS USED",
                format_skills(
                    &conversation_analytics::list_session_entity_links(pool, session).await?,
                ),
            ),
            InputSlice::SessionMetadata => (
                "SESSION METADATA",
                format_session_metadata(
                    hooks_track::find_session_metrics(pool, session)
                        .await
                        .as_ref(),
                    hooks_track::find_session_timing(pool, session, user_id)
                        .await
                        .as_ref(),
                ),
            ),
            InputSlice::Activity => (
                "ACTIVITY",
                activity::generate_session_summary(pool, user_id, session)
                    .await
                    .map(|s| {
                        if s.tags.is_empty() {
                            s.summary
                        } else {
                            format!("{}\nTags: {}", s.summary, s.tags)
                        }
                    })
                    .unwrap_or_default(),
            ),
            InputSlice::Metrics => {
                let metrics = get_session_metrics(pool, user_id, session).await?;
                // Why: discard-ok: an in-memory struct always serialises
                let body = serde_json::to_string_pretty(&metrics).unwrap_or_default();
                out.metrics = Some(metrics);
                ("METRICS (json)", body)
            },
        };
        let body = if body.trim().is_empty() {
            "(none recorded)".to_owned()
        } else {
            body
        };
        out.blocks.push(InputBlock {
            slice: *slice,
            label,
            body,
        });
    }
    Ok(out)
}

fn format_user_messages(messages: &[String]) -> String {
    messages
        .iter()
        .enumerate()
        .map(|(i, msg)| format!("{}. \"{}\"", i + 1, msg))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_skills(links: &[SessionEntityLink]) -> String {
    links
        .iter()
        .filter(|e| e.entity_type == ENTITY_SKILL)
        .map(|e| format!("{} (x{})", e.entity_name, e.usage_count))
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_transcript_tail(turns: Option<Vec<RawTurnBody>>) -> String {
    let Some(turns) = turns else {
        return String::new();
    };
    let start = turns.len().saturating_sub(TRANSCRIPT_TAIL_TURNS);
    turns[start..]
        .iter()
        .map(|turn| {
            let mut content = turn.content.clone();
            if content.len() > TRANSCRIPT_TURN_CHARS {
                let cut = content
                    .char_indices()
                    .nth(TRANSCRIPT_TURN_CHARS)
                    .map_or(content.len(), |(i, _)| i);
                content.truncate(cut);
                content.push_str(" …");
            }
            format!("[turn {}] {content}", turn.ordinal)
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn format_session_metadata(
    m: Option<&hooks_track::SessionMetricsRow>,
    t: Option<&hooks_track::SessionTimingRow>,
) -> String {
    let mut parts = Vec::new();
    if let Some(m) = m {
        let mut meta = Vec::new();
        if !m.client_source.is_empty() {
            meta.push(format!("Client={}", m.client_source));
        }
        if !m.permission_mode.is_empty() {
            meta.push(format!("Mode={}", m.permission_mode));
        }
        if !m.model.is_empty() {
            meta.push(format!("Model={}", m.model));
        }
        if !meta.is_empty() {
            parts.push(meta.join(", "));
        }
        let user_p = m.user_prompts.unwrap_or(0);
        let auto_a = m.automated_actions.unwrap_or(0);
        let mut session = Vec::new();
        if user_p > 0 || auto_a > 0 {
            session.push(format!(
                "{user_p} user prompts, {auto_a} automated tool calls"
            ));
        } else {
            session.push(format!("{} prompts", m.prompts));
        }
        session.push(format!(
            "{} files touched",
            m.unique_files_touched.unwrap_or(0)
        ));
        if m.errors > 0 {
            session.push(format!("{} errors", m.errors));
        }
        if m.subagent_spawns > 0 {
            session.push(format!("{} subagents", m.subagent_spawns));
        }
        parts.push(session.join(", "));
    }
    if let Some(t) = t
        && let (Some(started), Some(ended)) = (t.started, t.ended)
    {
        let minutes = (ended - started).num_seconds().max(0) / 60;
        parts.push(format!(
            "Started {}, ended {} ({minutes} minutes)",
            started.format("%Y-%m-%d %H:%M"),
            ended.format("%H:%M")
        ));
    }
    parts.join("\n")
}
