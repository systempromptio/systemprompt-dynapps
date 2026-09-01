//! HTML to Markdown for Confluence and Jira content.
//!
//! Confluence and Jira render their content as HTML; the index wants
//! Markdown so the heading-aware splitter and the line pointers have
//! structure to work with. Atlassian macros and layout scaffolding are
//! dropped before conversion — they carry no prose.

use htmd::HtmlToMarkdown;

#[must_use]
pub fn html_to_markdown(html: &str) -> String {
    let converter = HtmlToMarkdown::builder()
        .skip_tags(vec!["script", "style", "iframe", "svg", "ac:parameter"])
        .build();
    match converter.convert(html) {
        Ok(markdown) => tidy(&markdown),
        Err(e) => {
            tracing::warn!(error = %e, "html to markdown failed; indexing the stripped text");
            tidy(&strip_tags(html))
        },
    }
}

fn tidy(markdown: &str) -> String {
    let mut out = String::with_capacity(markdown.len());
    let mut blank_run = 0usize;
    for line in markdown.lines() {
        let trimmed = line.trim_end();
        if trimmed.trim().is_empty() {
            blank_run += 1;
            if blank_run > 1 {
                continue;
            }
        } else {
            blank_run = 0;
        }
        out.push_str(trimmed);
        out.push('\n');
    }
    out.trim().to_owned()
}

#[must_use]
pub fn strip_tags(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                out.push(' ');
            },
            _ if !in_tag => out.push(c),
            _ => {},
        }
    }
    out
}
