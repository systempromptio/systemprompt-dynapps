//! Language codes and the text-search configuration each one indexes with.

pub fn is_language_code(code: &str) -> bool {
    let mut parts = code.splitn(2, ['_', '-']);
    let base = parts.next().unwrap_or_default();
    let base_ok = (2..=3).contains(&base.len()) && base.bytes().all(|b| b.is_ascii_lowercase());
    base_ok
        && parts.all(|rest| {
            (2..=8).contains(&rest.len()) && rest.bytes().all(|b| b.is_ascii_alphanumeric())
        })
}

pub fn fts_config(code: &str) -> &'static str {
    let base = code.split(['_', '-']).next().unwrap_or_default();
    match base.to_ascii_lowercase().as_str() {
        "es" => "spanish",
        "en" => "english",
        "nl" => "dutch",
        "fr" => "french",
        "de" => "german",
        "pt" => "portuguese",
        "it" => "italian",
        _ => "simple",
    }
}
