//! Assembling the conversation detail page context.

use super::figures::{charts, kpis};
use super::rows::{decision_view, judge_view, safety_view, skill_view, tool_view, turn_view};
use super::views::{ConversationDetailContext, LedgerCallView};
use crate::repositories::analysis::conversations::planes::{
    ConversationDecisionRow, ConversationSafetyRow, ConversationSkillRow, ConversationToolCallRow,
    ConversationTurnRow,
};
use crate::repositories::analysis::conversations::{ConversationFactRow, ConversationFacts};
use systemprompt_web_admin_core::ssr::analysis_urls::{
    ANALYSIS_CONVERSATIONS_URL, analysis_conversation_url,
};
use systemprompt_web_admin_core::ssr::format::{client_chip, format_duration_ms, local_time};
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;

pub(super) struct Planes {
    pub(super) turns: Vec<ConversationTurnRow>,
    pub(super) tools: Vec<ConversationToolCallRow>,
    pub(super) decisions: Vec<ConversationDecisionRow>,
    pub(super) skills: Vec<ConversationSkillRow>,
    pub(super) safety: Vec<ConversationSafetyRow>,
}

fn artifact_views(tools: &[ConversationToolCallRow]) -> Vec<LedgerCallView> {
    tools
        .iter()
        .filter(|t| t.artifact_kind.is_some())
        .cloned()
        .map(tool_view)
        .collect()
}

fn display_name(f: &ConversationFacts) -> String {
    f.display_name
        .clone()
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| f.user_id.as_str().to_owned())
}

fn heading(f: &ConversationFacts) -> String {
    f.judge_title
        .clone()
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| f.title.clone())
}

fn session_href(f: &ConversationFacts) -> Option<String> {
    f.client_session_id
        .as_deref()
        .map(|s| format!("/admin/sessions/{}", urlencoding::encode(s)))
}

pub(super) fn build_context(
    row: &ConversationFactRow,
    planes: Planes,
    can_judge: bool,
) -> ConversationDetailContext {
    let (context_id, f) = (&row.context_id, &row.facts);
    let Planes {
        turns,
        tools,
        decisions,
        skills,
        safety,
    } = planes;
    let (turn_count, tool_count) = (turns.len(), tools.len());
    let (decision_count, skill_count, safety_count) = (decisions.len(), skills.len(), safety.len());
    let artifacts = artifact_views(&tools);
    let turn_charts = charts(&turns);
    let heading = heading(f);
    let short = context_id.as_str()[..context_id.as_str().len().min(8)].to_owned();
    ConversationDetailContext {
        page: "analysis-conversation",
        title: format!("{heading} · Conversation"),
        crumb: heading.clone(),
        breadcrumbs: vec![
            BreadcrumbView::link("Analysis", ANALYSIS_CONVERSATIONS_URL),
            BreadcrumbView::link("Conversations", ANALYSIS_CONVERSATIONS_URL),
            BreadcrumbView::current(short),
        ],
        context_key: context_id.as_str().to_owned(),
        heading,
        user_href: format!("/admin/users/{}", urlencoding::encode(f.user_id.as_str())),
        display_name: display_name(f),
        group_name: f.group_name.clone(),
        project_name: f.project_name.clone(),
        client: client_chip(&f.client_kind, &f.client_attestation),
        wire_protocol: f.wire_protocol.clone(),
        models: f.models.clone(),
        providers: f.providers.clone(),
        first_display: local_time(f.first_at),
        last_display: local_time(f.last_at),
        duration_display: format_duration_ms(f.duration_seconds * 1000),
        hook_status: f
            .hook_status
            .clone()
            .unwrap_or_else(|| "no hooks".to_owned()),
        session_href: session_href(f),
        context_href: format!(
            "/admin/contexts/{}",
            urlencoding::encode(context_id.as_str())
        ),
        back_url: ANALYSIS_CONVERSATIONS_URL,
        help: crate::handlers::ssr::analysis::help::conversation_detail_help(),
        export: systemprompt_web_admin_core::export::view::ExportView::conversation(
            context_id, true,
        ),
        judge: judge_view(
            f,
            format!("{}/judge", analysis_conversation_url(context_id)),
        ),
        can_judge,
        kpis: kpis(f),
        charts: turn_charts,
        turns: turns.into_iter().enumerate().map(turn_view).collect(),
        turn_count,
        tool_count,
        artifact_count: artifacts.len(),
        artifacts,
        tool_calls: tools.into_iter().map(tool_view).collect(),
        decisions: decisions.into_iter().map(decision_view).collect(),
        decision_count,
        skills: skills.into_iter().map(skill_view).collect(),
        skill_count,
        safety: safety.into_iter().map(safety_view).collect(),
        safety_count,
        documents: crate::handlers::ssr::analysis::documents_panel::DocumentsPanelView::default(),
    }
}
