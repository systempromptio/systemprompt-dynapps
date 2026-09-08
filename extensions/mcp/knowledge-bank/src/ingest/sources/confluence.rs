//! Confluence connector: every current page of the configured spaces the
//! caller can read, with footer comments, as Markdown.
//!
//! Reads go through the connected Atlassian MCP as the caller, so what comes
//! back is what the caller may read, and the ingestor grants exactly that to
//! the caller. Incremental: a page whose version predates the caller's cursor
//! and that the index already holds is not re-chunked. Every page id seen is
//! reported so the caller's grants on deleted pages are dropped. A page that
//! fails to read is logged and skipped, never fatal to the space.

use super::FetchRequest;
use super::atlassian::{AtlassianMcp, json_date, json_str, text_of};
use crate::error::KnowledgeBankError;
use crate::ingest::document::{FetchOutcome, SourceDocument, content_hash};
use crate::ingest::html::html_to_markdown;
use serde_json::{Value, json};

pub const SOURCE: &str = "confluence";
const PAGE_SIZE: usize = 250;
const LIST_SPACES: &str = "listConfluenceSpaces";
const LIST_CONTENT: &str = "listConfluenceContent";
const GET_CONTENT: &str = "getConfluenceContent";
const LIST_COMMENTS: &str = "listConfluenceComments";

pub async fn fetch(
    request: &FetchRequest<'_>,
    client: &AtlassianMcp,
) -> Result<FetchOutcome, KnowledgeBankError> {
    let mut outcome = FetchOutcome {
        complete: true,
        ..FetchOutcome::default()
    };
    for space_key in &request.source.spaces {
        let space_id = resolve_space_id(client, space_key).await?;
        let mut cursor: Option<String> = None;
        loop {
            let mut args = json!({
                "cloudId": client.cloud_id(),
                "type": "page",
                "spaceId": space_id,
                "status": "current",
                "limit": PAGE_SIZE,
            });
            if let Some(c) = &cursor {
                args["cursor"] = json!(c);
            }
            let payload = client.call(LIST_CONTENT, args).await?;
            for page in payload
                .get("results")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                ingest_page(request, client, space_key, page, &mut outcome).await;
            }
            cursor = next_cursor(&payload);
            if cursor.is_none() {
                break;
            }
        }
    }
    Ok(outcome)
}

async fn resolve_space_id(client: &AtlassianMcp, key: &str) -> Result<String, KnowledgeBankError> {
    let payload = client
        .call(
            LIST_SPACES,
            json!({ "cloudId": client.cloud_id(), "keys": [key] }),
        )
        .await?;
    payload
        .get("results")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|space| json_str(space, "/key").is_none_or(|k| k.eq_ignore_ascii_case(key)))
        .and_then(|space| json_str(space, "/id"))
        .map(str::to_owned)
        .ok_or_else(|| {
            KnowledgeBankError::Backend(format!(
                "confluence space '{key}' not found or not readable with your connected account"
            ))
        })
}

// Why: the listing carries no timestamp, so change detection needs the page
// itself; one full read per page is one call either way, and the first sync
// is the one that reads everything.
async fn ingest_page(
    request: &FetchRequest<'_>,
    client: &AtlassianMcp,
    space_key: &str,
    listed: &Value,
    outcome: &mut FetchOutcome,
) {
    let Some(id) = json_str(listed, "/id") else {
        return;
    };
    outcome.current_refs.push(id.to_owned());
    let doc_id = SourceDocument::doc_id_for(request.source_type, id);
    let content = match client
        .call(
            GET_CONTENT,
            json!({
                "cloudId": client.cloud_id(),
                "content_id": id,
                "detail": "full",
                "content_format": "markdown",
                "include_metadata": true,
            }),
        )
        .await
    {
        Ok(content) => content,
        Err(e) => {
            tracing::warn!(page_id = id, error = %e, "confluence page could not be read; skipped");
            return;
        },
    };
    let updated = json_date(&content, "/metadata/version/createdAt");
    if request.can_skip(&doc_id, updated) {
        outcome.unchanged.push(doc_id);
        return;
    }
    let title = json_str(&content, "/title")
        .or_else(|| json_str(listed, "/title"))
        .unwrap_or("(untitled)");
    let body_text = match json_str(&content, "/body/format") {
        Some("markdown") => text_of(content.pointer("/body/value")),
        _ => html_to_markdown(json_str(&content, "/body/value").unwrap_or_default()),
    };
    let site = client.site_url();
    let mut body = build_document(
        &PageMeta {
            site,
            space_key,
            id,
            title,
            updated,
        },
        &body_text,
    );
    if request.include_comments {
        match fetch_comments(client, id).await {
            Ok(comments) => body.push_str(&comments),
            Err(e) => {
                tracing::warn!(page_id = id, error = %e, "confluence comments could not be read; page indexed without them");
            },
        }
    }
    outcome.documents.push(SourceDocument {
        doc_id,
        source_type: request.source_type.to_owned(),
        source: SOURCE.to_owned(),
        source_ref: id.to_owned(),
        title: title.to_owned(),
        uri: format!("{site}/wiki/spaces/{space_key}/pages/{id}"),
        content_hash: content_hash(&body),
        body,
        mtime: updated,
        authored_date: updated.map(|d| d.date_naive()),
    });
}

struct PageMeta<'a> {
    site: &'a str,
    space_key: &'a str,
    id: &'a str,
    title: &'a str,
    updated: Option<chrono::DateTime<chrono::Utc>>,
}

fn build_document(page: &PageMeta<'_>, body_markdown: &str) -> String {
    let updated = page
        .updated
        .map_or_else(String::new, |d| d.format("%Y-%m-%d").to_string());
    let (site, space_key, id, title) = (page.site, page.space_key, page.id, page.title);
    format!(
        "---\ntitle: \"{}\"\nsource_type: confluence\npage_id: {id}\nspace: {space_key}\nupdated: {updated}\nurl: {site}/wiki/spaces/{space_key}/pages/{id}\n---\n\n# {title}\n\n{}\n",
        title.replace('"', "'"),
        body_markdown.trim()
    )
}

async fn fetch_comments(client: &AtlassianMcp, id: &str) -> Result<String, KnowledgeBankError> {
    let payload = client
        .call(
            LIST_COMMENTS,
            json!({
                "cloudId": client.cloud_id(),
                "content-id": id,
                "content-type": "page",
                "comment-type": "footer",
                "body-format": "markdown",
                "include-replies": true,
                "limit": PAGE_SIZE,
            }),
        )
        .await?;
    let mut out = String::new();
    for comment in payload
        .get("comments")
        .or_else(|| payload.get("results"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        render_comment(comment, &mut out);
        for reply in comment
            .get("replies")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            render_comment(reply, &mut out);
        }
    }
    if out.is_empty() {
        Ok(String::new())
    } else {
        Ok(format!("\n## Comments\n{out}"))
    }
}

fn render_comment(comment: &Value, out: &mut String) {
    let author = json_str(comment, "/authorAccountId")
        .or_else(|| json_str(comment, "/author/displayName"))
        .unwrap_or("unknown");
    let date = json_date(comment, "/createdAt")
        .or_else(|| json_date(comment, "/version/createdAt"))
        .map_or_else(
            || "0000-00-00".to_owned(),
            |d| d.format("%Y-%m-%d").to_string(),
        );
    let text = text_of(comment.get("body"));
    if text.trim().is_empty() {
        return;
    }
    out.push_str(&format!("\n**{author}** ({date}):\n{}\n", text.trim()));
}

// Why: the hosted server names its cursor three ways depending on the tool
// version; any of them continues the listing.
fn next_cursor(payload: &Value) -> Option<String> {
    json_str(payload, "/nextCursor")
        .or_else(|| json_str(payload, "/cursor"))
        .map(str::to_owned)
        .or_else(|| {
            json_str(payload, "/_links/next")
                .and_then(|next| next.split("cursor=").nth(1))
                .map(|c| c.split('&').next().unwrap_or(c).to_owned())
        })
        .filter(|c| !c.is_empty())
}
