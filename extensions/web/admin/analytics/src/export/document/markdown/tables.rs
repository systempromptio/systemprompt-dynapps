//! The planes around a conversation as Markdown tables: requests, tool
//! ledger, governance decisions, skills, safety findings and hook events.
//! Every free-text cell is flattened to one line — a pipe or newline would
//! break the row — and clipped, since the JSON document carries it in full.

use crate::export::document::bundle::ConversationBundle;
use systemprompt_web_admin_core::export::format::usd;

const TABLE_TEXT_CHARS: usize = 160;

pub(super) fn planes(out: &mut String, b: &ConversationBundle) {
    requests_table(out, b);
    ledger_table(out, b);
    governance_tables(out, b);
    hook_table(out, b);
}

fn requests_table(out: &mut String, b: &ConversationBundle) {
    line!(out, "## Requests\n");
    line!(
        out,
        "| # | At | Kind | Model | Status | In | Out | Cost | Latency | Tools |\n|---|---|---|---|---|---|---|---|---|---|"
    );
    for (i, r) in b.requests.iter().enumerate() {
        line!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            i + 1,
            r.created_at.to_rfc3339(),
            r.kind,
            r.model.as_deref().unwrap_or("—"),
            r.status,
            r.input_tokens.unwrap_or_default(),
            r.output_tokens.unwrap_or_default(),
            usd(r.cost_microdollars),
            r.latency_ms
                .map_or_else(|| "—".to_owned(), |ms| format!("{ms} ms")),
            r.tool_names.join(", ")
        );
    }
}

fn ledger_table(out: &mut String, b: &ConversationBundle) {
    line!(out, "\n## Tool ledger\n");
    line!(
        out,
        "| At | Tool | State | Status | Duration | Result | Error |\n|---|---|---|---|---|---|---|"
    );
    for t in &b.tool_ledger {
        line!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} |",
            t.occurred_at.map(|a| a.to_rfc3339()).unwrap_or_default(),
            cell(t.tool_name.as_deref().unwrap_or("—")),
            t.state,
            t.execution_status.as_deref().unwrap_or("—"),
            t.execution_time_ms
                .map_or_else(|| "—".to_owned(), |ms| format!("{ms} ms")),
            t.artifact_kind.as_deref().unwrap_or("—"),
            cell(t.error_message.as_deref().unwrap_or(""))
        );
    }
}

fn governance_tables(out: &mut String, b: &ConversationBundle) {
    line!(out, "\n## Governance decisions\n");
    line!(
        out,
        "| At | Tool | Decision | Policy | Reason |\n|---|---|---|---|---|"
    );
    for d in &b.decisions {
        line!(
            out,
            "| {} | {} | {} | {} | {} |",
            d.created_at.to_rfc3339(),
            cell(&d.tool_name),
            d.decision,
            d.policy,
            cell(&d.reason)
        );
    }
    line!(out, "\n## Skills\n");
    line!(
        out,
        "| Skill | Plugin | Marketplace | Version | Invocations | First |\n|---|---|---|---|---|---|"
    );
    for s in &b.skills {
        line!(
            out,
            "| {} | {} | {} | {} | {} | {} |",
            s.skill,
            s.plugin_id.as_ref().map_or("—", |p| p.as_str()),
            s.marketplace_id.as_ref().map_or("—", |m| m.as_str()),
            s.marketplace_hash
                .as_deref()
                .map_or("—", |h| &h[..h.len().min(12)]),
            s.invocations,
            s.first_invoked_at.to_rfc3339()
        );
    }
    line!(out, "\n## Safety findings\n");
    line!(
        out,
        "| At | Phase | Category | Severity | Scanner | Blocked | Excerpt |\n|---|---|---|---|---|---|---|"
    );
    for s in &b.safety_findings {
        line!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} |",
            s.created_at.to_rfc3339(),
            s.phase,
            s.category,
            s.severity,
            s.scanner,
            s.blocked,
            cell(s.excerpt.as_deref().unwrap_or(""))
        );
    }
}

fn hook_table(out: &mut String, b: &ConversationBundle) {
    line!(out, "\n## Hook events\n");
    line!(
        out,
        "| At | Event | Tool | Description |\n|---|---|---|---|"
    );
    for e in &b.hook_events {
        line!(
            out,
            "| {} | {} | {} | {} |",
            e.created_at.to_rfc3339(),
            e.event_type,
            cell(e.tool_name.as_deref().unwrap_or("")),
            cell(
                e.description
                    .as_deref()
                    .or(e.prompt_preview.as_deref())
                    .unwrap_or("")
            )
        );
    }
}

fn cell(text: &str) -> String {
    let flat: String = text
        .chars()
        .map(|c| {
            if c == '\n' || c == '\r' {
                ' '
            } else if c == '|' {
                '¦'
            } else {
                c
            }
        })
        .collect();
    if flat.chars().count() > TABLE_TEXT_CHARS {
        let head: String = flat.chars().take(TABLE_TEXT_CHARS).collect();
        format!("{head}…")
    } else {
        flat
    }
}
