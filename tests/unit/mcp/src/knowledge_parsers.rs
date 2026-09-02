//! The file parsers behind the files categories (FR-ING-5): Markdown and
//! text pass through, DOCX and PDF are reduced to text, an unsupported
//! suffix is an error rather than a panic, and the transcript de-wrapper
//! changes whitespace only so line pointers into its output stay honest.

use std::io::Write;
use systemprompt_mcp_knowledge_bank::ingest::parsers::{
    SUPPORTED_SUFFIXES, dewrap_transcript, docx_xml_to_text, is_supported, parse_named,
};

fn docx(paragraphs: &[&str]) -> Vec<u8> {
    let body: String = paragraphs
        .iter()
        .map(|p| format!("<w:p><w:r><w:t>{p}</w:t></w:r></w:p>"))
        .collect();
    let xml = format!(
        "<?xml version=\"1.0\"?><w:document xmlns:w=\"w\"><w:body>{body}</w:body></w:document>"
    );
    let mut cursor = std::io::Cursor::new(Vec::new());
    {
        let mut archive = zip::ZipWriter::new(&mut cursor);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        archive
            .start_file("word/document.xml", options)
            .expect("zip entry");
        archive.write_all(xml.as_bytes()).expect("zip body");
        archive.finish().expect("zip finish");
    }
    cursor.into_inner()
}

#[test]
fn markdown_and_text_pass_through_unchanged() {
    assert_eq!(
        parse_named("note.md", b"# Title\n\nbody").expect("md"),
        "# Title\n\nbody"
    );
    assert_eq!(parse_named("NOTE.TXT", b"plain").expect("txt"), "plain");
    assert_eq!(SUPPORTED_SUFFIXES, ["md", "txt", "pdf", "docx"]);
    assert!(is_supported(std::path::Path::new("a/b/c.Docx")));
    assert!(!is_supported(std::path::Path::new("a/b/c.png")));
}

#[test]
fn a_docx_body_becomes_paragraph_separated_text() {
    let bytes = docx(&[
        "Decision: use banker&apos;s rounding",
        "Owner: payments &amp; checkout",
    ]);
    let text = parse_named("adr.docx", &bytes).expect("docx parses");
    assert_eq!(
        text.trim(),
        "Decision: use banker's rounding\nOwner: payments & checkout"
    );
    assert_eq!(docx_xml_to_text("<w:p>a<w:tab/>b</w:p>"), "a\tb\n");
}

#[test]
fn an_unsupported_suffix_or_broken_binary_is_an_error_not_a_panic() {
    let err = parse_named("image.png", b"\x89PNG").expect_err("png is refused");
    assert!(
        err.to_string().contains("unsupported file type .png"),
        "{err}"
    );
    assert!(parse_named("bad.docx", b"not a zip").is_err());
    assert!(parse_named("bad.pdf", b"not a pdf").is_err());
}

#[test]
fn dewrap_joins_wrapped_sentences_but_keeps_speaker_headers() {
    let wrapped = "Alice Smith:\nWe agreed to ship the\ncart fix on Monday. QA\nstarts Tuesday.\n\nBob Jones:\nFine by me.";
    let text = dewrap_transcript(wrapped);
    assert_eq!(
        text,
        "Alice Smith:\nWe agreed to ship the cart fix on Monday.\nQA starts Tuesday.\nBob Jones:\nFine by me."
    );
    fn words(s: &str) -> Vec<&str> {
        s.split_whitespace().collect()
    }
    assert_eq!(words(wrapped), words(&text), "wording is never changed");
}
