//! The store-free half of the file layer: a `FileRef` names exactly one
//! source and stays declarable to every provider when a tool embeds it, the
//! inline limit refuses base64 over 64 KB and points at the upload tool, and
//! names and MIME types are checked before anything reaches the store.

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use rmcp::schemars::{self, JsonSchema};
use serde_json::Value;
use systemprompt::identifiers::{DocumentId, DocumentRevisionId, DocumentUploadId};
use systemprompt::wire::schema::gemini_declaration_violations;
use systemprompt_mcp_shared::files::{
    DEFAULT_INLINE_MAX_BYTES, FilePolicy, FileRef, FileSource, FilesError, StoredRef, check_name,
    download_name, normalize_mime, numbered_name, tagged_name,
};

use crate::files_support::UPLOAD_TOOL;

#[derive(serde::Serialize, JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
struct ToolWithFiles {
    attachments: Vec<FileRef>,
    file: FileRef,
}

#[test]
fn files_ref_with_no_source_or_two_sources_is_refused() {
    let empty = FileRef {
        name: "only-a-name.pdf".to_owned(),
        ..FileRef::default()
    };
    assert!(matches!(empty.source(), Err(FilesError::InvalidRef(_))));

    let two = FileRef {
        upload_id: DocumentUploadId::generate().as_str().to_owned(),
        content_base64: "aGk=".to_owned(),
        ..FileRef::default()
    };
    let error = two.source().expect_err("two sources");
    assert!(error.to_string().contains("exactly one"), "{error}");
}

#[test]
fn files_ref_parses_each_source_and_rejects_malformed_ids() {
    let document = DocumentId::generate();
    let revision = DocumentRevisionId::generate();
    let parsed = FileRef::document(&document, &revision)
        .source()
        .expect("document");
    assert_eq!(
        parsed,
        FileSource::Stored(StoredRef::Document {
            id: document,
            revision: Some(revision),
        })
    );

    let blob = FileRef {
        blob_ref: "AB".repeat(32),
        ..FileRef::default()
    };
    assert!(matches!(
        blob.source(),
        Ok(FileSource::Stored(StoredRef::Blob(_)))
    ));

    let orphan_revision = FileRef {
        revision_id: DocumentRevisionId::generate().as_str().to_owned(),
        content_base64: "aGk=".to_owned(),
        ..FileRef::default()
    };
    assert!(matches!(
        orphan_revision.source(),
        Err(FilesError::InvalidRef(_))
    ));

    for bad in [
        FileRef {
            upload_id: "../etc/passwd".to_owned(),
            ..FileRef::default()
        },
        FileRef {
            blob_ref: "not-a-hash".to_owned(),
            ..FileRef::default()
        },
    ] {
        assert!(matches!(bad.source(), Err(FilesError::InvalidRef(_))));
    }
}

#[test]
fn files_ref_schema_is_inlined_and_gemini_declarable() {
    let input = ToolWithFiles {
        attachments: vec![FileRef::default()],
        file: FileRef::default(),
    };
    assert_eq!(
        serde_json::to_value(&input).expect("serializes")["file"],
        serde_json::json!({}),
        "unset fields are omitted on output"
    );
    let schema = serde_json::to_value(schemars::schema_for!(ToolWithFiles)).expect("schema");
    assert!(schema.get("$defs").is_none(), "{schema:#}");
    assert!(!schema.to_string().contains("$ref"), "{schema:#}");
    let violations = gemini_declaration_violations(&schema);
    assert!(violations.is_empty(), "{violations:?}\n{schema:#}");
    let fields = &schema["properties"]["file"]["properties"];
    for field in ["upload_id", "blob_ref", "document_id", "content_base64"] {
        assert_eq!(fields[field]["type"], Value::from("string"), "{field}");
    }
    assert!(schema["properties"]["file"].get("required").is_none());
}

#[test]
fn files_inline_base64_up_to_the_limit_decodes() {
    let policy = FilePolicy::new(UPLOAD_TOOL);
    let body = vec![7_u8; usize::try_from(DEFAULT_INLINE_MAX_BYTES).expect("limit")];
    let encoded = STANDARD.encode(&body);
    let wrapped: String = encoded
        .as_bytes()
        .chunks(76)
        .map(|line| format!("{}\n", String::from_utf8_lossy(line)))
        .collect();
    assert_eq!(policy.decode_inline(&wrapped).expect("at the limit"), body);
}

#[test]
fn files_inline_base64_over_the_limit_names_the_upload_tool() {
    let policy = FilePolicy::new(UPLOAD_TOOL);
    let one_over = vec![1_u8; usize::try_from(DEFAULT_INLINE_MAX_BYTES + 1).expect("limit")];
    let error = policy
        .decode_inline(&STANDARD.encode(&one_over))
        .expect_err("over the limit");
    assert!(
        matches!(error, FilesError::InlineTooLarge { .. }),
        "{error}"
    );
    assert!(error.to_string().contains(UPLOAD_TOOL), "{error}");

    let far_over = "A".repeat(4 * 1024 * 1024);
    assert!(matches!(
        policy.decode_inline(&far_over),
        Err(FilesError::InlineTooLarge { .. })
    ));
    assert!(matches!(
        policy.decode_inline("not base64 at all!"),
        Err(FilesError::InvalidBase64)
    ));
}

#[test]
fn files_mime_types_are_normalized_checked_and_filtered() {
    assert_eq!(
        normalize_mime(" Application/PDF; charset=binary").as_deref(),
        Some("application/pdf")
    );
    for bad in ["pdf", "", "text/", "/plain", "image/png extra"] {
        assert_eq!(normalize_mime(bad), None, "{bad}");
    }

    let mut policy = FilePolicy::new(UPLOAD_TOOL);
    assert!(matches!(
        policy.mime("pdf"),
        Err(FilesError::InvalidMime { .. })
    ));
    assert_eq!(
        policy.download_mime("garbage").expect("falls back"),
        "application/octet-stream"
    );

    policy.allowed_mime = vec!["image/*".to_owned(), "application/pdf".to_owned()];
    assert_eq!(policy.mime("image/PNG").expect("wildcard"), "image/png");
    assert_eq!(
        policy.mime("application/pdf").expect("exact"),
        "application/pdf"
    );
    assert!(matches!(
        policy.mime("application/zip"),
        Err(FilesError::MimeNotAllowed { .. })
    ));
    assert!(matches!(
        policy.download_mime(""),
        Err(FilesError::MimeNotAllowed { .. })
    ));
}

#[test]
fn files_names_are_leaves_and_collisions_are_numbered() {
    assert_eq!(check_name("  report.pdf ").expect("plain"), "report.pdf");
    for bad in ["", "   ", "../x.pdf", "a\\b", "line\nbreak", ".."] {
        assert!(check_name(bad).is_err(), "{bad:?}");
    }
    assert!(check_name(&"n".repeat(256)).is_err());

    assert_eq!(download_name("/tmp/inbox/report.pdf"), "report.pdf");
    assert_eq!(download_name("C:\\Users\\x\\a.docx"), "a.docx");
    assert_eq!(download_name("dir/"), "download");
    assert_eq!(download_name("bell\u{7}.txt"), "bell.txt");

    assert_eq!(numbered_name("report.pdf", 2), "report (2).pdf");
    assert_eq!(numbered_name("README", 3), "README (3)");
    assert_eq!(numbered_name(".env", 2), ".env (2)");
    let longest = format!("{}.pdf", "n".repeat(251));
    let renamed = numbered_name(&longest, 2);
    assert!(renamed.len() <= 255, "{}", renamed.len());
    assert!(renamed.ends_with(" (2).pdf"));
    assert!(check_name(&tagged_name(&longest, "0a1b2c3d")).is_ok());
}

#[test]
fn files_debug_output_never_carries_inline_content() {
    let secret = "c2VjcmV0IGN1c3RvbWVyIGRhdGE=";
    let inline = FileRef {
        content_base64: secret.to_owned(),
        ..FileRef::default()
    };
    assert!(!format!("{inline:?}").contains(secret));
    let source = inline.source().expect("inline source");
    assert!(!format!("{source:?}").contains(secret));
}
