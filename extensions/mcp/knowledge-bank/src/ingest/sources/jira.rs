//! Jira connector: issues of the configured projects (narrowed by optional
//! JQL) with status header, rendered description, and comments, laid out so
//! the `none` chunk strategy can split body from thread.
//!
//! Incremental syncs add `updated >= last_sync` to the JQL and therefore see
//! only changed issues, so they are never `complete`; a full sync reports
//! every key and is. Jira ran the search as the caller, so an issue behind a
//! security level the caller holds is indexed like any other and granted to
//! the caller alone.

use super::FetchRequest;
use super::atlassian::{AtlassianClient, json_date, json_str};
use crate::error::KnowledgeBankError;
use crate::ingest::document::{FetchOutcome, SourceDocument, content_hash};
use crate::ingest::html::html_to_markdown;
use serde_json::{Value, json};

pub const SOURCE: &str = "jira";
const PAGE_SIZE: usize = 100;
const FIELDS: [&str; 11] = [
    "summary",
    "status",
    "issuetype",
    "priority",
    "assignee",
    "reporter",
    "created",
    "updated",
    "description",
    "labels",
    "comment",
];

pub async fn fetch(
    request: &FetchRequest<'_>,
    client: &AtlassianClient,
) -> Result<FetchOutcome, KnowledgeBankError> {
    let mut outcome = FetchOutcome {
        complete: request.last_sync.is_none(),
        ..FetchOutcome::default()
    };
    let jql = build_jql(
        &request.source.projects,
        request.source.jql.as_deref(),
        request.last_sync,
    );
    let mut next_token: Option<String> = None;
    loop {
        // JSON: protocol boundary — the search request body Jira expects.
        let mut body = json!({
            "jql": jql,
            "maxResults": PAGE_SIZE,
            "fields": FIELDS,
            "expand": "renderedFields",
        });
        if let Some(token) = &next_token {
            body["nextPageToken"] = json!(token);
        }
        let payload = client.post("/rest/api/3/search/jql", &body).await?;
        for issue in payload
            .get("issues")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if let Some(document) = issue_to_document(request, client, issue) {
                outcome.current_refs.push(document.source_ref.clone());
                outcome.documents.push(document);
            }
        }
        next_token = json_str(&payload, "/nextPageToken").map(str::to_owned);
        if next_token.is_none() || payload.get("isLast").and_then(Value::as_bool) == Some(true) {
            break;
        }
    }
    Ok(outcome)
}

#[must_use]
pub fn build_jql(
    projects: &[String],
    extra: Option<&str>,
    last_sync: Option<chrono::DateTime<chrono::Utc>>,
) -> String {
    let mut clauses = vec![format!(
        "project IN ({})",
        projects
            .iter()
            .map(|p| format!("\"{}\"", p.replace('"', "")))
            .collect::<Vec<_>>()
            .join(", ")
    )];
    if let Some(extra) = extra.map(str::trim).filter(|e| !e.is_empty()) {
        clauses.push(format!("({extra})"));
    }
    if let Some(last) = last_sync {
        clauses.push(format!("updated >= \"{}\"", last.format("%Y-%m-%d %H:%M")));
    }
    format!("{} ORDER BY updated ASC", clauses.join(" AND "))
}

fn issue_to_document(
    request: &FetchRequest<'_>,
    client: &AtlassianClient,
    issue: &Value,
) -> Option<SourceDocument> {
    let key = json_str(issue, "/key")?;
    let fields = issue.get("fields")?;
    let rendered = issue.get("renderedFields");
    let summary = json_str(fields, "/summary").unwrap_or("(no summary)");
    let title = format!("[{key}] {summary}");
    let updated = json_date(fields, "/updated");
    let body = build_document(
        client,
        &IssueView {
            key,
            title: &title,
            fields,
            rendered,
        },
        request.include_comments,
    );
    Some(SourceDocument {
        doc_id: SourceDocument::doc_id_for(request.source_type, key),
        source_type: request.source_type.to_owned(),
        source: SOURCE.to_owned(),
        source_ref: key.to_owned(),
        title,
        uri: format!("{}/browse/{key}", client.base_url()),
        content_hash: content_hash(&body),
        body,
        mtime: updated,
        authored_date: updated.map(|d| d.date_naive()),
    })
}

struct IssueView<'a> {
    key: &'a str,
    title: &'a str,
    fields: &'a Value,
    rendered: Option<&'a Value>,
}

fn build_document(
    client: &AtlassianClient,
    issue: &IssueView<'_>,
    include_comments: bool,
) -> String {
    let (key, title, fields, rendered) = (issue.key, issue.title, issue.fields, issue.rendered);
    let field = |pointer: &str| json_str(fields, pointer).unwrap_or("-").to_owned();
    let updated = json_date(fields, "/updated")
        .map_or_else(String::new, |d| d.format("%Y-%m-%d").to_string());
    let description = rendered
        .and_then(|r| json_str(r, "/description"))
        .map(html_to_markdown)
        .unwrap_or_default();
    let mut out = format!(
        "---\ntitle: \"{}\"\nsource_type: jira\nkey: {key}\nstatus: {}\nupdated: {updated}\nurl: {}/browse/{key}\n---\n\n# {title}\n\n**Status:** {} | **Type:** {} | **Priority:** {} | **Assignee:** {}\n\n## Description\n\n{description}\n",
        title.replace('"', "'"),
        field("/status/name"),
        client.base_url(),
        field("/status/name"),
        field("/issuetype/name"),
        field("/priority/name"),
        field("/assignee/displayName"),
    );
    if include_comments {
        let comments = rendered
            .and_then(|r| r.pointer("/comment/comments"))
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|c| {
                let text = html_to_markdown(json_str(c, "/body")?);
                let author = json_str(c, "/author/displayName").unwrap_or("unknown");
                let date = json_str(c, "/created")
                    .map(|d| d.chars().take(10).collect::<String>())
                    .unwrap_or_default();
                Some(format!("\n**{author}** ({date}):\n{text}\n"))
            })
            .collect::<String>();
        if !comments.is_empty() {
            out.push_str("\n## Comments\n");
            out.push_str(&comments);
        }
    }
    out
}
