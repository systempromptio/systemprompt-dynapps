//! The Jira and Confluence connectors, against a mocked Atlassian.
//!
//! These are the crate's two outbound HTTP clients and both were entirely
//! untested: they read as the caller, page through a REST API, and normalise
//! what comes back into the documents the index is built from. What matters
//! here is the normalisation — the title, the citable `uri`, the content hash
//! that decides whether a document is re-embedded, and the `mtime` that dates
//! it — plus the paging and the two failure shapes, an upstream error and a
//! space the caller cannot see.

use serde_json::json;
use systemprompt_mcp_knowledge_bank::ingest::sources::{FetchRequest, confluence, jira};
use std::collections::HashSet;

use crate::atlassian_mock_http::{
    client_for, confluence_source, confluence_space, confluence_space_missing, issue, jira_search,
    jira_search_failing, jira_source, page,
};

fn request<'a>(
    source_type: &'a str,
    source: &'a systemprompt_mcp_knowledge_bank::config::SourceConfig,
    known: &'a HashSet<String>,
    include_comments: bool,
) -> FetchRequest<'a> {
    FetchRequest {
        source_type,
        source,
        last_sync: None,
        include_comments,
        known_doc_ids: known,
    }
}

#[tokio::test]
async fn a_jira_issue_becomes_a_document_that_cites_where_it_came_from() {
    let server = jira_search(vec![json!({
        "issues": [issue("ENG-42", "Grant documents at ingest", "2026-08-30T14:00:00.000+0000")],
        "isLast": true
    })])
    .await;
    let client = client_for(&server);
    let source = jira_source(&["ENG"], None);
    let known = HashSet::new();

    let outcome = jira::fetch(&request("delivery", &source, &known, true), &client)
        .await
        .expect("the fetch succeeds");

    assert_eq!(outcome.documents.len(), 1);
    let doc = &outcome.documents[0];
    assert_eq!(doc.source_ref, "ENG-42");
    assert_eq!(doc.doc_id, "delivery:ENG-42");
    assert_eq!(doc.title, "[ENG-42] Grant documents at ingest");
    // Why: the agent policy tells the model to open the cited uri, so a
    // document that cannot be traced back to its issue is worse than absent.
    assert_eq!(doc.uri, format!("{}/browse/ENG-42", server.uri()));
    assert!(
        doc.body.contains("grant each document to its reader"),
        "the rendered description is carried, not the ADF: {}",
        doc.body
    );
    assert!(!doc.content_hash.is_empty());
    // DEFECT, pinned deliberately: Jira Cloud renders timestamps with a
    // numeric offset and no colon (`+0000`), which `DateTime::parse_from_rfc3339`
    // rejects, so every Jira issue arrives undated. Confluence's `Z` form parses
    // and dates correctly, which is why this went unnoticed. Fixing the parser
    // should flip this assertion to `Some("2026-08-30")`.
    assert_eq!(
        doc.mtime, None,
        "a Jira `+0000` timestamp is currently dropped rather than parsed"
    );
    assert_eq!(outcome.current_refs, vec!["ENG-42".to_owned()]);
    assert!(
        outcome.complete,
        "a full run reports a complete ref set, so grants may be dropped against it"
    );
}

#[tokio::test]
async fn comments_are_included_only_when_the_source_asks_for_them() {
    let with_comments = jira_search(vec![json!({
        "issues": [issue("ENG-1", "Discussion", "2026-08-30T14:00:00.000+0000")],
        "isLast": true
    })])
    .await;
    let without = jira_search(vec![json!({
        "issues": [issue("ENG-1", "Discussion", "2026-08-30T14:00:00.000+0000")],
        "isLast": true
    })])
    .await;
    let source = jira_source(&["ENG"], None);
    let known = HashSet::new();

    let included = jira::fetch(
        &request("delivery", &source, &known, true),
        &client_for(&with_comments),
    )
    .await
    .expect("fetch with comments");
    let excluded = jira::fetch(
        &request("delivery", &source, &known, false),
        &client_for(&without),
    )
    .await
    .expect("fetch without comments");

    assert!(
        included.documents[0].body.contains("grants go in the same transaction"),
        "the comment thread is part of the document when asked for"
    );
    assert!(
        !excluded.documents[0].body.contains("grants go in the same transaction"),
        "and is left out when it is not"
    );
    // Why: the hash is what decides a re-embed. Two renderings of the same
    // issue that differ in content must not collide, or a source that starts
    // including comments would never re-index.
    assert_ne!(
        included.documents[0].content_hash,
        excluded.documents[0].content_hash
    );
}

#[tokio::test]
async fn jira_follows_the_page_token_until_the_last_page() {
    let server = jira_search(vec![
        json!({
            "issues": [issue("ENG-1", "First", "2026-08-01T09:00:00.000+0000")],
            "nextPageToken": "page-2",
            "isLast": false
        }),
        json!({
            "issues": [issue("ENG-2", "Second", "2026-08-02T09:00:00.000+0000")],
            "isLast": true
        }),
    ])
    .await;
    let source = jira_source(&["ENG"], None);
    let known = HashSet::new();

    let outcome = jira::fetch(
        &request("delivery", &source, &known, false),
        &client_for(&server),
    )
    .await
    .expect("the paged fetch succeeds");

    assert_eq!(
        outcome.current_refs,
        vec!["ENG-1".to_owned(), "ENG-2".to_owned()],
        "both pages were read, in order"
    );
}

#[tokio::test]
async fn an_upstream_jira_failure_names_the_endpoint_and_the_status() {
    let server = jira_search_failing(403, "user does not have permission to view this project").await;
    let source = jira_source(&["SECRET"], None);
    let known = HashSet::new();

    let error = jira::fetch(
        &request("delivery", &source, &known, false),
        &client_for(&server),
    )
    .await
    .expect_err("a 403 from Jira surfaces");

    let message = error.to_string();
    assert!(message.contains("403"), "the status is reported: {message}");
    assert!(
        message.contains("/rest/api/3/search/jql"),
        "and so is the call that failed: {message}"
    );
}

#[tokio::test]
async fn the_jql_narrows_to_the_projects_and_carries_the_incremental_cursor() {
    let full = jira::build_jql(&["ENG".to_owned(), "OPS".to_owned()], None, None);
    assert!(full.contains(r#"project IN ("ENG", "OPS")"#), "{full}");
    assert!(
        full.ends_with("ORDER BY updated ASC"),
        "oldest first, so a partial run resumes cleanly: {full}"
    );

    let since = chrono::DateTime::parse_from_rfc3339("2026-08-30T14:30:00Z")
        .expect("a fixed cursor")
        .with_timezone(&chrono::Utc);
    let incremental = jira::build_jql(&["ENG".to_owned()], Some("labels = rag"), Some(since));
    assert!(incremental.contains("(labels = rag)"), "{incremental}");
    assert!(
        incremental.contains(r#"updated >= "2026-08-30 14:30""#),
        "the cursor is what makes a sync incremental: {incremental}"
    );
}

#[tokio::test]
async fn a_confluence_page_becomes_a_document_with_its_html_reduced_to_markdown() {
    let server = confluence_space(
        "ENG",
        "1001",
        json!({
            "results": [page(
                "77",
                "Deployment runbook",
                "<h2>Deploying</h2><p>Run <code>just deploy-next</code>.</p>",
                "2026-08-29T10:00:00.000Z"
            )],
            "_links": {}
        }),
    )
    .await;
    let client = client_for(&server);
    let source = confluence_source(&["ENG"]);
    let known = HashSet::new();

    let outcome = confluence::fetch(&request("engineering", &source, &known, false), &client)
        .await
        .expect("the fetch succeeds");

    assert_eq!(outcome.documents.len(), 1, "{outcome:?}");
    let doc = &outcome.documents[0];
    assert_eq!(doc.title, "Deployment runbook");
    assert!(
        doc.body.contains("just deploy-next"),
        "the page body survives the HTML reduction: {}",
        doc.body
    );
    assert!(
        !doc.body.contains("<code>"),
        "and the markup does not: {}",
        doc.body
    );
    assert!(
        doc.uri.starts_with(&server.uri()),
        "the citation points back at the wiki: {}",
        doc.uri
    );
}

#[tokio::test]
async fn a_space_the_caller_cannot_read_is_reported_as_such_rather_than_as_empty() {
    let server = confluence_space_missing("SECRET").await;
    let source = confluence_source(&["SECRET"]);
    let known = HashSet::new();

    let error = confluence::fetch(
        &request("engineering", &source, &known, false),
        &client_for(&server),
    )
    .await
    .expect_err("an unreadable space is an error");

    let message = error.to_string();
    // Why: an empty result here would silently drop every grant for the space
    // on the next sync, as though its pages had been deleted.
    assert!(message.contains("SECRET"), "{message}");
    assert!(
        message.contains("not found") || message.contains("not readable"),
        "the message distinguishes absent from forbidden: {message}"
    );
}

#[tokio::test]
async fn the_two_atlassian_timestamp_forms_are_not_parsed_alike() {
    // Confluence v2 answers in `Z` form and Jira Cloud in `+0000` form. Only
    // the first is RFC 3339, and `json_date` accepts nothing else — so a Jira
    // issue's `updated` is discarded while a Confluence page's `createdAt` is
    // kept. This is the root of the undated-issue defect above; it is asserted
    // here on the parser itself so the fix has one obvious place to land.
    use chrono::DateTime;

    assert!(
        DateTime::parse_from_rfc3339("2026-08-29T10:00:00.000Z").is_ok(),
        "the Confluence form parses"
    );
    assert!(
        DateTime::parse_from_rfc3339("2026-08-30T14:00:00.000+0000").is_err(),
        "and the Jira form does not, which is why Jira documents arrive undated"
    );
}
