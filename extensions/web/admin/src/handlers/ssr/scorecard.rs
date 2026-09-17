//! The judge's four criterion scores as one component, used wherever a
//! conversation, skill, marketplace or version is listed. A score reads on
//! the same three tones everywhere; an unscored criterion is a muted dash,
//! never an empty cell, so "not judged yet" is visibly different from a low
//! score.

use chrono::{DateTime, Duration, Utc};
use serde::Serialize;

use super::analysis::evaluation::views::score_tone;
use super::types::{LineChartSpec, SvgLineChartView, SvgSeriesInput, line_chart};
use crate::repositories::analysis::retained::RetainedWindow;

pub(crate) const CRITERIA: [(&str, &str, &str); 4] = [
    ("intent", "I", "Intent"),
    ("quality", "Q", "Quality"),
    ("speed", "S", "Speed"),
    ("cost", "C", "Cost"),
];

#[derive(Debug, Clone, Serialize)]
pub struct ScorePillView {
    pub key: &'static str,
    pub letter: &'static str,
    pub label: &'static str,
    pub value: String,
    pub tone: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta_tone: Option<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScorecardView {
    pub pills: [ScorePillView; 4],
    pub scored: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
    pub title: String,
}

// Why: the four means (or single scores) one scorecard renders.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Scores {
    pub intent: Option<f64>,
    pub quality: Option<f64>,
    pub speed: Option<f64>,
    pub cost: Option<f64>,
}

impl Scores {
    pub(crate) const fn from_points(
        intent: Option<i16>,
        quality: Option<i16>,
        speed: Option<i16>,
        cost: Option<i16>,
    ) -> Self {
        const fn f(v: Option<i16>) -> Option<f64> {
            match v {
                Some(v) => Some(v as f64),
                None => None,
            }
        }
        Self {
            intent: f(intent),
            quality: f(quality),
            speed: f(speed),
            cost: f(cost),
        }
    }

    pub(crate) const fn any(self) -> bool {
        self.intent.is_some()
            || self.quality.is_some()
            || self.speed.is_some()
            || self.cost.is_some()
    }
}

fn pill(key: usize, value: Option<f64>, prior: Option<f64>) -> ScorePillView {
    let (k, letter, label) = CRITERIA[key];
    let (delta, delta_tone) = match (value, prior) {
        (Some(v), Some(p)) if (v - p).abs() >= 0.5 => {
            let d = v - p;
            (
                Some(format!("{}{:.0}", if d > 0.0 { "+" } else { "" }, d)),
                Some(if d > 0.0 { "ok" } else { "err" }),
            )
        },
        _ => (None, None),
    };
    ScorePillView {
        key: k,
        letter,
        label,
        value: value.map_or_else(|| "—".to_owned(), |v| format!("{v:.0}")),
        tone: value.map_or("muted", |v| score_tone(v.round() as i16)),
        delta,
        delta_tone,
    }
}

impl Default for ScorecardView {
    fn default() -> Self {
        Self::new(Scores::default(), None, "Not judged yet")
    }
}

impl ScorecardView {
    pub(crate) fn new(scores: Scores, href: Option<String>, title: impl Into<String>) -> Self {
        Self::with_prior(scores, Scores::default(), href, title)
    }

    pub(crate) fn with_prior(
        scores: Scores,
        prior: Scores,
        href: Option<String>,
        title: impl Into<String>,
    ) -> Self {
        Self {
            pills: [
                pill(0, scores.intent, prior.intent),
                pill(1, scores.quality, prior.quality),
                pill(2, scores.speed, prior.speed),
                pill(3, scores.cost, prior.cost),
            ],
            scored: scores.any(),
            href,
            title: title.into(),
        }
    }
}

// Why: days with no judged session carry the previous day's mean forward so
// the line reads as a level, not a collapse to zero; the subtitle says so.
pub(crate) type ScorePoint = (DateTime<Utc>, Option<f64>);

fn daily_series(
    label: &str,
    points: &[ScorePoint],
    start: DateTime<Utc>,
    days: usize,
) -> SvgSeriesInput {
    let mut values = Vec::with_capacity(days);
    let mut last: i64 = 0;
    let mut latest = None;
    for i in 0..days {
        let day = start + Duration::days(i64::try_from(i).unwrap_or(0));
        if let Some((_, Some(v))) = points
            .iter()
            .find(|(d, v)| v.is_some() && d.date_naive() == day.date_naive())
        {
            last = v.round() as i64;
            latest = Some(last);
        }
        values.push(last);
    }
    SvgSeriesInput {
        label: label.to_owned(),
        values,
        value_display: latest.map_or_else(|| "—".to_owned(), |v| v.to_string()),
    }
}

pub(crate) fn score_trend_chart(
    title: &'static str,
    subtitle: String,
    window: RetainedWindow,
    series: &[(&str, Vec<ScorePoint>)],
) -> SvgLineChartView {
    let days = usize::try_from((window.end - window.start).num_days().max(1)).unwrap_or(1);
    let inputs = series
        .iter()
        .map(|(label, points)| daily_series(label, points, window.start, days))
        .collect();
    let mid = window.start + (window.end - window.start) / 2;
    line_chart(LineChartSpec {
        title,
        subtitle,
        empty_message: "No judged session in this window yet.",
        series: inputs,
        ref_lines: vec![
            (80, "good ≥ 80".to_owned(), "ok"),
            (50, "watch < 50".to_owned(), "warn"),
        ],
        y_max: Some(100),
        y_display: |v| v.to_string(),
        x_start_display: window.start.format("%b %-d").to_string(),
        x_mid_display: mid.format("%b %-d").to_string(),
        x_end_display: window.end.format("%b %-d").to_string(),
        show_area: true,
    })
}
