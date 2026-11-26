//! The chunker against what dynbrain's Python chunker produced.

use dynbrain::assets::markdown::index_text;
use dynbrain::ingest::chunker::{MAX_CHARS, chunk_document};

fn same_as_python(document: &str, expected: &str) {
    let want: Vec<[String; 2]> = serde_json::from_str(expected).expect("the fixture is JSON");
    let got = chunk_document(document);
    assert_eq!(got.len(), want.len(), "the number of chunks");
    for (index, (got, want)) in got.iter().zip(&want).enumerate() {
        assert_eq!(got.content, want[0], "the text of chunk {index}");
        assert_eq!(
            got.heading_path, want[1],
            "the heading path of chunk {index}"
        );
    }
}

#[test]
fn the_retrieval_pipeline_doc_is_cut_as_python_cuts_it() {
    same_as_python(
        include_str!("../fixtures/chunker/retrieval-pipeline.md"),
        include_str!("../fixtures/chunker/retrieval-pipeline.json"),
    );
}

#[test]
fn the_git_knowledge_readme_is_cut_as_python_cuts_it() {
    same_as_python(
        include_str!("../fixtures/chunker/knowledge-git.md"),
        include_str!("../fixtures/chunker/knowledge-git.json"),
    );
}

#[test]
fn the_ai_mcp_readme_is_cut_as_python_cuts_it() {
    same_as_python(
        include_str!("../fixtures/chunker/ai-mcp-knowledge.md"),
        include_str!("../fixtures/chunker/ai-mcp-knowledge.json"),
    );
}

#[test]
fn headings_fences_unicode_stops_and_overlong_words_are_cut_as_python_cuts_them() {
    same_as_python(
        include_str!("../fixtures/chunker/edge-cases.md"),
        include_str!("../fixtures/chunker/edge-cases.json"),
    );
}

#[test]
fn no_chunk_is_longer_than_the_cap_and_blank_text_has_none() {
    let long = include_str!("../fixtures/chunker/edge-cases.md");
    assert!(
        chunk_document(long)
            .iter()
            .all(|chunk| chunk.content.chars().count() <= MAX_CHARS)
    );
    assert!(chunk_document("").is_empty());
    assert!(chunk_document(" \n\t\n ").is_empty());
}

#[test]
fn a_document_without_headings_has_empty_paths_and_overlapping_chunks() {
    let sentences: Vec<String> = (0..70)
        .map(|n| format!("w{n}a w{n}b w{n}c w{n}d w{n}e w{n}f w{n}g w{n}h w{n}i end{n}."))
        .collect();
    let chunks = chunk_document(&sentences.join(" "));
    assert_eq!(chunks.len(), 3);
    assert!(chunks.iter().all(|chunk| chunk.heading_path.is_empty()));
    assert!(chunks[1].content.starts_with(&sentences[25]));
    assert!(chunks[0].content.ends_with(&sentences[29]));
}

#[test]
fn a_file_reference_becomes_its_alt_text_and_no_url_reaches_a_chunk() {
    let body = "# Design\n\nThe layout is in ![Architecture diagram](asset://3) and the rules in [the spec](asset://4). See asset://5 too.";
    let chunks = chunk_document(&index_text(body));
    assert_eq!(chunks.len(), 1);
    let text = &chunks[0].content;
    assert!(text.contains("Architecture diagram"), "{text}");
    assert!(text.contains("the spec"), "{text}");
    assert!(!text.contains("asset://"), "{text}");
    assert_eq!(chunks[0].heading_path, "Design");
}
