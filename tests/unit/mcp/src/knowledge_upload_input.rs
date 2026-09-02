//! Validation of `add_knowledge_document` arguments (FR-ING-6): a bare file
//! name with a supported suffix, exactly one content form, a decoded-size cap,
//! and the error shape the caller gets back.

use systemprompt::traits::ExtensionError;
use systemprompt_mcp_knowledge_bank::UploadRequest;
use systemprompt_mcp_knowledge_bank::error::KnowledgeBankError;
use systemprompt_mcp_knowledge_bank::ingest::MAX_UPLOAD_BYTES;
use systemprompt_mcp_knowledge_bank::tools::{
    AddKnowledgeDocumentInput, add_document_input_schema,
};

fn input(
    file_name: &str,
    content: Option<&str>,
    base64: Option<&str>,
) -> AddKnowledgeDocumentInput {
    AddKnowledgeDocumentInput {
        source_type: "meeting_notes".to_owned(),
        file_name: file_name.to_owned(),
        content: content.map(str::to_owned),
        content_base64: base64.map(str::to_owned),
        title: None,
        overwrite: false,
    }
}

fn rejection(input: AddKnowledgeDocumentInput) -> String {
    match UploadRequest::try_from(input) {
        Err(KnowledgeBankError::InvalidInput(message)) => message,
        Err(other) => panic!("expected InvalidInput, got {other}"),
        Ok(_) => panic!("expected the input to be rejected"),
    }
}

#[test]
fn text_content_becomes_the_bytes_and_the_title_is_trimmed_to_none() {
    let mut raw = input("2026-09-02_sync.md", Some("# Sync\n\nnotes"), None);
    raw.title = Some("   ".to_owned());
    let request = UploadRequest::try_from(raw).expect("valid");
    assert_eq!(request.bytes, b"# Sync\n\nnotes");
    assert_eq!(request.title, None);
    assert_eq!(request.source_type, "meeting_notes");
}

#[test]
fn binary_content_arrives_as_base64() {
    let request = UploadRequest::try_from(input("deck.pdf", None, Some("JVBERi0xLjQ=")))
        .expect("valid base64");
    assert_eq!(request.bytes, b"%PDF-1.4");
    assert!(rejection(input("deck.pdf", None, Some("not base64!"))).contains("content_base64"));
}

#[test]
fn exactly_one_content_form_is_required() {
    assert!(rejection(input("a.md", None, None)).contains("content"));
    assert!(rejection(input("a.md", Some("x"), Some("eA=="))).contains("not both"));
    assert!(rejection(input("a.md", Some(""), None)).contains("empty"));
}

#[test]
fn the_file_name_is_a_bare_name_with_a_supported_suffix() {
    for bad in [
        "../escape.md",
        "dir/inner.md",
        ".hidden.md",
        "no-suffix",
        "image.png",
        "",
    ] {
        let message = rejection(input(bad, Some("x"), None));
        assert!(message.contains("file_name"), "{bad}: {message}");
    }
    assert!(UploadRequest::try_from(input("ok_name-1.DOCX", Some("x"), None)).is_ok());
}

#[test]
fn the_decoded_size_is_capped_and_invalid_input_is_a_bad_request() {
    let big = "x".repeat(MAX_UPLOAD_BYTES + 1);
    let message = rejection(input("big.txt", Some(&big), None));
    assert!(message.contains("limit"), "{message}");
    let err = KnowledgeBankError::InvalidInput("x".to_owned());
    assert_eq!(err.code(), "INVALID_INPUT");
    assert_eq!(err.status(), axum::http::StatusCode::BAD_REQUEST);
    assert!(!err.is_retryable());
}

#[test]
fn the_tool_schema_requires_source_type_and_file_name_only() {
    let schema = add_document_input_schema();
    let required: Vec<&str> = schema["required"]
        .as_array()
        .expect("required list")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert_eq!(required, ["source_type", "file_name"]);
    assert!(schema["properties"]["content_base64"].is_object());
    assert!(schema["properties"]["overwrite"].is_object());
}
