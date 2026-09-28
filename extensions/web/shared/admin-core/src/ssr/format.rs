//! Display formatting shared by the admin entity pages.
//!
//! The pure value formatters live in `systemprompt_web_shared::format` (so
//! the test workspace can exercise them); this module re-exports them for
//! the ssr pages and keeps the admin-specific helpers.
//!
//! Ids, costs, token counts and durations are rendered in a handful of places
//! across the entity list and detail pages; the rules live here so a cost reads
//! the same on the sessions list as it does on the session it links to.

use serde::Serialize;

pub use systemprompt_web_shared::format::{format_cost, format_duration_ms, short_id, short_num};

pub fn format_token_total(total: i64) -> String {
    if total <= 0 {
        return "—".to_owned();
    }
    short_num(total)
}

pub fn local_time(t: chrono::DateTime<chrono::Utc>) -> String {
    t.with_timezone(&chrono::Local)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

pub fn format_span(
    start: Option<chrono::DateTime<chrono::Utc>>,
    end: Option<chrono::DateTime<chrono::Utc>>,
) -> String {
    match (start, end) {
        (Some(a), Some(b)) => format_duration_ms((b - a).num_milliseconds().max(0)),
        _ => "\u{2014}".to_owned(),
    }
}

pub fn relative_time(t: chrono::DateTime<chrono::Utc>) -> String {
    systemprompt_web_shared::format::relative_time(chrono::Utc::now().timestamp() - t.timestamp())
}

// Why: core's `ClientKind` owns `ai_requests.client_kind`; a value a newer
// core writes is shown as stored rather than mislabelled.
pub fn client_label(client_kind: &str) -> String {
    systemprompt::models::wire::origin::ClientKind::parse(client_kind)
        .map_or_else(|_| client_kind.to_owned(), |kind| kind.label().to_owned())
}

#[derive(Debug, Clone, Serialize)]
pub struct ClientChipView {
    pub label: String,
    pub tone: &'static str,
    pub title: String,
}

pub fn client_chip(client_kind: &str, attestation: &str) -> ClientChipView {
    use systemprompt::models::wire::origin::ClientAttestation;
    let tone = match ClientAttestation::parse(attestation) {
        Ok(ClientAttestation::HostToken) => "accent",
        Ok(ClientAttestation::Declared) => "info",
        Ok(_) | Err(_) => "muted",
    };
    ClientChipView {
        label: client_label(client_kind),
        tone,
        title: attestation_title(attestation),
    }
}

pub fn attestation_title(attestation: &str) -> String {
    systemprompt::models::wire::origin::ClientAttestation::parse(attestation).map_or_else(
        |_| format!("Attestation: {attestation}"),
        |tier| tier.label().to_owned(),
    )
}

pub fn key_slug(label: &str) -> String {
    label
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>()
        .join("-")
}
