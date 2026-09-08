//! The Jira and Confluence connectors, against a fake Atlassian MCP gateway.
//!
//! Both read as the caller through this instance's own gateway route, page
//! through the hosted server's tools, and normalise what comes back into the
//! documents the index is built from. What matters here is the normalisation
//! — the title, the citable `uri`, the content hash that decides whether a
//! document is re-embedded, and the `mtime` that dates it — plus the paging,
//! the incremental cursor that skips an unchanged page the index already
//! holds (FR-ING-2, FR-ING-3), the site resolution, and the failure shapes:
//! a wrong bearer, an account reaching no or many sites, a refused tool, and
//! a space the caller cannot see.

use serde_json::json;
use std::collections::HashSet;
use systemprompt_mcp_knowledge_bank::config::{AtlassianSettings, FilesSettings};
use systemprompt_mcp_knowledge_bank::error::KnowledgeBankError;
use systemprompt_mcp_knowledge_bank::ingest::sources::atlassian::text_of;
use systemprompt_mcp_knowledge_bank::ingest::sources::{FetchRequest, confluence, jira};

use crate::atlassian_mcp_gateway::{
    CLOUD_ID, FakeGateway, SITE, confluence_source, confluence_space, issue, jira_source, page,
    settings,
};

fn request<'a>(
    source_type: &'a str,
    source: &'a systemprompt_mcp_knowledge_bank::config::SourceConfig,
    known: &'a HashSet<String>,
    include_comments: bool,
) -> FetchRequest<'a> {
    static FILES: FilesSettings = FilesSettings {
        root: String::new(),
    };
    FetchRequest {
        source_type,
        source,
        files: &FILES,
        last_sync: None,
        include_comments,
        known_doc_ids: known,
    }
}

#[tokio::test]
async fn the_connection_is_bound_to_the_callers_bearer_and_the_one_reachable_site() {
    let gateway = FakeGateway::start().await;

    let client = gateway.connect().await;
    assert_eq!(client.cloud_id(), CLOUD_ID);
    assert_eq!(client.site_url(), SITE);

    // Why: the gateway answers `initialize` for the caller's bearer only; a
    // different one never reaches Atlassian, and the sync must say so rather
    // than index nothing quietly.
    let err = gateway
        .try_connect("someone-else", settings())
        .await
        .expect_err("a foreign bearer is refused");
    assert!(
        matches!(err, KnowledgeBankError::AtlassianUnavailable(_)),
        "{err}"
    );
    assert!(err.to_string().contains("connected-accounts"), "{err}");

    let err = gateway
        .try_connect("", settings())
        .await
        .expect_err("no bearer at all");
    assert!(matches!(err, KnowledgeBankError::AtlassianUnavailable(_)));
}

#[tokio::test]
async fn an_account_reaching_several_sites_needs_a_configured_cloud_id() {
    let gateway = FakeGateway::start_with_sites(&["site-a", "site-b"]).await;

    let err = gateway
        .try_connect(crate::atlassian_mcp_gateway::BEARER, settings())
        .await
        .expect_err("two sites is a configuration question");
    assert!(matches!(err, KnowledgeBankError::Config(_)), "{err}");
    assert!(err.to_string().contains("atlassian.cloud_id"), "{err}");

    let pinned = AtlassianSettings {
        cloud_id: Some("site-b".to_owned()),
        ..settings()
    };
    let client = gateway
        .try_connect(crate::atlassian_mcp_gateway::BEARER, pinned)
        .await
        .expect("a configured id picks the site");
    assert_eq!(client.cloud_id(), "site-b");

    let elsewhere = AtlassianSettings {
        cloud_id: Some("site-z".to_owned()),
        ..settings()
    };
    let err = gateway
        .try_connect(crate::atlassian_mcp_gateway::BEARER, elsewhere)
        .await
        .expect_err("a site the account cannot reach");
    assert!(
        matches!(err, KnowledgeBankError::AtlassianUnavailable(_)),
        "{err}"
    );

    let none = FakeGateway::start_with_sites(&[]).await;
    let err = none
        .try_connect(crate::atlassian_mcp_gateway::BEARER, settings())
        .await
        .expect_err("no site at all");
    assert!(
        matches!(err, KnowledgeBankError::AtlassianUnavailable(_)),
        "{err}"
    );
}

#[tokio::test]
async fn a_jira_issue_becomes_a_document_that_cites_where_it_came_from() {
    let gateway = FakeGateway::start().await;
    gateway
        .on_call(
            "searchJiraIssuesUsingJql",
            json!({"cloudId": CLOUD_ID}),
            json!({"issues": [issue("ENG-42", "Grant documents at ingest", "2026-08-30T14:00:00.000+0000")], "isLast": true}),
        )
        .await;
    let client = gateway.connect().await;
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
    assert_eq!(doc.uri, format!("{SITE}/browse/ENG-42"));
    assert!(
        doc.body.contains("grant each document to its reader"),
        "the description is carried: {}",
        doc.body
    );
    assert!(
        doc.body.contains("**Labels:** governance, rag"),
        "{}",
        doc.body
    );
    assert!(!doc.content_hash.is_empty());
    // Why: Jira Cloud writes its offset without a colon (`+0000`), which is
    // not RFC 3339. Reading it is what gives the document a date from its
    // own source system rather than from the model's reading of the body.
    assert_eq!(
        doc.mtime
            .map(|m| m.format("%Y-%m-%d").to_string())
            .as_deref(),
        Some("2026-08-30"),
        "the issue is dated by its last update"
    );
    assert_eq!(
        doc.authored_date.map(|d| d.to_string()).as_deref(),
        Some("2026-08-30"),
        "and that date is what the dating step takes as authoritative"
    );
    assert_eq!(outcome.current_refs, vec!["ENG-42".to_owned()]);
    assert!(
        outcome.complete,
        "a full run reports a complete ref set, so grants may be dropped against it"
    );
}

#[tokio::test]
async fn comments_are_included_only_when_the_source_asks_for_them() {
    let gateway = FakeGateway::start().await;
    gateway
        .on_call(
            "searchJiraIssuesUsingJql",
            json!({}),
            json!({"issues": [issue("ENG-1", "Discussion", "2026-08-30T14:00:00.000+0000")], "isLast": true}),
        )
        .await;
    let client = gateway.connect().await;
    let source = jira_source(&["ENG"], None);
    let known = HashSet::new();

    let included = jira::fetch(&request("delivery", &source, &known, true), &client)
        .await
        .expect("fetch with comments");
    let excluded = jira::fetch(&request("delivery", &source, &known, false), &client)
        .await
        .expect("fetch without comments");

    assert!(
        included.documents[0]
            .body
            .contains("grants go in the same transaction"),
        "the comment thread is part of the document when asked for"
    );
    assert!(
        !excluded.documents[0]
            .body
            .contains("grants go in the same transaction"),
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
async fn a_thread_the_search_did_not_carry_inline_is_read_per_issue() {
    let gateway = FakeGateway::start().await;
    let mut bare = issue("ENG-7", "Bare", "2026-08-30T14:00:00.000+0000");
    bare["fields"]
        .as_object_mut()
        .expect("fields")
        .remove("comment");
    gateway
        .on_call(
            "searchJiraIssuesUsingJql",
            json!({}),
            json!({"issues": [bare], "isLast": true}),
        )
        .await;
    gateway
        .on_call(
            "listJiraIssueComments",
            json!({"issueIdOrKey": "ENG-7"}),
            json!({"comments": [{
                "author": {"displayName": "Walter Skinner"},
                "created": "2026-08-21T08:00:00.000+0000",
                "body": {"type": "doc", "content": [{"type": "paragraph", "content": [{"type": "text", "text": "Read from the comments tool."}]}]}
            }]}),
        )
        .await;
    let client = gateway.connect().await;
    let source = jira_source(&["ENG"], None);
    let known = HashSet::new();

    let outcome = jira::fetch(&request("delivery", &source, &known, true), &client)
        .await
        .expect("fetch");

    let body = &outcome.documents[0].body;
    assert!(body.contains("Walter Skinner"), "{body}");
    assert!(
        body.contains("Read from the comments tool."),
        "an ADF body is flattened to its text: {body}"
    );
}

#[tokio::test]
async fn jira_follows_the_page_token_until_the_last_page() {
    let gateway = FakeGateway::start().await;
    gateway
        .on_call_at(
            "searchJiraIssuesUsingJql",
            json!({"nextPageToken": "page-2"}),
            json!({"issues": [issue("ENG-2", "Second", "2026-08-02T09:00:00.000+0000")], "isLast": true}),
            1,
        )
        .await;
    gateway
        .on_call(
            "searchJiraIssuesUsingJql",
            json!({}),
            json!({"issues": [issue("ENG-1", "First", "2026-08-01T09:00:00.000+0000")], "nextPageToken": "page-2", "isLast": false}),
        )
        .await;
    let client = gateway.connect().await;
    let source = jira_source(&["ENG"], None);
    let known = HashSet::new();

    let outcome = jira::fetch(&request("delivery", &source, &known, false), &client)
        .await
        .expect("the paged fetch succeeds");

    assert_eq!(
        outcome.current_refs,
        vec!["ENG-1".to_owned(), "ENG-2".to_owned()],
        "both pages were read, in order"
    );
}

#[tokio::test]
async fn a_refused_jira_read_names_the_tool_and_atlassians_reason() {
    let gateway = FakeGateway::start().await;
    gateway
        .on_call_error(
            "searchJiraIssuesUsingJql",
            "user does not have permission to view this project",
        )
        .await;
    let client = gateway.connect().await;
    let source = jira_source(&["SECRET"], None);
    let known = HashSet::new();

    let error = jira::fetch(&request("delivery", &source, &known, false), &client)
        .await
        .expect_err("a refused search surfaces");

    let message = error.to_string();
    assert!(message.contains("searchJiraIssuesUsingJql"), "{message}");
    assert!(message.contains("does not have permission"), "{message}");
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
async fn a_confluence_page_becomes_a_document_with_its_markdown_and_comments() {
    let gateway = FakeGateway::start().await;
    confluence_space(
        &gateway,
        "ENG",
        "1001",
        vec![page(
            "77",
            "Deployment runbook",
            "## Deploying\n\nRun `just deploy-next`.",
            "2026-08-29T10:00:00.000Z",
        )],
    )
    .await;
    gateway
        .on_call(
            "listConfluenceComments",
            json!({"content-id": "77"}),
            json!({"comments": [{
                "id": "9",
                "authorAccountId": "712020:abc",
                "createdAt": "2026-08-30T11:48:32.356Z",
                "body": {"format": "markdown", "value": "Rollback is `just deploy` on main.\n"},
                "replies": []
            }]}),
        )
        .await;
    let client = gateway.connect().await;
    let source = confluence_source(&["ENG"]);
    let known = HashSet::new();

    let outcome = confluence::fetch(&request("engineering", &source, &known, true), &client)
        .await
        .expect("the fetch succeeds");

    assert_eq!(outcome.documents.len(), 1, "{outcome:?}");
    let doc = &outcome.documents[0];
    assert_eq!(doc.title, "Deployment runbook");
    assert!(doc.body.contains("just deploy-next"), "{}", doc.body);
    assert!(
        doc.body.contains("## Comments") && doc.body.contains("Rollback is"),
        "footer comments follow the page: {}",
        doc.body
    );
    assert_eq!(
        doc.uri,
        format!("{SITE}/wiki/spaces/ENG/pages/77"),
        "the citation points back at the wiki"
    );
    assert_eq!(
        doc.authored_date.map(|d| d.to_string()).as_deref(),
        Some("2026-08-29"),
        "the RFC 3339 version timestamp dates the page"
    );
}

#[tokio::test]
async fn a_space_the_caller_cannot_read_is_reported_as_such_rather_than_as_empty() {
    let gateway = FakeGateway::start().await;
    gateway
        .on_call(
            "listConfluenceSpaces",
            json!({"keys": "SECRET"}),
            json!({"results": []}),
        )
        .await;
    let client = gateway.connect().await;
    let source = confluence_source(&["SECRET"]);
    let known = HashSet::new();

    let error = confluence::fetch(&request("engineering", &source, &known, false), &client)
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
async fn an_unchanged_confluence_page_is_skipped_by_the_callers_cursor() {
    let gateway = FakeGateway::start().await;
    confluence_space(
        &gateway,
        "ENG",
        "1001",
        vec![
            page(
                "77",
                "Deployment runbook",
                "unchanged",
                "2026-08-29T10:00:00.000Z",
            ),
            page(
                "78",
                "New page",
                "new since the cursor",
                "2026-09-01T10:00:00.000Z",
            ),
        ],
    )
    .await;
    let client = gateway.connect().await;
    let source = confluence_source(&["ENG"]);
    let known: HashSet<String> = ["engineering:77".to_owned()].into_iter().collect();
    let mut req = request("engineering", &source, &known, false);
    req.last_sync = Some(
        chrono::DateTime::parse_from_rfc3339("2026-08-30T00:00:00Z")
            .expect("cursor")
            .with_timezone(&chrono::Utc),
    );

    let outcome = confluence::fetch(&req, &client)
        .await
        .expect("the fetch succeeds");

    assert_eq!(outcome.unchanged, vec!["engineering:77".to_owned()]);
    assert_eq!(outcome.documents.len(), 1);
    assert_eq!(outcome.documents[0].source_ref, "78");
    let mut refs = outcome.current_refs.clone();
    refs.sort();
    assert_eq!(
        refs,
        ["77", "78"],
        "both pages are current, so neither is pruned"
    );
    assert!(outcome.complete);
}

#[tokio::test]
async fn a_page_that_cannot_be_read_is_skipped_without_losing_the_space() {
    let gateway = FakeGateway::start().await;
    confluence_space(
        &gateway,
        "ENG",
        "1001",
        vec![page("1", "Readable", "fine", "2026-08-29T10:00:00.000Z")],
    )
    .await;
    // A second page is listed but its read is refused.
    gateway
        .on_call_at(
            "listConfluenceContent",
            json!({"spaceId": "1001"}),
            json!({"results": [
                {"id": "1", "title": "Readable", "type": "page", "status": "current", "spaceId": "1001"},
                {"id": "2", "title": "Restricted", "type": "page", "status": "current", "spaceId": "1001"}
            ]}),
            1,
        )
        .await;
    gateway
        .on_call_error("getConfluenceContent", "restricted")
        .await;
    gateway
        .on_call_at(
            "getConfluenceContent",
            json!({"content_id": "1"}),
            page("1", "Readable", "fine", "2026-08-29T10:00:00.000Z"),
            1,
        )
        .await;
    let client = gateway.connect().await;
    let source = confluence_source(&["ENG"]);
    let known = HashSet::new();

    let outcome = confluence::fetch(&request("engineering", &source, &known, false), &client)
        .await
        .expect("one bad page does not fail the space");

    assert_eq!(outcome.documents.len(), 1);
    assert_eq!(outcome.documents[0].source_ref, "1");
    let mut refs = outcome.current_refs.clone();
    refs.sort();
    assert_eq!(refs, ["1", "2"], "the listed page keeps its grant");
    assert!(outcome.complete, "the listing itself was complete");
}

#[tokio::test]
async fn debug_output_never_shows_the_callers_bearer() {
    let gateway = FakeGateway::start().await;
    let client = gateway.connect().await;
    let shown = format!("{client:?}");
    assert!(shown.contains("cloud_id"), "{shown}");
    assert!(
        !shown.contains(crate::atlassian_mcp_gateway::BEARER),
        "the bearer is never printed: {shown}"
    );
}

#[test]
fn rich_text_arrives_in_four_shapes_and_leaves_as_markdown() {
    assert_eq!(text_of(Some(&json!("plain **md**"))), "plain **md**");
    assert_eq!(
        text_of(Some(&json!({"format": "markdown", "value": "wrapped"}))),
        "wrapped"
    );
    let html = text_of(Some(&json!("<p>Hello <code>world</code></p>")));
    assert!(html.contains("Hello") && !html.contains("<p>"), "{html}");
    let adf = text_of(Some(&json!({"type": "doc", "content": [
        {"type": "paragraph", "content": [{"type": "text", "text": "one"}]},
        {"type": "paragraph", "content": [{"type": "text", "text": "two"}]}
    ]})));
    assert_eq!(adf, "one\ntwo");
    assert_eq!(text_of(None), "");
}
