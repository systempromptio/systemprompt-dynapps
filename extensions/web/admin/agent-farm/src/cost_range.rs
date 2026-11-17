//! The Cost page's period: the query, its parsing and the instants farm-api is
//! asked for.

use chrono::{Duration, NaiveDate};
use serde::Deserialize;

pub const DEFAULT_DAYS: i64 = 7;

#[derive(Debug, Default, Deserialize)]
pub struct CostQuery {
    #[serde(default)]
    tenant: Option<String>,
    #[serde(default)]
    from: Option<String>,
    #[serde(default)]
    to: Option<String>,
}

pub fn clean(value: Option<&String>) -> Option<&str> {
    value.map(|v| v.trim()).filter(|v| !v.is_empty())
}

/// The inclusive day range shown, plus what was wrong with the request.
#[derive(Debug, PartialEq, Eq)]
pub struct DayRange {
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub notes: Vec<String>,
}

impl DayRange {
    pub fn instants(&self) -> (String, String) {
        let until = self.to + Duration::days(1);
        (
            format!("{}T00:00:00Z", self.from),
            format!("{until}T00:00:00Z"),
        )
    }
}

pub fn parse_day(label: &str, raw: Option<&str>, notes: &mut Vec<String>) -> Option<NaiveDate> {
    let raw = raw?;
    NaiveDate::parse_from_str(raw, "%Y-%m-%d").map_or_else(
        |_| {
            notes.push(format!(
                "\"{raw}\" is not a date (use YYYY-MM-DD): the default for \"{label}\" was used."
            ));
            None
        },
        Some,
    )
}

pub fn resolve_range(from: Option<&str>, to: Option<&str>, today: NaiveDate) -> DayRange {
    let mut notes = Vec::new();
    let to = parse_day("to", to, &mut notes).unwrap_or(today);
    let from = parse_day("from", from, &mut notes).unwrap_or(to - Duration::days(DEFAULT_DAYS - 1));
    if from > to {
        notes.push("\"from\" was after \"to\": the two were swapped.".to_owned());
        return DayRange {
            from: to,
            to: from,
            notes,
        };
    }
    DayRange { from, to, notes }
}
