//! Checks on every value a tool puts into a Graph path or `OData` expression,
//! ported from the Belgian server's `sanitize.py` with the same rules.
//!
//! Ids and folder names are validated rather than escaped, so a value that
//! passes can sit in a path segment verbatim: an id is base64, a GUID, a
//! Teams chat id (`19:x_y@unq.gbl.spaces`) or a `SharePoint` site id
//! (`host,guid,guid`), never a separator or `..`; a folder is a well-known
//! name in any case or an id. `OData` string literals have their quotes
//! doubled (`$search` also escapes `"`). Datetimes must be ISO 8601 with a
//! zone and stay unquoted, because quoting makes Graph compare strings (400
//! "incompatible types"). `$select` lists take property names and type-cast
//! segments only. `$top` clamps to `1..=max`, `$skip` to non-negative.

use super::GraphError;

const WELL_KNOWN_FOLDERS: [&str; 17] = [
    "inbox",
    "drafts",
    "sentitems",
    "deleteditems",
    "junkemail",
    "archive",
    "clutter",
    "conflicts",
    "conversationhistory",
    "localfailures",
    "msgfolderroot",
    "outbox",
    "recoverableitemsdeletions",
    "scheduled",
    "searchfolders",
    "serverfailures",
    "syncissues",
];

const MAX_HOSTNAME_LEN: usize = 253;
pub const DEFAULT_MAX_TOP: i64 = 50;

const fn invalid(message: String) -> GraphError {
    GraphError::Invalid(message)
}

pub fn validate_id<'a>(value: &'a str, name: &str) -> Result<&'a str, GraphError> {
    if value.is_empty() || value.contains("..") || value.contains(['/', '\\']) {
        return Err(invalid(format!(
            "Invalid {name}: must not contain path separators"
        )));
    }
    let allowed = |c: char| c.is_ascii_alphanumeric() || "-._=+@:!,".contains(c);
    if !value.chars().all(allowed) {
        return Err(invalid(format!(
            "Invalid {name}: contains disallowed characters"
        )));
    }
    Ok(value)
}

pub fn validate_folder(value: &str) -> Result<&str, GraphError> {
    if WELL_KNOWN_FOLDERS.contains(&value.to_ascii_lowercase().as_str()) {
        return Ok(value);
    }
    validate_id(value, "folder")
}

pub fn validate_path(value: &str) -> Result<&str, GraphError> {
    if value.contains("..") {
        return Err(invalid("Path must not contain '..'".to_owned()));
    }
    Ok(value.trim_matches('/'))
}

pub fn validate_hostname(value: &str) -> Result<&str, GraphError> {
    if value.is_empty()
        || value.len() > MAX_HOSTNAME_LEN
        || value.contains("..")
        || value.contains('/')
    {
        return Err(invalid("Invalid hostname".to_owned()));
    }
    let label_ok = |label: &str| {
        !label.is_empty()
            && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
            && !label.starts_with('-')
            && !label.ends_with('-')
    };
    if !value.split('.').all(label_ok) {
        return Err(invalid(
            "Invalid hostname: must be a valid DNS name".to_owned(),
        ));
    }
    Ok(value)
}

pub fn sanitize_search(value: &str) -> String {
    value.replace('\'', "''").replace('"', "\\\"")
}

pub fn sanitize_odata_string(value: &str) -> String {
    value.replace('\'', "''")
}

pub fn sanitize_odata_datetime(value: &str) -> Result<&str, GraphError> {
    if is_iso_datetime(value) {
        Ok(value)
    } else {
        Err(invalid(format!(
            "Invalid datetime: must be ISO 8601 with timezone (e.g. '2024-01-01T00:00:00Z'), got '{value}'"
        )))
    }
}

fn digits(s: &str, n: usize) -> bool {
    s.len() == n && s.bytes().all(|b| b.is_ascii_digit())
}

fn is_iso_datetime(value: &str) -> bool {
    let Some((date, time)) = value.split_once('T') else {
        return false;
    };
    let date_ok = matches!(date.split('-').collect::<Vec<_>>().as_slice(),
        [y, m, d] if digits(y, 4) && digits(m, 2) && digits(d, 2));
    let (clock, zone) = if let Some(clock) = time.strip_suffix('Z') {
        (clock, "Z")
    } else if time.len() > 6 && time.is_char_boundary(time.len() - 6) {
        time.split_at(time.len() - 6)
    } else {
        return false;
    };
    let zone_ok = zone == "Z"
        || matches!(zone.as_bytes(), [b'+' | b'-', h1, h2, b':', m1, m2]
            if [h1, h2, m1, m2].iter().all(|b| b.is_ascii_digit()));
    let (hms, fraction) = clock.split_once('.').unwrap_or((clock, "0"));
    let clock_ok = matches!(hms.split(':').collect::<Vec<_>>().as_slice(),
        [h, m, s] if digits(h, 2) && digits(m, 2) && digits(s, 2))
        && !fraction.is_empty()
        && fraction.bytes().all(|b| b.is_ascii_digit());
    date_ok && zone_ok && clock_ok
}

pub fn sanitize_select(value: &str, name: &str) -> Result<String, GraphError> {
    let parts: Vec<&str> = value
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect();
    if parts.is_empty() {
        return Err(invalid(format!(
            "Invalid {name}: must name at least one field"
        )));
    }
    let segment_ok = |segment: &str| {
        segment.starts_with(|c: char| c.is_ascii_alphabetic())
            && segment
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '.')
    };
    if let Some(bad) = parts.iter().find(|part| !part.split('/').all(segment_ok)) {
        return Err(invalid(format!(
            "Invalid {name}: '{bad}' is not a field name"
        )));
    }
    Ok(parts.join(","))
}

pub fn clamp_top(value: i64, maximum: i64) -> u32 {
    value.clamp(1, maximum.max(1)) as u32
}

pub fn clamp_skip(value: i64) -> u32 {
    value.clamp(0, i64::from(u32::MAX)) as u32
}
