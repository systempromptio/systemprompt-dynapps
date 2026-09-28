//! Shared entry point for the activity constructors.

use systemprompt_web_shared::format::truncate_chars;

pub(super) fn truncate(s: &str, max: usize) -> String {
    let head = truncate_chars(s, max);
    if head.len() == s.len() {
        return s.to_owned();
    }
    format!("{}\u{2026}", back_to_last_word_boundary(head))
}

fn back_to_last_word_boundary(head: &str) -> &str {
    head.rfind(' ').map_or(head, |pos| &head[..pos])
}
