//! The ten tiles of the overview strip, in three rows of intent: how much
//! happened (requests, spend, people, conversations), what the work was made
//! of (tokens and cache, tool calls), and how well it went (latency, errors,
//! denials).
//!
//! Every tile links to the page that owns exactly the rows it counted, with
//! the window carried along, so the overview stays a place to start from.

use crate::repositories::overview::kpis::OverviewKpis;
use systemprompt_web_admin_core::ssr::format::{
    format_cost, format_duration_ms, format_token_total, short_num,
};
use systemprompt_web_admin_core::util::delta::delta;

use super::kpis_view::{KpiTileView, Tile, TileInputs, comparison, percent_tenths, tile};

pub(super) fn volume_tiles(k: &OverviewKpis, i: &TileInputs<'_>) -> Vec<KpiTileView> {
    let preset = i.range.preset();
    let vs = i.range.previous_label();
    vec![
        tile(
            Tile {
                label: "Requests",
                icon: "bolt",
                value: short_num(k.requests),
                note: comparison(&short_num(k.prev_requests), vs),
                hint: "gateway requests in the window".to_owned(),
                href: format!("/admin/requests?preset={preset}"),
                tone: "",
            },
            i.comparable
                .then(|| delta(k.requests, k.prev_requests, true)),
            &i.column(|p| p.requests),
        ),
        tile(
            Tile {
                label: "Spend",
                icon: "coins",
                value: format_cost(k.cost_microdollars),
                note: comparison(&format_cost(k.prev_cost_microdollars), vs),
                hint: format!(
                    "{} across the window · {} per request",
                    format_cost(k.cost_microdollars),
                    format_cost(k.cost_microdollars / k.requests.max(1))
                ),
                href: format!("/admin/analytics?tab=cost&preset={preset}"),
                tone: "",
            },
            i.comparable
                .then(|| delta(k.cost_microdollars, k.prev_cost_microdollars, false)),
            &i.column(|p| p.cost_microdollars),
        ),
        tile(
            Tile {
                label: "Active people",
                icon: "people",
                value: k.active_users.to_string(),
                note: comparison(&k.prev_active_users.to_string(), vs),
                hint: format!("{} distinct accounts made a request", k.active_users),
                href: "/admin/users".to_owned(),
                tone: "",
            },
            i.comparable
                .then(|| delta(k.active_users, k.prev_active_users, true)),
            &i.column(|p| p.users),
        ),
        tile(
            Tile {
                label: "Conversations",
                icon: "chat",
                value: short_num(k.conversations),
                note: comparison(&short_num(k.prev_conversations), vs),
                hint: format!(
                    "{} distinct conversations carried a request · {} requests each",
                    k.conversations,
                    k.requests / k.conversations.max(1)
                ),
                href: format!("/admin/analysis/conversations?preset={preset}"),
                tone: "",
            },
            i.comparable
                .then(|| delta(k.conversations, k.prev_conversations, true)),
            &i.column(|p| p.conversations),
        ),
    ]
}

pub(super) fn work_tiles(k: &OverviewKpis, i: &TileInputs<'_>) -> Vec<KpiTileView> {
    let preset = i.range.preset();
    let vs = i.range.previous_label();
    let cache = k.cache_share_percent();
    vec![
        tile(
            Tile {
                label: "Tokens",
                icon: "token",
                value: format_token_total(k.tokens),
                note: if k.input_tokens + k.cache_tokens > 0 {
                    format!("{cache}% read from cache")
                } else {
                    comparison(&format_token_total(k.prev_tokens), vs)
                },
                hint: format!(
                    "{} input + output tokens ({} {}) · {} cache reads",
                    format_token_total(k.tokens),
                    format_token_total(k.prev_tokens),
                    vs,
                    format_token_total(k.cache_tokens)
                ),
                href: format!("/admin/analytics?tab=models&preset={preset}"),
                tone: cache_tone(k),
            },
            i.comparable.then(|| delta(k.tokens, k.prev_tokens, true)),
            &i.column(|p| p.tokens),
        ),
        tile(
            Tile {
                label: "Tool calls",
                icon: "wrench",
                value: short_num(k.tool_calls),
                note: comparison(&short_num(k.prev_tool_calls), vs),
                hint: format!(
                    "{} tool calls the models made · {:.1} per request",
                    k.tool_calls,
                    k.tool_calls as f64 / k.requests.max(1) as f64
                ),
                href: format!("/admin/analytics?tab=tools&preset={preset}"),
                tone: "",
            },
            i.comparable
                .then(|| delta(k.tool_calls, k.prev_tool_calls, true)),
            &i.column(|p| p.tool_calls),
        ),
    ]
}

pub(super) fn health_tiles(k: &OverviewKpis, i: &TileInputs<'_>) -> Vec<KpiTileView> {
    let preset = i.range.preset();
    let vs = i.range.previous_label();
    vec![
        tile(
            Tile {
                label: "p50 latency",
                icon: "clock",
                value: format_duration_ms(k.p50_latency_ms),
                note: comparison(&format_duration_ms(k.prev_p50_latency_ms), vs),
                hint: "half of routed requests finished faster than this".to_owned(),
                href: format!("/admin/traces?preset={preset}"),
                tone: "",
            },
            i.comparable
                .then(|| delta(k.p50_latency_ms, k.prev_p50_latency_ms, false)),
            &i.column(|p| p.p50_latency_ms),
        ),
        tile(
            Tile {
                label: "p95 latency",
                icon: "gauge",
                value: format_duration_ms(k.p95_latency_ms),
                note: comparison(&format_duration_ms(k.prev_p95_latency_ms), vs),
                hint: "95% of routed requests finished faster than this".to_owned(),
                href: format!("/admin/traces?preset={preset}"),
                tone: latency_tone(k.p95_latency_ms),
            },
            i.comparable
                .then(|| delta(k.p95_latency_ms, k.prev_p95_latency_ms, false)),
            &i.column(|p| p.p95_latency_ms),
        ),
        tile(
            Tile {
                label: "Error rate",
                icon: "alert",
                value: percent_tenths(k.error_rate_tenths(), k.requests),
                note: comparison(
                    &percent_tenths(k.prev_error_rate_tenths(), k.prev_requests),
                    vs,
                ),
                hint: format!("{} failed of {}", k.errors, short_num(k.requests)),
                href: format!("/admin/requests?status=failed&preset={preset}"),
                tone: if k.errors > 0 { "err" } else { "" },
            },
            i.comparable
                .then(|| delta(k.error_rate_tenths(), k.prev_error_rate_tenths(), false)),
            &i.column(|p| p.errors),
        ),
        tile(
            Tile {
                label: "Denied",
                icon: "shield",
                value: k.denied.to_string(),
                note: comparison(&k.prev_denied.to_string(), vs),
                hint: "refused by policy or quota before routing".to_owned(),
                href: format!("/admin/requests?status=rejected&preset={preset}"),
                tone: if k.denied > 0 { "warn" } else { "" },
            },
            i.comparable.then(|| delta(k.denied, k.prev_denied, false)),
            &i.column(|p| p.rejected),
        ),
    ]
}

const fn cache_tone(k: &OverviewKpis) -> &'static str {
    let total = k.input_tokens + k.cache_tokens;
    if total == 0 {
        ""
    } else if k.cache_tokens * 2 >= total {
        "ok"
    } else if k.cache_tokens * 5 >= total {
        "warn"
    } else {
        "err"
    }
}

const LATENCY_OK_P95_MS: i64 = 8_000;
const GATEWAY_P95_TARGET_MS: i64 = 20_000;

const fn latency_tone(p95_ms: i64) -> &'static str {
    match p95_ms {
        0 => "",
        ms if ms <= LATENCY_OK_P95_MS => "ok",
        ms if ms <= GATEWAY_P95_TARGET_MS => "warn",
        _ => "err",
    }
}
