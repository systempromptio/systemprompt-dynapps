//! Window bounds arrive as HTML date controls (`%Y-%m-%d`), datetime controls
//! (UTC without an offset) or RFC3339 from shared links; they render back as a
//! date whenever they sit on a UTC midnight so the date control round-trips.
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub fn parse(value: &str) -> Option<DateTime<Utc>> {
    if let Ok(parsed) = DateTime::parse_from_rfc3339(value) {
        return Some(parsed.with_timezone(&Utc));
    }
    if let Ok(day) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        return Some(day.and_time(NaiveTime::MIN).and_utc());
    }
    ["%Y-%m-%dT%H:%M:%S", "%Y-%m-%dT%H:%M"]
        .iter()
        .find_map(|format| NaiveDateTime::parse_from_str(value, format).ok())
        .map(|value| value.and_utc())
}

pub fn render(value: DateTime<Utc>) -> String {
    if value.time() == NaiveTime::MIN {
        value.format("%Y-%m-%d").to_string()
    } else {
        value.format("%Y-%m-%dT%H:%M:%S").to_string()
    }
}

/// One edge of the analysis window as the query carries it: absent, or a UTC
/// instant that serialises back as a bare date whenever it sits on midnight so
/// an HTML date control round-trips it unchanged.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct WindowBound(pub Option<DateTime<Utc>>);

impl From<Option<DateTime<Utc>>> for WindowBound {
    fn from(value: Option<DateTime<Utc>>) -> Self {
        Self(value)
    }
}

impl From<DateTime<Utc>> for WindowBound {
    fn from(value: DateTime<Utc>) -> Self {
        Self(Some(value))
    }
}

impl std::ops::Deref for WindowBound {
    type Target = Option<DateTime<Utc>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> Deserialize<'de> for WindowBound {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = Option::<String>::deserialize(d)?;
        raw.filter(|value| !value.is_empty())
            .map(|value| {
                parse(&value)
                    .ok_or_else(|| serde::de::Error::custom("Expected a UTC date or date and time"))
            })
            .transpose()
            .map(Self)
    }
}

impl Serialize for WindowBound {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.0.map(render).serialize(s)
    }
}
