//! The chunk splitter's invariants.
//!
//! The ceiling is the one that matters operationally: Titan v2 rejects an
//! over-long input, so a chunk that escapes `max_chars + split_tolerance`
//! fails the whole ingestion run rather than degrading. The breadcrumb
//! matters for REQ-047's "traceable source references" — a fragment lifted
//! out of a long page has to still say which section it came from.

use systemprompt_mcp_knowledge_bank::config::ChunkingConfig;
use systemprompt_mcp_knowledge_bank::ingest::chunker::{Chunk, enforce_ceiling, split_markdown};

fn config() -> ChunkingConfig {
    ChunkingConfig {
        target_chars: 200,
        max_chars: 400,
        min_chars: 40,
        split_tolerance: 20,
        enrich: false,
    }
}

#[test]
fn headings_become_breadcrumbs_on_the_chunks_beneath_them() {
    let body = "# Runbook\n\nIntro paragraph that is long enough to stand on its own as a chunk of prose.\n\n## Rollback\n\nRun the rollback command and then verify the health endpoint responds.\n";
    let chunks = split_markdown(body, config());

    assert!(!chunks.is_empty());
    let rollback = chunks
        .iter()
        .find(|c| c.text.contains("rollback command"))
        .expect("the rollback section is chunked");
    assert_eq!(rollback.breadcrumb, "Runbook > Rollback");
}

#[test]
fn no_chunk_exceeds_the_ceiling() {
    let config = config();
    let ceiling = config.max_chars + config.split_tolerance;
    let body = "# Long\n\n".to_owned() + &"word ".repeat(2000);

    for chunk in split_markdown(&body, config) {
        assert!(
            chunk.text.chars().count() <= ceiling,
            "chunk of {} chars exceeds the {ceiling} ceiling",
            chunk.text.chars().count()
        );
    }
}

#[test]
fn the_ceiling_prefers_a_sentence_boundary() {
    let config = config();
    let sentence = "This is a complete sentence that ends properly. ";
    let chunk = Chunk {
        text: sentence.repeat(30),
        breadcrumb: "Doc".to_owned(),
    };

    let split = enforce_ceiling(&chunk, config);
    assert!(split.len() > 1);
    for piece in split.iter().take(split.len() - 1) {
        assert!(
            piece.text.ends_with('.'),
            "expected a sentence boundary, got: {:?}",
            piece.text.chars().rev().take(20).collect::<String>()
        );
    }
}

#[test]
fn splitting_a_long_chunk_keeps_its_breadcrumb_on_every_piece() {
    let config = config();
    let chunk = Chunk {
        text: "word ".repeat(500),
        breadcrumb: "Runbook > Rollback".to_owned(),
    };

    for piece in enforce_ceiling(&chunk, config) {
        assert_eq!(piece.breadcrumb, "Runbook > Rollback");
    }
}

#[test]
fn a_multibyte_body_never_splits_mid_character() {
    let config = config();
    let chunk = Chunk {
        text: "日本語のテキストです。".repeat(100),
        breadcrumb: String::new(),
    };

    let split = enforce_ceiling(&chunk, config);
    assert!(split.len() > 1);
    let rejoined: String = split.iter().map(|c| c.text.as_str()).collect();
    assert!(
        rejoined.chars().all(|c| c != '\u{fffd}'),
        "a split landed inside a multi-byte character"
    );
}

#[test]
fn undersized_chunks_are_merged_into_their_neighbour() {
    let body = "# A\n\nShort.\n\n# B\n\nAlso short.\n";
    let chunks = split_markdown(body, config());

    assert_eq!(
        chunks.len(),
        1,
        "two fragments below min_chars should merge rather than ship as noise"
    );
}
