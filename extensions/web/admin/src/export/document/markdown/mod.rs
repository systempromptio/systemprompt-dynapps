//! A conversation bundle rendered for reading: the transcript turn by turn
//! with each tool call's input and result, then the planes around it as
//! tables. The JSON document is the record; this is the same record laid out
//! for a person. A user prompt is block-quoted: it is often a skill body with
//! headings of its own, and quoted it stays inside its turn instead of
//! joining the document's outline.

use super::bundle::ConversationBundle;
use crate::export::format::usd;
use crate::handlers::ssr::transcript_view::{StepView, ThreadView};

// Why: `std::fmt::Write` into a `String` cannot fail, but `writeln!` still
// returns a `Result` that every line would have to discard.
macro_rules! line {
    ($out:expr, $($arg:tt)*) => {{
        $out.push_str(&format!($($arg)*));
        $out.push('\n');
    }};
}

mod tables;

use tables::planes;

pub(crate) fn render(b: &ConversationBundle) -> String {
    let mut out = String::new();
    let title = b.title.as_deref().unwrap_or("Conversation");
    line!(out, "# {title}\n");
    line!(out, "- Context: `{}`", b.context_id);
    if let Some(name) = &b.header.display_name {
        line!(out, "- Person: {name}");
    }
    if let Some(session) = &b.header.client_session_id {
        line!(out, "- Harness session: `{session}`");
    }
    if let Some(f) = &b.facts {
        line!(
            out,
            "- Client: {} · Models: {} · {} → {}",
            f.client_kind,
            f.models.join(", "),
            f.first_at.to_rfc3339(),
            f.last_at.to_rfc3339()
        );
        line!(
            out,
            "- Requests: {} ({} turns) · Tokens: {} in / {} out · Cost: ${} · Tool calls: {} · Errors: {}",
            f.request_count,
            f.turn_count,
            f.input_tokens,
            f.output_tokens,
            usd(f.cost_microdollars),
            f.tool_calls_intended,
            f.error_count
        );
        if let Some(score) = f.completion {
            line!(
                out,
                "- Judge: {score}/100 · {} · {}",
                f.outcome.as_deref().unwrap_or("—"),
                f.summary.as_deref().unwrap_or("")
            );
        }
    }
    line!(
        out,
        "- Exported: {}{}",
        b.exported_at.to_rfc3339(),
        if b.redacted { " (redacted)" } else { "" }
    );
    line!(out, "\n## Transcript\n");
    for thread in &b.transcript.threads {
        thread_section(&mut out, thread);
    }
    if b.transcript.threads.is_empty() {
        line!(
            out,
            "_No message bodies were stored for this conversation._\n"
        );
    }
    planes(&mut out, b);
    out
}

fn thread_section(out: &mut String, thread: &ThreadView) {
    if !thread.is_main {
        line!(out, "### Thread {} — {}\n", thread.index, thread.label);
    }
    if let Some(system) = &thread.system_prompt {
        line!(
            out,
            "<details><summary>System prompt</summary>\n\n```text\n{system}\n```\n\n</details>\n"
        );
    }
    for turn in &thread.turns {
        line!(out, "### Turn {} — {}\n", turn.number, turn.ts_full);
        line!(out, "**User**\n\n{}\n", quoted(&turn.prompt));
        for step in &turn.steps {
            step_block(out, step);
        }
    }
}

fn step_block(out: &mut String, step: &StepView) {
    if step.is_tool {
        line!(
            out,
            "**Tool** `{}`\n",
            step.tool_name.as_deref().unwrap_or("?")
        );
        if let Some(input) = &step.tool_input_pretty {
            line!(out, "```json\n{input}\n```\n");
        }
        if let Some(result) = &step.tool_result_pretty {
            line!(out, "Result:\n\n```json\n{result}\n```\n");
        }
        return;
    }
    if step.is_assistant {
        let meta = step
            .meta
            .as_ref()
            .map(|m| {
                format!(
                    " _({} · {} · {} · {})_",
                    m.model, m.status, m.latency_display, m.cost_display
                )
            })
            .unwrap_or_default();
        line!(out, "**Assistant**{meta}\n");
    }
    if let Some(text) = &step.text {
        line!(out, "{text}\n");
    }
}

fn quoted(text: &str) -> String {
    text.lines()
        .map(|l| format!("> {l}"))
        .collect::<Vec<_>>()
        .join("\n")
}
