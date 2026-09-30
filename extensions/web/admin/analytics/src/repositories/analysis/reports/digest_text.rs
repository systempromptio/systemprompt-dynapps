//! The digest rendered as the compact text the model is handed: one line per
//! entity with its id first, so the model can cite an id verbatim and the
//! report builder can turn it back into a console link.

use systemprompt_web_shared::format::format_cost as usd;

use super::digest::{DigestJudged, ReportDigest};

fn score(avg: Option<f64>) -> String {
    avg.map_or_else(|| "n/a".to_owned(), |v| format!("{v:.0}"))
}

#[must_use]
pub fn render_digest_text(d: &ReportDigest) -> String {
    let t = &d.totals;
    let mut out = String::new();
    out.push_str(&format!(
        "WINDOW {} → {}\nTOTALS conversations={} people={} turns={} tokens={} cost={} failed_requests={} denied_tool_calls={} safety_findings={} tool_calls={} artifacts={} judged={} mean_completion={} achieved={} partial={} abandoned={} unclear={}\n",
        d.window_start.format("%Y-%m-%d"), d.window_end.format("%Y-%m-%d"),
        t.conversations, t.people, t.turns, t.tokens, usd(t.cost_microdollars), t.errors,
        t.denied, t.safety_findings, t.tool_calls, t.artifacts, t.judged,
        score(t.completion_avg), t.achieved, t.partial, t.abandoned, t.unclear
    ));
    out.push_str("\nSKILLS (id | invocations | conversations | cost | mean_completion | errors)\n");
    for s in &d.skills {
        out.push_str(&format!(
            "skill {} | {} | {} | {} | {} | {}\n",
            s.skill,
            s.invocations,
            s.conversations,
            usd(s.cost_microdollars),
            score(s.completion_avg),
            s.errors
        ));
    }
    out.push_str("\nMODELS (id | conversations | requests | cost | failed | mean_completion)\n");
    for m in &d.models {
        out.push_str(&format!(
            "model {} | {} | {} | {} | {} | {}\n",
            m.model,
            m.conversations,
            m.requests,
            usd(m.cost_microdollars),
            m.errors,
            score(m.completion_avg)
        ));
    }
    out.push_str("\nPEOPLE (id | name | conversations | cost | mean_completion | errors)\n");
    for p in &d.people {
        out.push_str(&format!(
            "person {} | {} | {} | {} | {} | {}\n",
            p.user_key,
            p.display_name.as_deref().unwrap_or("-"),
            p.conversations,
            usd(p.cost_microdollars),
            score(p.completion_avg),
            p.errors
        ));
    }
    push_judged(&mut out, "WORST JUDGED", &d.worst);
    push_judged(&mut out, "BEST JUDGED", &d.best);
    push_mix(&mut out, d);
    out
}

fn push_judged(out: &mut String, heading: &str, rows: &[DigestJudged]) {
    out.push_str(&format!(
        "\n{heading} (id | title | completion | outcome | intent | cost | turns | summary)\n"
    ));
    for j in rows {
        out.push_str(&format!(
            "conversation {} | {} | {} | {} | {} | {} | {} | {}\n",
            j.context_key,
            j.judge_title.as_deref().unwrap_or(&j.title),
            j.completion
                .map_or_else(|| "n/a".to_owned(), |c| c.to_string()),
            j.outcome.as_deref().unwrap_or("-"),
            j.category.as_deref().unwrap_or("-"),
            usd(j.cost_microdollars),
            j.turns,
            j.summary.as_deref().unwrap_or("")
        ));
    }
}

fn push_mix(out: &mut String, d: &ReportDigest) {
    out.push_str("\nCOST OUTLIERS (id | title | cost | turns | tokens | completion | model)\n");
    for o in &d.outliers {
        out.push_str(&format!(
            "conversation {} | {} | {} | {} | {} | {} | {}\n",
            o.context_key,
            o.title,
            usd(o.cost_microdollars),
            o.turns,
            o.tokens,
            o.completion
                .map_or_else(|| "n/a".to_owned(), |c| c.to_string()),
            o.model.as_deref().unwrap_or("-")
        ));
    }
    out.push_str("\nDENIED TOOL CALLS (tool | denied | people)\n");
    for x in &d.denials {
        out.push_str(&format!(
            "tool {} | {} | {}\n",
            x.tool_name, x.denied, x.people
        ));
    }
    out.push_str("\nINTENT MIX (category | conversations | mean_completion)\n");
    for i in &d.intents {
        out.push_str(&format!(
            "{} | {} | {}\n",
            i.category,
            i.conversations,
            score(i.completion_avg)
        ));
    }
    out.push_str("\nCLIENT MIX (client | conversations | cost)\n");
    for c in &d.clients {
        out.push_str(&format!(
            "{} | {} | {}\n",
            c.client_kind,
            c.conversations,
            usd(c.cost_microdollars)
        ));
    }
}
