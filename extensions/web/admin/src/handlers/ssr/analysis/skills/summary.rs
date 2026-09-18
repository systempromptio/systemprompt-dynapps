//! The KPI tiles, the invocations chart and the unused-skills list above and
//! below the Skills table.

use chrono::{Duration, NaiveDate};

use super::SPARK_DAYS;
use super::rows::spark_values;
use super::views::{SkillsKpiView, UnusedSkillView};
use crate::handlers::ssr::analysis::tone::{
    completion_tone, error_rate_tone, percent, score_display,
};
use crate::handlers::ssr::format::{format_cost, format_token_total};
use crate::handlers::ssr::types::{
    LineChartSpec, SvgLineChartView, SvgSeriesInput, line_chart, sparkline_toned,
};
use crate::repositories::analysis::inventory_index::{InventoryIndex, MarketplaceAudience};
use crate::repositories::analysis::skills::{MarketplaceAdoptionRow, SkillFactRow};

// Why: the window's headline figures, computed once and read by both tile
// groups.
struct Figures {
    spark_sum: Vec<i64>,
    completion: Option<f64>,
    judged: i64,
    installed: i64,
    entitled: i64,
    active: i64,
    invocations: i64,
    cost: i64,
}

impl Figures {
    fn new(
        rows: &[SkillFactRow],
        adoption: &[MarketplaceAdoptionRow],
        audience: &MarketplaceAudience,
        today: NaiveDate,
    ) -> Self {
        let judged_rows: Vec<&SkillFactRow> =
            rows.iter().filter(|r| r.completion_avg.is_some()).collect();
        let judged: i64 = judged_rows.iter().map(|r| r.judged).sum();
        let completion = (judged > 0).then(|| {
            judged_rows
                .iter()
                .map(|r| r.completion_avg.unwrap_or(0.0) * r.judged as f64)
                .sum::<f64>()
                / judged as f64
        });
        Self {
            spark_sum: (0..SPARK_DAYS)
                .map(|i| {
                    rows.iter()
                        .map(|r| spark_values(r, today)[usize::try_from(i).unwrap_or(0)])
                        .sum()
                })
                .collect(),
            completion,
            judged,
            installed: adoption.iter().map(|a| a.installed).sum(),
            entitled: i64::try_from(audience.users_reaching_any_marketplace()).unwrap_or(0),
            active: rows.iter().map(|r| r.users).max().unwrap_or(0),
            invocations: rows.iter().map(|r| r.invocations).sum(),
            cost: rows.iter().map(|r| r.cost_microdollars).sum(),
        }
    }
}

fn tile(
    label: &'static str,
    value: String,
    note: String,
    tone: &'static str,
    hint: &'static str,
) -> SkillsKpiView {
    SkillsKpiView {
        label,
        value,
        note,
        tone,
        hint,
        spark: sparkline_toned(&[], tone, String::new()),
    }
}

fn adoption_tiles(rows: &[SkillFactRow], g: &Figures) -> Vec<SkillsKpiView> {
    let sum = |f: fn(&SkillFactRow) -> i64| rows.iter().map(f).sum::<i64>();
    let invocations = g.invocations;
    let (installed, entitled, active) = (g.installed, g.entitled, g.active);
    let entitled_i = entitled;
    let mut tiles = vec![
        tile(
            "Skills used",
            rows.len().to_string(),
            format!("{} invocations in the window", invocations),
            "accent",
            "Distinct plugin:skill keys with at least one hook-reported invocation",
        ),
        tile(
            "People",
            active.to_string(),
            format!("of {entitled} entitled · {installed} installed"),
            "accent",
            "Most people any one skill reached; entitlement is resolved from the access-control rules",
        ),
        tile(
            "Install rate",
            percent(installed, entitled_i),
            format!("{installed} consumers hold a verified receipt"),
            if entitled_i > 0 && installed * 2 >= entitled_i {
                "ok"
            } else {
                "warn"
            },
            "Distinct consumers with a verified installation receipt over the people entitled to any marketplace",
        ),
        tile(
            "Conversations",
            sum(|r| r.conversations).to_string(),
            format!(
                "{} turns · {} tool calls",
                sum(|r| r.turns),
                sum(|r| r.tool_calls)
            ),
            "accent",
            "Gateway conversations whose harness session invoked a skill",
        ),
    ];
    if let Some(head) = tiles.first_mut() {
        head.spark = sparkline_toned(&g.spark_sum, head.tone, "Invocations per day".to_owned());
    }
    tiles
}

fn spend_tiles(rows: &[SkillFactRow], g: &Figures) -> Vec<SkillsKpiView> {
    let sum = |f: fn(&SkillFactRow) -> i64| rows.iter().map(f).sum::<i64>();
    let (cost, invocations, completion) = (g.cost, g.invocations, g.completion);
    let judged_total = g.judged;
    vec![
        tile(
            "Tokens",
            format_token_total(sum(|r| r.input_tokens + r.output_tokens)),
            format!("{} from cache", format_token_total(sum(|r| r.cache_tokens))),
            "accent",
            "Tokens the skill conversations ran up",
        ),
        tile(
            "Cost",
            format_cost(cost),
            format!("{} per invocation", format_cost(cost / invocations.max(1))),
            "accent",
            "Priced spend of the skill conversations",
        ),
        tile(
            "Errors",
            (sum(|r| r.errors)).to_string(),
            format!("failed requests · {} denied tool calls", sum(|r| r.denied)),
            error_rate_tone(sum(|r| r.errors), sum(|r| r.requests)),
            "Failed requests and governance denials inside skill conversations",
        ),
        tile(
            "Completed",
            score_display(completion),
            format!("{judged_total} conversations judged"),
            completion_tone(completion),
            "Mean of the judge's one completion score over judged skill conversations",
        ),
    ]
}

pub(super) fn kpis(
    rows: &[SkillFactRow],
    adoption: &[MarketplaceAdoptionRow],
    audience: &MarketplaceAudience,
    today: NaiveDate,
) -> Vec<SkillsKpiView> {
    let figures = Figures::new(rows, adoption, audience, today);
    let mut tiles = adoption_tiles(rows, &figures);
    tiles.extend(spend_tiles(rows, &figures));
    tiles
}

pub(super) fn chart(rows: &[SkillFactRow], today: NaiveDate) -> SvgLineChartView {
    let mut top: Vec<&SkillFactRow> = rows.iter().collect();
    top.sort_by_key(|r| std::cmp::Reverse(r.invocations));
    let labels: Vec<String> = (0..SPARK_DAYS)
        .rev()
        .map(|back| (today - Duration::days(back)).format("%b %-d").to_string())
        .collect();
    let mut series: Vec<SvgSeriesInput> = top
        .iter()
        .take(6)
        .map(|r| SvgSeriesInput {
            label: r.skill.clone(),
            values: spark_values(r, today),
            value_display: r.invocations.to_string(),
        })
        .collect();
    if top.len() > 6 {
        let rest: Vec<i64> = (0..SPARK_DAYS)
            .map(|i| {
                top.iter()
                    .skip(6)
                    .map(|r| spark_values(r, today)[usize::try_from(i).unwrap_or(0)])
                    .sum()
            })
            .collect();
        series.push(SvgSeriesInput {
            label: format!("{} other skills", top.len() - 6),
            value_display: rest.iter().sum::<i64>().to_string(),
            values: rest,
        });
    }
    line_chart(LineChartSpec {
        title: "Invocations per day",
        subtitle: format!("Last {SPARK_DAYS} days, top skills"),
        empty_message: "No skill was invoked in the last fourteen days",
        series,
        ref_lines: Vec::new(),
        y_max: None,
        y_display: |v| v.to_string(),
        x_start_display: labels.first().cloned().unwrap_or_default(),
        x_mid_display: labels.get(labels.len() / 2).cloned().unwrap_or_default(),
        x_end_display: labels.last().cloned().unwrap_or_default(),
        show_area: true,
        x_labels: labels,
        y_unit: "",
    })
}

pub(super) fn unused(
    rows: &[SkillFactRow],
    index: &InventoryIndex,
    audience: &MarketplaceAudience,
) -> Vec<UnusedSkillView> {
    let used: std::collections::HashSet<&str> = rows.iter().map(|r| r.skill.as_str()).collect();
    let used = &used;
    let mut out: Vec<UnusedSkillView> = index
        .plugins_by_skill
        .iter()
        .flat_map(|(skill, plugins)| {
            plugins.iter().filter_map(move |p| {
                let key = format!("{}:{}", p.0, skill.replace('_', "-"));
                (!used.contains(key.as_str())).then(|| UnusedSkillView {
                    entitled: audience.users_reaching_skill(index, skill),
                    marketplace: index
                        .marketplaces_by_plugin
                        .get(&p.0)
                        .and_then(|m| m.first())
                        .map(|m| m.id.clone())
                        .unwrap_or_default(),
                    plugin: p.0.clone(),
                    skill: key,
                })
            })
        })
        .collect();
    out.sort_by(|a, b| {
        b.entitled
            .cmp(&a.entitled)
            .then_with(|| a.skill.cmp(&b.skill))
    });
    out
}
