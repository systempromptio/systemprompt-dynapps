//! The answer's context: documents whole when they fit, their best chunks
//! whole and in reading order when they do not, and the numbered blocks.

use dynbrain::ids::ChunkId;
use dynbrain::search::context::{
    BLOCK_SEPARATOR, ContextChunk, ContextSource, GAP_MARK, QUOTA_CHARS, TRUNCATED_MARK, blocks,
    document_text, select_chunks,
};

fn chunk(id: i64, sequence: i32, content: &str) -> ContextChunk {
    ContextChunk {
        id: ChunkId(id),
        sequence,
        content: content.to_owned(),
    }
}

#[test]
fn every_document_is_granted_25000_characters() {
    assert_eq!(QUOTA_CHARS, 25_000);
}

#[test]
fn a_document_that_fits_its_share_is_given_whole() {
    let ranked = [chunk(1, 0, "ignored")];
    assert_eq!(
        document_text("# Title\n\nAll of it.", 100, &ranked),
        "# Title\n\nAll of it."
    );
}

#[test]
fn a_long_document_gives_its_best_chunks_whole_in_reading_order() {
    let ranked = [
        chunk(13, 3, "dddd dddd"),
        chunk(11, 1, "bbbb bbbb"),
        chunk(14, 2, "eeee eeee eeee eeee eeee"),
        chunk(10, 0, "aaaa aaaa"),
    ];
    let markdown = "x".repeat(200);
    let text = document_text(&markdown, 30, &ranked);
    assert_eq!(
        text,
        format!("aaaa aaaa\nbbbb bbbb\n{GAP_MARK}\ndddd dddd"),
        "the most relevant chunks that fit, put back in order, a gap where one was skipped"
    );
    assert!(
        !text.contains("eeee"),
        "a chunk that does not fit whole is skipped, never cut"
    );
}

#[test]
fn only_the_best_chunk_is_ever_cut_and_the_cut_is_marked() {
    let ranked = [chunk(1, 0, &"y".repeat(50)), chunk(2, 1, "small")];
    let text = select_chunks(&ranked, 20);
    assert_eq!(text, format!("{}{TRUNCATED_MARK}", "y".repeat(20)));
}

#[test]
fn a_long_document_without_chunks_is_cut_to_its_head_and_marked() {
    let markdown = "z".repeat(40);
    assert_eq!(
        document_text(&markdown, 10, &[]),
        format!("{}{TRUNCATED_MARK}", "z".repeat(10))
    );
    assert_eq!(document_text("", 10, &[]), "");
}

#[test]
fn blocks_are_numbered_by_the_position_of_their_source() {
    let source = |title: &str, lang: &str, text: &str| ContextSource {
        title: title.to_owned(),
        path: format!("handbook/{}", title.to_lowercase()),
        lang: lang.to_owned(),
        text: text.to_owned(),
    };
    let context = blocks(&[
        source("Backups", "es", "Copias diarias."),
        source("Empty", "en", ""),
        source("Restore", "", "Restore steps."),
    ]);
    assert_eq!(
        context,
        format!(
            "[1] Backups (handbook/backups, lang=es)\nCopias diarias.{BLOCK_SEPARATOR}[3] Restore (handbook/restore, lang=?)\nRestore steps."
        ),
        "an empty source keeps its number and leaves no block; an unknown language is `?`"
    );
    assert_eq!(blocks(&[]), "");
}
