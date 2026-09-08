//! Jira connector: issues of the configured projects (narrowed by optional
//! JQL) with status header, description, and comments, laid out so the
//! `none` chunk strategy can split body from thread.
//!
//! Reads go through the connected Atlassian MCP as the caller. Incremental
//! syncs add `updated >= last_sync` to the JQL and therefore see only changed
//! issues, so they are never `complete`; a full sync reports every key and
//! is. Jira ran the search as the caller, so an issue behind a security
//! level the caller holds is indexed like any other and granted to the caller
//! alone.

use super::FetchRequest;
use super::atlassian::{AtlassianMcp, json_date, json_str, text_of};
use crate::error::KnowledgeBankError;
use crate::ingest::document::{FetchOutcome, SourceDocument, content_hash};
use serde_json::{Value, json};

pub const SOURCE: &str = "jira";
const PAGE_SIZE: usize = 100;
const SEARCH: &str = "searchJiraIssuesUsingJql";
const LIST_COMMENTS: &str = "listJiraIssueComments";
const FIELDS: [&str; 10] = [
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
];

pub async fn fetch(
    request: &FetchRequest<'_>,
    client: &AtlassianMcp,
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
        let mut args = json!({
            "cloudId": client.cloud_id(),
            "jql": jql,
            "maxResults": PAGE_SIZE,
            "fields": FIELDS,
            "view": "full",
            "responseContentFormat": "markdown",
        });
        if let Some(token) = &next_token {
            args["nextPageToken"] = json!(token);
        }
        let payload = client.call(SEARCH, args).await?;
        for issue in payload
            .get("issues")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if let Some(document) = issue_to_document(request, client, issue).await {
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

// Why: the search view may nest fields under `fields` (the REST shape) or
// flatten them (the compact view); both are read the same way.
fn field<'a>(issue: &'a Value, name: &str) -> Option<&'a Value> {
    issue
        .pointer(&format!("/fields/{name}"))
        .or_else(|| issue.get(name))
}

fn field_str<'a>(issue: &'a Value, name: &str, pointer: &str) -> Option<&'a str> {
    field(issue, name).and_then(|v| {
        if pointer.is_empty() {
            v.as_str()
        } else {
            v.pointer(pointer).and_then(Value::as_str)
        }
    })
}

async fn issue_to_document(
    request: &FetchRequest<'_>,
    client: &AtlassianMcp,
    issue: &Value,
) -> Option<SourceDocument> {
    let key = json_str(issue, "/key")?;
    let summary = field_str(issue, "summary", "").unwrap_or("(no summary)");
    let title = format!("[{key}] {summary}");
    let updated = field(issue, "updated")
        .and_then(Value::as_str)
        .and_then(|raw| json_date(&json!(raw), ""));
    let mut body = build_document(
        client.site_url(),
        &IssueView {
            key,
            title: &title,
            issue,
        },
    );
    if request.include_comments {
        body.push_str(&comments_section(client, key, issue).await);
    }
    Some(SourceDocument {
        doc_id: SourceDocument::doc_id_for(request.source_type, key),
        source_type: request.source_type.to_owned(),
        source: SOURCE.to_owned(),
        source_ref: key.to_owned(),
        title,
        uri: format!("{}/browse/{key}", client.site_url()),
        content_hash: content_hash(&body),
        body,
        mtime: updated,
        authored_date: updated.map(|d| d.date_naive()),
    })
}

struct IssueView<'a> {
    key: &'a str,
    title: &'a str,
    issue: &'a Value,
}

fn build_document(site: &str, view: &IssueView<'_>) -> String {
    let (key, title, issue) = (view.key, view.title, view.issue);
    let named = |name: &str| {
        field_str(issue, name, "/name")
            .or_else(|| field_str(issue, name, ""))
            .unwrap_or("-")
            .to_owned()
    };
    let assignee = field_str(issue, "assignee", "/displayName")
        .or_else(|| field_str(issue, "assignee", ""))
        .unwrap_or("-");
    let updated = field(issue, "updated")
        .and_then(Value::as_str)
        .and_then(|raw| json_date(&json!(raw), ""))
        .map_or_else(String::new, |d| d.format("%Y-%m-%d").to_string());
    let description = text_of(field(issue, "description"));
    let labels = field(issue, "labels")
        .and_then(Value::as_array)
        .map(|l| {
            l.iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        })
        .filter(|l| !l.is_empty())
        .map_or_else(String::new, |l| format!("\n**Labels:** {l}"));
    format!(
        "---\ntitle: \"{}\"\nsource_type: jira\nkey: {key}\nstatus: {}\nupdated: {updated}\nurl: {site}/browse/{key}\n---\n\n# {title}\n\n**Status:** {} | **Type:** {} | **Priority:** {} | **Assignee:** {assignee}{labels}\n\n## Description\n\n{}\n",
        title.replace('"', "'"),
        named("status"),
        named("status"),
        named("issuetype"),
        named("priority"),
        description.trim(),
    )
}

// Why: the search view carries the thread when the site returns it inline;
// otherwise it is one more read per issue. Either way a failure here loses
// the thread, not the issue.
async fn comments_section(client: &AtlassianMcp, key: &str, issue: &Value) -> String {
    let inline = field(issue, "comment")
        .and_then(|c| c.get("comments"))
        .and_then(Value::as_array)
        .cloned();
    let comments = match inline {
        Some(list) => list,
        None => match client
            .call(
                LIST_COMMENTS,
                json!({
                    "cloudId": client.cloud_id(),
                    "issueIdOrKey": key,
                    "maxResults": PAGE_SIZE,
                    "responseContentFormat": "markdown",
                }),
            )
            .await
        {
            Ok(payload) => payload
                .get("comments")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default(),
            Err(e) => {
                tracing::warn!(issue = key, error = %e, "jira comments could not be read; issue indexed without them");
                Vec::new()
            },
        },
    };
    let rendered = comments
        .iter()
        .filter_map(|c| {
            let text = text_of(c.get("body"));
            if text.trim().is_empty() {
                return None;
            }
            let author = json_str(c, "/author/displayName")
                .or_else(|| json_str(c, "/author"))
                .unwrap_or("unknown");
            let date = json_str(c, "/created")
                .map(|d| d.chars().take(10).collect::<String>())
                .unwrap_or_default();
            Some(format!("\n**{author}** ({date}):\n{}\n", text.trim()))
        })
        .collect::<String>();
    if rendered.is_empty() {
        String::new()
    } else {
        format!("\n## Comments\n{rendered}")
    }
}
