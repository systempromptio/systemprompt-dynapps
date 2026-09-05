//! `add_knowledge_document` through the real ingestor (FR-ING-6): an
//! uploaded note is searchable immediately with no sync or rebuild; the
//! bytes land inside the category tree, so a later full rebuild and a prune
//! leave the document in place without re-embedding it; a binary arrives as
//! base64 and is parsed; a second upload of the same name needs `overwrite`
//! and replaces exactly one document; and an Atlassian category refuses
//! uploads.

use crate::knowledge_files::{Fixture, docx, filter, signed_in};
use base64::Engine;
use systemprompt_mcp_knowledge_bank::error::KnowledgeBankError;
use systemprompt_mcp_knowledge_bank::tools::AddKnowledgeDocumentInput;
use systemprompt_mcp_knowledge_bank::{
    FilesConnector, KnowledgeRetriever, PruneOptions, Pruner, RunOptions, UploadRequest,
};

fn upload(source_type: &str, file_name: &str, content: &str) -> UploadRequest {
    UploadRequest::try_from(AddKnowledgeDocumentInput {
        source_type: source_type.to_owned(),
        file_name: file_name.to_owned(),
        content: Some(content.to_owned()),
        content_base64: None,
        title: None,
        overwrite: false,
    })
    .expect("valid upload")
}

#[tokio::test]
async fn an_uploaded_note_is_searchable_immediately_without_a_sync() {
    let Some(fx) = Fixture::create_or_skip().await else {
        return;
    };
    let before = fx.model.embeds();
    let report = fx
        .ingestor()
        .add_document(
            &signed_in("u-alice"),
            upload(
                "meeting_notes",
                "2026-09-02_checkout-sync.md",
                "# Checkout sync\n\nWe agreed the express checkout ships behind a feature flag on the 9th.\n",
            ),
        )
        .await
        .expect("upload succeeds");
    assert_eq!(
        report.doc_id,
        "meeting_notes:uploads/2026-09-02_checkout-sync.md"
    );
    assert!(
        report
            .uri
            .ends_with("/meeting_notes/uploads/2026-09-02_checkout-sync.md")
    );
    assert_eq!(report.chunks, 1);
    assert!(!report.replaced);
    assert_eq!(report.granted_to[0].kind, "role");
    assert!(
        fx.model.embeds() > before,
        "it was embedded now, not queued"
    );
    assert!(
        fx.root
            .path()
            .join("meeting_notes/uploads/2026-09-02_checkout-sync.md")
            .is_file(),
        "the bytes are persisted inside the category tree"
    );

    let hits = fx
        .retriever()
        .search(
            &signed_in("u-bob"),
            "express checkout feature flag",
            &filter(&["meeting_notes"]),
        )
        .await
        .expect("search");
    assert_eq!(
        hits.first().map(|h| h.doc_id.as_str()),
        Some(report.doc_id.as_str())
    );
    assert_eq!(hits[0].title, "Checkout sync");

    fx.db.cleanup().await;
}

#[tokio::test]
async fn an_uploaded_document_survives_a_later_full_rebuild_and_prune() {
    let Some(fx) = Fixture::create_or_skip().await else {
        return;
    };
    let ingestor = fx.ingestor();
    fx.sync_files("u-alice", &RunOptions::default()).await;
    let report = ingestor
        .add_document(
            &signed_in("u-alice"),
            upload(
                "adrs",
                "009-flags.md",
                "# ADR-009 Feature flags\n\nEvery risky change ships behind a flag.\n",
            ),
        )
        .await
        .expect("upload");
    let embeds_after_upload = fx.model.embeds();

    let rebuild = ingestor
        .run_with(
            &signed_in("u-alice"),
            &FilesConnector,
            &RunOptions {
                rebuild: true,
                ..RunOptions::default()
            },
        )
        .await
        .expect("rebuild");
    let adrs = rebuild
        .sources
        .iter()
        .find(|s| s.source_type == "adrs")
        .expect("adrs");
    assert_eq!(adrs.seen, 2, "the upload is walked like any other file");
    assert_eq!(adrs.pruned, 0);
    assert_eq!(
        fx.model.embeds(),
        embeds_after_upload,
        "unchanged text hits the embedding cache on rebuild"
    );

    fx.db.age_sync_state("u-alice", 400).await;
    let prune = Pruner::new(fx.db.raw_pool_arc(), fx.db.raw_pool_arc())
        .prune(PruneOptions {
            stale_days: 90,
            dry_run: false,
        })
        .await
        .expect("prune");
    assert_eq!(prune.documents_deleted, 0, "{prune:?}");
    assert_eq!(
        fx.db.grants_of(&report.doc_id).await,
        vec![("role".to_owned(), "user".to_owned())]
    );
    let hits = fx
        .retriever()
        .search(
            &signed_in("u-bob"),
            "risky change ships behind a flag",
            &filter(&["adrs"]),
        )
        .await
        .expect("search");
    assert_eq!(
        hits.first().map(|h| h.doc_id.as_str()),
        Some(report.doc_id.as_str())
    );

    fx.db.cleanup().await;
}

#[tokio::test]
async fn an_uploaded_binary_arrives_as_base64_and_is_parsed() {
    let Some(fx) = Fixture::create_or_skip().await else {
        return;
    };
    let bytes = docx(&[
        "Vault decision",
        "Tokenised cards need a consent checkbox and a PSP vault call.",
    ]);
    let request = UploadRequest::try_from(AddKnowledgeDocumentInput {
        source_type: "meeting_notes".to_owned(),
        file_name: "vault.docx".to_owned(),
        content: None,
        content_base64: Some(base64::engine::general_purpose::STANDARD.encode(&bytes)),
        title: Some("Vault decision (2026-02-02)".to_owned()),
        overwrite: false,
    })
    .expect("valid");
    let report = fx
        .ingestor()
        .add_document(&signed_in("u-alice"), request)
        .await
        .expect("docx upload");
    assert_eq!(report.title, "Vault decision (2026-02-02)");
    assert_eq!(
        std::fs::read(fx.root.path().join("meeting_notes/uploads/vault.docx")).expect("stored"),
        bytes,
        "the original bytes are kept, not the extracted text"
    );
    let hits = fx
        .retriever()
        .search(
            &signed_in("u-bob"),
            "consent checkbox PSP vault",
            &filter(&["meeting_notes"]),
        )
        .await
        .expect("search");
    assert_eq!(
        hits.first().map(|h| h.uri.as_str()),
        Some(report.uri.as_str())
    );

    fx.db.cleanup().await;
}

#[tokio::test]
async fn a_second_upload_of_the_same_name_needs_overwrite_and_replaces_one_document() {
    let Some(fx) = Fixture::create_or_skip().await else {
        return;
    };
    let ingestor = fx.ingestor();
    ingestor
        .add_document(
            &signed_in("u-alice"),
            upload("adrs", "010.md", "# ADR-010 First draft\n\nUse queues.\n"),
        )
        .await
        .expect("first upload");
    let err = ingestor
        .add_document(
            &signed_in("u-bob"),
            upload("adrs", "010.md", "# ADR-010 Second\n\nUse streams.\n"),
        )
        .await
        .expect_err("the name is taken");
    assert!(
        matches!(err, KnowledgeBankError::InvalidInput(ref m) if m.contains("overwrite")),
        "{err}"
    );

    let mut replacement = upload(
        "adrs",
        "010.md",
        "# ADR-010 Second\n\nUse streams instead of queues.\n",
    );
    replacement.overwrite = true;
    let report = ingestor
        .add_document(&signed_in("u-admin"), replacement)
        .await
        .expect("overwrite");
    assert!(report.replaced);
    let docs = fx.db.count("knowledge_documents").await;
    assert_eq!(docs, 1, "one row for the name, whatever its history");
    let hits = fx
        .retriever()
        .search(
            &signed_in("u-bob"),
            "streams instead of queues",
            &filter(&["adrs"]),
        )
        .await
        .expect("search");
    assert_eq!(hits[0].title, "ADR-010 Second");

    fx.db.cleanup().await;
}

#[tokio::test]
async fn an_upload_into_an_atlassian_category_or_an_unknown_one_is_refused() {
    let Some(fx) = Fixture::create_or_skip().await else {
        return;
    };
    let ingestor = fx.ingestor();
    let err = ingestor
        .add_document(
            &signed_in("u-alice"),
            upload("confluence", "page.md", "# x\n\nbody"),
        )
        .await
        .expect_err("confluence is synced, never uploaded to");
    assert!(
        matches!(err, KnowledgeBankError::InvalidInput(ref m) if m.contains("files categories")),
        "{err}"
    );
    let err = ingestor
        .add_document(
            &signed_in("u-alice"),
            upload("sharepoint", "page.md", "# x\n\nbody"),
        )
        .await
        .expect_err("unknown category");
    assert!(matches!(err, KnowledgeBankError::UnknownSource(_)));
    assert_eq!(fx.db.count("knowledge_documents").await, 0);

    fx.db.cleanup().await;
}
