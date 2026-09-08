//! Durable Drive queue, deletion safety and project-filtered retrieval.

use crate::common::TempDb;
use crate::fake_model::FakeModel;
use crate::knowledge_files::signed_in;
use std::sync::Arc;
use systemprompt_mcp_knowledge_bank::config::KnowledgeBankConfig;
use systemprompt_mcp_knowledge_bank::drive::store;
use systemprompt_mcp_knowledge_bank::drive::types::{DriveFile, ProjectAssignment};
use systemprompt_mcp_knowledge_bank::ingest::document::{SourceDocument, content_hash};
use systemprompt_mcp_knowledge_bank::ingest::pipeline::{DocumentJob, process_document};
use systemprompt_mcp_knowledge_bank::ingest::writer;
use systemprompt_mcp_knowledge_bank::{KnowledgeRetriever, PgRetriever, SearchFilter};

fn file(id: &str, version: &str) -> DriveFile {
    DriveFile {
        id: id.to_owned(),
        name: format!("{id}.txt"),
        mime_type: "text/plain".to_owned(),
        version: Some(version.to_owned()),
        ..Default::default()
    }
}

#[tokio::test]
async fn requests_during_a_run_survive_and_incomplete_inventory_never_prunes() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    db.create_knowledge_schema().await;
    let pool = db.raw_pool_arc();
    store::enqueue(
        &pool,
        "marketing",
        "drive",
        "u-alice",
        store::SyncRequest {
            force: true,
            rebuild: false,
        },
    )
    .await
    .unwrap();
    store::save_file(&pool, "marketing", &file("a", "1"), Some("first"))
        .await
        .unwrap();
    store::save_file(&pool, "marketing", &file("b", "1"), Some("first"))
        .await
        .unwrap();
    store::finish_inventory(&pool, "marketing", "first", "token-1")
        .await
        .unwrap();
    store::enqueue(
        &pool,
        "marketing",
        "drive",
        "u-bob",
        store::SyncRequest {
            force: true,
            rebuild: false,
        },
    )
    .await
    .unwrap();
    store::complete_listing(&pool, "marketing", 1, false)
        .await
        .unwrap();
    let state = store::state(&pool, "marketing").await.unwrap().unwrap();
    assert_eq!(state.requested, 2);
    assert_eq!(state.completed, 1);
    assert!(state.force_requested);
    store::save_file(&pool, "marketing", &file("a", "2"), Some("partial"))
        .await
        .unwrap();
    let live = store::files(&pool, "marketing").await.unwrap();
    assert_eq!(
        live.len(),
        2,
        "an interrupted inventory cannot infer removal"
    );
    assert_eq!(
        store::state(&pool, "marketing")
            .await
            .unwrap()
            .unwrap()
            .page_token
            .as_deref(),
        Some("token-1")
    );
    store::finish_inventory(&pool, "marketing", "partial", "token-2")
        .await
        .unwrap();
    let pending = store::pending(&pool, "marketing").await.unwrap();
    assert!(pending.iter().any(|f| f.file_id == "b" && f.removed));
    assert_eq!(store::files(&pool, "marketing").await.unwrap().len(), 1);
    db.cleanup().await;
}

#[tokio::test]
async fn extraction_checkpoint_survives_replay_but_new_version_invalidates_it() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    db.create_knowledge_schema().await;
    let pool = db.raw_pool_arc();
    store::enqueue(
        &pool,
        "marketing",
        "drive",
        "u-alice",
        store::SyncRequest {
            force: false,
            rebuild: false,
        },
    )
    .await
    .unwrap();
    let original = file("a", "1");
    store::save_file(&pool, "marketing", &original, None)
        .await
        .unwrap();
    store::checkpoint(
        &pool,
        "marketing",
        "a",
        store::Checkpoint {
            key: "cache-key",
            arn: "arn:job",
            output: "s3://bucket/output/",
            local_text: "local notes",
        },
    )
    .await
    .unwrap();
    store::save_file(&pool, "marketing", &original, None)
        .await
        .unwrap();
    let arn: Option<String> =
        sqlx::query_scalar("SELECT invocation_arn FROM knowledge_drive_files WHERE file_id='a'")
            .fetch_one(&*pool)
            .await
            .unwrap();
    assert_eq!(arn.as_deref(), Some("arn:job"));
    store::save_file(&pool, "marketing", &file("a", "2"), None)
        .await
        .unwrap();
    let rows = store::pending(&pool, "marketing").await.unwrap();
    assert_eq!(rows.len(), 1);
    assert!(rows[0].invocation_arn.is_none());
    assert!(rows[0].extracted_text.is_none());
    db.cleanup().await;
}

#[tokio::test]
async fn project_filter_precedes_ranking_shared_is_optional_and_moves_do_not_duplicate() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    db.create_knowledge_schema().await;
    let pool = db.raw_pool_arc();
    let config=KnowledgeBankConfig::parse("chunking: {target_chars: 200, max_chars: 400, min_chars: 40, enrich: false}\ndates: {extract_via_llm: false}\nsources:\n  marketing: {kind: google_drive, strategy: script}\ngoogle_drive:\n  sources:\n    marketing: {drive_id: drive, credential_secret: google}\n").unwrap();
    let model = Arc::new(FakeModel::default());
    store::enqueue(
        &pool,
        "marketing",
        "drive",
        "u-alice",
        store::SyncRequest {
            force: false,
            rebuild: false,
        },
    )
    .await
    .unwrap();
    for (id, project) in [("a", "launch"), ("b", "other"), ("c", "shared")] {
        let file = file(id, "1");
        store::save_file(&pool, "marketing", &file, None)
            .await
            .unwrap();
        store::assign(
            &pool,
            "marketing",
            id,
            &ProjectAssignment {
                id: project.to_owned(),
                name: project.to_owned(),
                path: format!("Projects/{project}/{}.txt", file.id),
            },
        )
        .await
        .unwrap();
        let body = format!(
            "# Launch marketing\n\nProject {project} plans a launch campaign with customer interviews, product positioning and a launch schedule. The approved campaign starts next month and includes a public announcement."
        );
        let doc = SourceDocument {
            doc_id: format!("marketing:{id}"),
            source_type: "marketing".to_owned(),
            source: "google_drive".to_owned(),
            source_ref: id.to_owned(),
            title: file.name.clone(),
            uri: file.uri(),
            content_hash: content_hash(&body),
            body,
            mtime: None,
            authored_date: None,
        };
        process_document(&DocumentJob {
            pool: &pool,
            model: model.as_ref(),
            config: &config,
            source: config.source("marketing").unwrap(),
            document: &doc,
        })
        .await
        .unwrap();
        let pending = store::pending(&pool, "marketing")
            .await
            .unwrap()
            .into_iter()
            .find(|f| f.file_id == id)
            .unwrap();
        store::indexed(&pool, "marketing", &pending, &doc.doc_id)
            .await
            .unwrap();
        writer::replace_configured_grants(
            &pool,
            &doc.doc_id,
            &config.source("marketing").unwrap().configured_subjects(),
        )
        .await
        .unwrap();
    }
    let retriever = PgRetriever::new(pool.clone(), model.clone(), config);
    let mut principal = signed_in("u-bob");
    principal.roles = vec!["user".to_owned()];
    let mut filter = SearchFilter {
        source_types: vec!["marketing".to_owned()],
        project_ids: vec!["launch".to_owned()],
        exclude_shared: false,
        top_k: Some(20),
    };
    let hits = retriever
        .search(&principal, "launch marketing", &filter)
        .await
        .unwrap();
    assert!(hits.iter().any(|h| h.doc_id == "marketing:a"));
    assert!(hits.iter().any(|h| h.doc_id == "marketing:c"));
    assert!(hits.iter().all(|h| h.doc_id != "marketing:b"));
    filter.exclude_shared = true;
    assert!(
        retriever
            .search(&principal, "launch", &filter)
            .await
            .unwrap()
            .iter()
            .all(|h| h.doc_id == "marketing:a")
    );
    let calls = model.embeds();
    store::assign(
        &pool,
        "marketing",
        "a",
        &ProjectAssignment {
            id: "other".to_owned(),
            name: "Other".to_owned(),
            path: "Projects/Other/a.txt".to_owned(),
        },
    )
    .await
    .unwrap();
    assert_eq!(model.embeds(), calls, "moving a file is a metadata update");
    assert!(
        retriever
            .search(&principal, "launch", &filter)
            .await
            .unwrap()
            .is_empty()
    );
    store::delete_document(&pool, "marketing", "a", "marketing:a")
        .await
        .unwrap();
    let chunks: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM knowledge_chunks WHERE doc_id='marketing:a'")
            .fetch_one(&*pool)
            .await
            .unwrap();
    assert_eq!(chunks, 0);
    db.cleanup().await;
}
