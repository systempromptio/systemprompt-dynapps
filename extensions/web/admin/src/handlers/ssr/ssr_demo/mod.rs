//! The Demo section: what the enterprise demo actually did, as the platform
//! recorded it.
//!
//! Two pages over one data layer — skill adoption and MCP tool usage — both
//! admin-only.
//!
//! Token and cost figures are *attributed*, never measured: see
//! [`crate::repositories::demo::attribution`] for the window rule that
//! [`ATTRIBUTION_NOTE`] states on every page.

mod context;
mod matrix_view;
mod skills;
mod tools;
mod view;

use crate::handlers::ssr::format::format_token_total;
use crate::repositories::demo::kpis::DemoKpis;
use view::{AttributedTotals, KpiView, ToolVerdictTotals, format_demo_cost};

pub(crate) use skills::demo_skills_page;
pub(crate) use tools::demo_tools_page;

const CHART_DAYS: i32 = 14;

const ATTRIBUTION_NOTE: &str = "Tokens and cost are attributed, not \
     measured: an AI request counts toward an invocation when it belongs to the \
     same user and falls between that invocation and the next one of its kind in \
     the session, or the session's last event plus five minutes. An invocation \
     is either a slash command the user typed or a Skill tool call the model \
     dispatched; a Skill tool call with no matching governance decision beside \
     it did not come from a client and is not counted.";

const fn kpi(label: &'static str, value: String) -> KpiView {
    KpiView {
        label,
        value,
        testid: "",
        variant: "",
    }
}

const fn kpi_tagged(label: &'static str, value: String, testid: &'static str) -> KpiView {
    KpiView {
        label,
        value,
        testid,
        variant: "",
    }
}

const fn kpi_variant(label: &'static str, value: String, variant: &'static str) -> KpiView {
    KpiView {
        label,
        value,
        testid: "",
        variant,
    }
}

// Why: the two screenshot test ids ride on these builders, so a page that
// prints skill invocations or MCP calls always tags them, and a page that
// prints neither does not invent a card to carry a tag.
fn skill_invocations_kpi(kpis: &DemoKpis) -> KpiView {
    kpi_tagged(
        "Skill invocations",
        kpis.skill_invocations.to_string(),
        "demo-kpi-skill-invocations",
    )
}

fn mcp_calls_kpi(kpis: &DemoKpis) -> KpiView {
    kpi_tagged(
        "MCP tool calls",
        kpis.mcp_calls.to_string(),
        "demo-kpi-mcp-calls",
    )
}

// Why: the two figures are folded from the rows the page's own table lists, so
// a strip can never disagree with the table under it.
fn attributed_kpis(usage: &AttributedTotals) -> [KpiView; 2] {
    [
        kpi("Attributed tokens", format_token_total(usage.total_tokens)),
        kpi("Attributed cost", format_demo_cost(usage.cost_microdollars)),
    ]
}

fn skill_kpi_strip(
    kpis: &DemoKpis,
    distinct_skills: i64,
    distinct_users: i64,
    usage: &AttributedTotals,
) -> Vec<KpiView> {
    let mut strip = vec![
        skill_invocations_kpi(kpis),
        kpi("Distinct skills", distinct_skills.to_string()),
        kpi("Distinct users", distinct_users.to_string()),
    ];
    strip.extend(attributed_kpis(usage));
    strip
}

fn tool_kpi_strip(
    kpis: &DemoKpis,
    verdicts: &ToolVerdictTotals,
    usage: &AttributedTotals,
) -> Vec<KpiView> {
    let mut strip = vec![
        mcp_calls_kpi(kpis),
        kpi_variant("Failures", kpis.mcp_failures.to_string(), "danger"),
        kpi_variant("Allowed", verdicts.allowed.to_string(), "success"),
        kpi_variant("Denied", verdicts.denied.to_string(), "danger"),
        kpi_variant("Held", verdicts.held.to_string(), "warning"),
        kpi_variant("Approved", verdicts.approved.to_string(), "success"),
    ];
    strip.extend(attributed_kpis(usage));
    strip
}
