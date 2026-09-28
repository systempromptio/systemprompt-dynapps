//! The time window audit, analytics, and trace queries are scoped by.
//!
//! A date parser rather than a repository — it reaches the database only to
//! count rows in a candidate window — so it lives outside `repositories`.
//!
//! Parsed from `?from=&to=&preset=` on audit pages. `count_requests_in_range`
//! exists so a page can cheaply test a candidate window before committing to it
//! and widen when the default returns nothing.
//!
//! A window the URL asked for but that could not be resolved falls back to the
//! default 24h and sets `rejected_bounds`, so the page can say the listing is
//! not for the window in the address bar. Custom ranges are capped at
//! [`MAX_CUSTOM_WINDOW_DAYS`] (anchored on `to`): the trace-page percentiles
//! aggregate every row in the window, so an unbounded `?from=&to=` would be a
//! full-table scan anyone could trigger from a query string.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

/// Query parameters parsed from `?from=&to=&preset=` on audit pages.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TimeRangeQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub preset: Option<String>,
}

/// Resolved absolute time range used by every governance audit query.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct TimeRange {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
    pub preset: TimeRangePreset,
    pub rejected_bounds: bool,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TimeRangePreset {
    Min15,
    Hour1,
    Hours24,
    Days7,
    Days30,
    Days90,
    Custom,
}

const LEGACY_PRESET_ALIASES: &[(&str, TimeRangePreset)] = &[("1d", TimeRangePreset::Hours24)];

impl TimeRangePreset {
    #[must_use]
    pub const fn duration(self) -> Option<Duration> {
        match self {
            Self::Min15 => Some(Duration::minutes(15)),
            Self::Hour1 => Some(Duration::hours(1)),
            Self::Hours24 => Some(Duration::hours(24)),
            Self::Days7 => Some(Duration::days(7)),
            Self::Days30 => Some(Duration::days(30)),
            Self::Days90 => Some(Duration::days(90)),
            Self::Custom => None,
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Min15 => "15m",
            Self::Hour1 => "1h",
            Self::Hours24 => "24h",
            Self::Days7 => "7d",
            Self::Days30 => "30d",
            Self::Days90 => "90d",
            Self::Custom => "custom",
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Min15 => "15 minutes",
            Self::Hour1 => "hour",
            Self::Hours24 => "24 hours",
            Self::Days7 => "7 days",
            Self::Days30 => "30 days",
            Self::Days90 => "90 days",
            Self::Custom => "custom range",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "15m" => Some(Self::Min15),
            "1h" => Some(Self::Hour1),
            "24h" => Some(Self::Hours24),
            "7d" => Some(Self::Days7),
            "30d" => Some(Self::Days30),
            "90d" => Some(Self::Days90),
            "custom" => Some(Self::Custom),
            _ => LEGACY_PRESET_ALIASES
                .iter()
                .find(|(alias, _)| *alias == value)
                .map(|&(_, preset)| preset),
        }
    }
}

pub fn parse_time_range(query: &TimeRangeQuery) -> TimeRange {
    let now = Utc::now();

    if let Some(preset_str) = query.preset.as_deref()
        && let Some(preset) = TimeRangePreset::parse(preset_str)
        && let Some(d) = preset.duration()
    {
        return TimeRange {
            from: now - d,
            to: now,
            preset,
            rejected_bounds: false,
        };
    }

    let parsed_from = query.from.as_deref().and_then(parse_rfc3339);
    let parsed_to = query.to.as_deref().and_then(parse_rfc3339);
    if let (Some(from), Some(to)) = (parsed_from, parsed_to) {
        return clamp_custom(from, to);
    }

    TimeRange {
        from: now - Duration::hours(24),
        to: now,
        preset: TimeRangePreset::Hours24,
        rejected_bounds: asked_for_a_window(query),
    }
}

fn asked_for_a_window(query: &TimeRangeQuery) -> bool {
    let preset_rejected = query
        .preset
        .as_deref()
        .is_some_and(|p| TimeRangePreset::parse(p).is_none());
    let bound_supplied = query.from.is_some() || query.to.is_some();
    preset_rejected || bound_supplied
}

pub const MAX_CUSTOM_WINDOW_DAYS: i64 = 30;

fn clamp_custom(from: DateTime<Utc>, to: DateTime<Utc>) -> TimeRange {
    let (from, to) = if from <= to { (from, to) } else { (to, from) };
    let max = Duration::days(MAX_CUSTOM_WINDOW_DAYS);
    let from = if to - from > max { to - max } else { from };
    TimeRange {
        from,
        to,
        preset: TimeRangePreset::Custom,
        rejected_bounds: false,
    }
}

fn parse_rfc3339(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

pub fn preset_to_range(preset: TimeRangePreset) -> TimeRange {
    let now = Utc::now();
    let d = preset.duration().unwrap_or_else(|| Duration::hours(24));
    TimeRange {
        from: now - d,
        to: now,
        preset,
        rejected_bounds: false,
    }
}

pub async fn count_requests_in_range(pool: &PgPool, range: TimeRange) -> Result<i64, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT COUNT(*)::bigint AS "count!"
           FROM ai_requests
           WHERE created_at >= $1 AND created_at < $2"#,
        range.from,
        range.to,
    )
    .fetch_one(pool)
    .await?;
    Ok(row.count)
}
