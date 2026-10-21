//! How a quota window reads in the console: a fixed span in seconds, or a UTC
//! calendar period (`calendar: day | week | month`), which core resets on the
//! calendar boundary itself.

use chrono::{Datelike, Months, NaiveDate};
use systemprompt::gateway::{CalendarPeriod, QuotaWindow};

#[must_use]
pub const fn is_calendar_month(window: &QuotaWindow) -> bool {
    matches!(window.calendar, Some(CalendarPeriod::Month))
}

#[must_use]
pub fn days_left_in_month(today: NaiveDate) -> u32 {
    let first = today.with_day(1).unwrap_or(today);
    let next = first + Months::new(1);
    u32::try_from((next - today).num_days()).unwrap_or(1)
}

#[must_use]
pub fn window_label(window: &QuotaWindow, today: NaiveDate) -> String {
    match window.calendar {
        Some(CalendarPeriod::Month) => {
            let left = days_left_in_month(today);
            let noun = if left == 1 { "day" } else { "days" };
            format!("calendar month ({left} {noun} left)")
        },
        Some(CalendarPeriod::Week) => "calendar week".to_owned(),
        Some(CalendarPeriod::Day) => "calendar day".to_owned(),
        None => match window.window_seconds {
            60 => "minute".to_owned(),
            3_600 => "hour".to_owned(),
            86_400 => "day".to_owned(),
            604_800 => "week".to_owned(),
            s if s % 86_400 == 0 => format!("{} days", s / 86_400),
            s if s % 3_600 == 0 => format!("{} hours", s / 3_600),
            s => format!("{s} s"),
        },
    }
}
