//! The per-source chunking strategies ported from the Node project-context
//! chunker, and the metadata that makes a chunk citable.
//!
//! Each strategy's load-bearing property is pinned: Jira keeps body and
//! thread apart and regroups a long thread per comment; transcripts never cut
//! inside a speaker turn and carry the meeting header; the LLM path parses
//! the model's line numbers defensively and slices the ORIGINAL text; tables
//! repeat their header on every row chunk; and line ranges point into the
//! source even when two chunks open with the same words.

use crate::knowledge_fake_model::FakeModel;
use systemprompt_mcp_knowledge_bank::config::{ChunkStrategy, ChunkingConfig};
use systemprompt_mcp_knowledge_bank::ingest::chunker::{
    self, SplitRequest, find_line_range, infer_title, jira, llm, markdown, transcript,
};
use systemprompt_mcp_knowledge_bank::ingest::html::html_to_markdown;

fn config() -> ChunkingConfig {
    ChunkingConfig {
        target_chars: 400,
        max_chars: 800,
        min_chars: 60,
        split_tolerance: 50,
        enrich: false,
    }
}

fn jira_doc(comment_count: usize, comment_len: usize) -> String {
    let mut doc = String::from(
        "---\ntitle: \"[NAVA-123] Cart total wrong\"\nsource_type: jira\n---\n# [NAVA-123] Cart total wrong\n\n**Status:** Done | **Type:** Bug\n\n## Description\n\nTotals ignore tax rounding on multi-currency baskets. This is long enough to count as a body on its own for the split.\n\n## Comments\n",
    );
    for i in 0..comment_count {
        doc.push_str(&format!(
            "\n**Person {i}** (2026-01-0{}):\n{}\n",
            (i % 9) + 1,
            "x".repeat(comment_len)
        ));
    }
    doc
}

#[test]
fn a_short_jira_thread_becomes_body_plus_one_comment_chunk() {
    let chunks = jira::split(&jira_doc(2, 40), config());
    assert_eq!(chunks.len(), 2);
    assert!(chunks[0].text.contains("## Description"));
    assert!(!chunks[0].text.contains("Person 0"));
    assert!(
        chunks[1]
            .text
            .starts_with("[NAVA-123] Cart total wrong\n\n## Comments")
    );
    assert!(chunks[1].text.contains("Person 1"));
}

#[test]
fn a_long_jira_thread_is_regrouped_per_comment_with_the_header_repeated() {
    let chunks = jira::split(&jira_doc(12, 900), config());
    assert!(
        chunks.len() > 2,
        "12 comments of 900 chars exceed the 5000 threshold"
    );
    for chunk in &chunks[1..] {
        assert!(
            chunk
                .text
                .starts_with("[NAVA-123] Cart total wrong\n\n## Comments")
        );
        assert!(chunk.text.chars().count() <= jira::JIRA_MAX_GROUP + 60);
    }
}

#[test]
fn a_ticket_without_comments_stays_whole() {
    let doc = "# [NAVA-1] Tiny\n\n## Description\n\nJust a body.";
    let chunks = jira::split(doc, config());
    assert_eq!(chunks.len(), 1);
}

fn transcript_doc() -> String {
    let mut doc = String::from("---\ntitle: Cart sync\n---\nDiscovery call on 5 Jan 2026.\n\n");
    for i in 0..30 {
        let speaker = if i % 2 == 0 { "Alice" } else { "Bob" };
        doc.push_str(&format!(
            "{speaker}   00:{i:02}\nTurn {i}: {}\n",
            "we talked about the rounding mode and the regression test. ".repeat(3)
        ));
    }
    doc
}

#[test]
fn transcripts_are_recognised_and_split_at_speaker_boundaries() {
    let doc = transcript_doc();
    assert!(transcript::looks_like_transcript(&doc));
    let chunks = transcript::split_script(&doc, "Cart sync", config());
    assert!(chunks.len() > 1);
    for chunk in &chunks {
        assert!(
            chunk
                .text
                .starts_with("Meeting: Cart sync | Date: 5 Jan 2026")
        );
        let body = chunk
            .text
            .trim_start_matches("Meeting: Cart sync | Date: 5 Jan 2026")
            .trim();
        let first = body.lines().next().unwrap_or_default();
        assert!(
            transcript::is_speaker_header(first)
                || first.starts_with("---")
                || first.starts_with("title"),
            "a chunk must open on a speaker turn, got: {first:?}"
        );
    }
}

#[test]
fn prose_without_speaker_headers_is_not_a_transcript() {
    assert!(!transcript::looks_like_transcript(
        "# Doc\n\nJust paragraphs.\n\nMore text."
    ));
}

#[test]
fn llm_line_numbers_are_parsed_from_json_or_bare_integers() {
    assert_eq!(
        llm::parse_line_numbers(r#"{"starts": [1, 12, 34]}"#),
        vec![1, 12, 34]
    );
    assert_eq!(
        llm::parse_line_numbers("Sure! ```json\n{\"starts\":[3,9]}\n```"),
        vec![3, 9]
    );
    assert_eq!(
        llm::parse_line_numbers("start at lines 5 and 40"),
        vec![5, 40]
    );
}

#[test]
fn llm_starts_slice_the_original_text_without_rewriting_it() {
    let lines: Vec<&str> = (1..=10).map(|_| "line").collect();
    let chunks = llm::split_at_lines(&lines, &[4, 4, 99, 8]);
    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0].text.lines().count(), 3);
    assert_eq!(chunks[1].text.lines().count(), 4);
    assert_eq!(chunks[2].text.lines().count(), 3);
}

#[tokio::test]
async fn the_llm_strategy_uses_the_models_boundaries_and_falls_back_on_failure() {
    let doc = transcript_doc();
    let request = SplitRequest {
        body: &doc,
        title: "Cart sync",
        source_type: "meeting_notes",
        strategy: ChunkStrategy::Llm,
        config: config(),
    };
    let model = FakeModel::replying(&[r#"{"starts": [1, 30]}"#]);
    let guided = chunker::split_document(request, Some(&model)).await;
    assert!(!guided.is_empty());
    assert_eq!(*model.converse_calls.lock().expect("lock"), 1);

    let failing = FakeModel::replying(&[]);
    let fallback = chunker::split_document(request, Some(&failing)).await;
    assert!(
        !fallback.is_empty(),
        "a failing model must fall back to the script splitter"
    );
    assert!(fallback[0].text.starts_with("Meeting: Cart sync"));
}

#[tokio::test]
async fn the_none_strategy_keeps_a_non_jira_document_whole() {
    let request = SplitRequest {
        body: "Just one blob of text that should not be split at all.",
        title: "Blob",
        source_type: "adrs",
        strategy: ChunkStrategy::None,
        config: config(),
    };
    let chunks = chunker::split_document(request, None).await;
    assert_eq!(chunks.len(), 1);
}

#[test]
fn wide_tables_split_by_row_and_repeat_the_header() {
    let mut table = String::from("### Cards\n| Field | Type |\n| --- | --- |\n");
    for i in 0..60 {
        table.push_str(&format!("| field_{i} | string_type_number_{i} |\n"));
    }
    let pieces = markdown::split_table_rows(table.trim(), 300);
    assert!(pieces.len() > 1);
    for piece in &pieces {
        assert!(piece.starts_with("### Cards\n| Field | Type |\n| --- | --- |"));
        assert!(
            piece.lines().count() > 3,
            "every piece carries rows, not just the header"
        );
    }
}

#[test]
fn breadcrumbs_appear_only_below_the_top_level_and_keep_the_heading_line() {
    let body = "# Checkout\nIntro that is long enough to survive the merge step of the splitter, honestly.\n\n## Payment\nCards are tokenized and this sentence pads the section past the minimum size.\n\n### Cards\nThe card fields (pan, cvv) for the payment API, padded to survive the merge step.\n";
    let chunks = chunker::split_markdown(body, config());
    let top = chunks
        .iter()
        .find(|c| c.text.starts_with("# Checkout"))
        .expect("top section");
    assert_eq!(top.breadcrumb, "");
    let cards = chunks
        .iter()
        .find(|c| c.text.contains("### Cards"))
        .expect("cards section");
    assert_eq!(cards.breadcrumb, "Checkout > Payment > Cards");
}

#[test]
fn line_ranges_point_into_the_original_even_with_repeated_openings() {
    let original = "Intro\n\nNote: same start\nbody one\n\nNote: same start\nbody two\n";
    let (first, cursor) =
        find_line_range(original, "Note: same start\nbody one", 0).expect("first");
    assert_eq!(first, "3-4");
    let (second, _) =
        find_line_range(original, "Note: same start\nbody two", cursor).expect("second");
    assert_eq!(second, "6-7");
}

#[test]
fn titles_come_from_frontmatter_then_heading_then_fallback() {
    assert_eq!(
        infer_title("---\ntitle: \"From FM\"\n---\n# Heading", "file"),
        "From FM"
    );
    assert_eq!(infer_title("# Heading\nbody", "file"), "Heading");
    assert_eq!(infer_title("no heading at all", "file"), "file");
}

#[test]
fn confluence_storage_html_becomes_headed_markdown() {
    let md = html_to_markdown(
        "<h2>Payment</h2><p>Cards are <strong>tokenized</strong>.</p><script>x()</script>",
    );
    assert!(md.contains("## Payment"));
    assert!(md.contains("**tokenized**"));
    assert!(!md.contains("x()"));
}
