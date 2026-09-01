//! Confluence connector: every current page of the configured spaces, with
//! footer comments, converted to Markdown, plus each page's read
//! restrictions translated into grants.
//!
//! Incremental: a page whose version predates `last_sync` is not re-read for
//! content, but its restrictions ARE re-read, so a permission change upstream
//! still lands at the next sync (REQ-052 "permission changes propagate").
//! Every page id seen is reported so deletions prune.

use super::FetchRequest;
use super::atlassian::{AtlassianClient, json_date, json_str};
use crate::error::KnowledgeBankError;
use crate::ingest::document::{FetchOutcome, SourceDocument, UnchangedDocument, content_hash};
use crate::ingest::grants::RawRestrictions;
use crate::ingest::html::html_to_markdown;
// JSON: protocol boundary — see `sources::atlassian`.
use serde_json::Value;

pub const SOURCE: &str = "confluence";
const PAGE_SIZE: usize = 100;

pub async fn fetch(
    request: &FetchRequest<'_>,
    client: &AtlassianClient,
) -> Result<FetchOutcome, KnowledgeBankError> {
    let mut outcome = FetchOutcome {
        complete: true,
        ..FetchOutcome::default()
    };
    for space_key in &request.source.spaces {
        let space_id = resolve_space_id(client, space_key).await?;
        let mut next = Some(format!(
            "/wiki/api/v2/spaces/{space_id}/pages?status=current&body-format=storage&limit={PAGE_SIZE}"
        ));
        while let Some(path) = next.take() {
            let payload = client.get(&path).await?;
            for page in payload
                .get("results")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                ingest_page(request, client, space_key, page, &mut outcome).await?;
            }
            next = json_str(&payload, "/_links/next").map(str::to_owned);
        }
    }
    Ok(outcome)
}

async fn resolve_space_id(
    client: &AtlassianClient,
    key: &str,
) -> Result<String, KnowledgeBankError> {
    let payload = client
        .get(&format!("/wiki/api/v2/spaces?keys={key}&limit=1"))
        .await?;
    json_str(&payload, "/results/0/id")
        .map(str::to_owned)
        .ok_or_else(|| KnowledgeBankError::Backend(format!("confluence space '{key}' not found")))
}

async fn ingest_page(
    request: &FetchRequest<'_>,
    client: &AtlassianClient,
    space_key: &str,
    page: &Value,
    outcome: &mut FetchOutcome,
) -> Result<(), KnowledgeBankError> {
    let Some(id) = json_str(page, "/id") else {
        return Ok(());
    };
    outcome.current_refs.push(id.to_owned());
    let updated = json_date(page, "/version/createdAt");
    let doc_id = SourceDocument::doc_id_for(request.source_type, id);

    let restrictions = read_restrictions(client, id).await;
    let grants = restrictions.as_ref().map(|r| request.grants.resolve(r));

    if let (Some(last), Some(updated_at)) = (request.last_sync, updated)
        && updated_at <= last
    {
        outcome.unchanged.push(UnchangedDocument { doc_id, grants });
        return Ok(());
    }

    let title = json_str(page, "/title").unwrap_or("(untitled)");
    let body_html = json_str(page, "/body/storage/value").unwrap_or_default();
    let mut body = build_document(
        client,
        &PageMeta {
            space_key,
            id,
            title,
            updated,
        },
        body_html,
    );
    if request.include_comments {
        body.push_str(&fetch_comments(client, id).await?);
    }
    let web_path = json_str(page, "/_links/webui").unwrap_or_default();
    outcome.documents.push(SourceDocument {
        doc_id,
        source_type: request.source_type.to_owned(),
        source: SOURCE.to_owned(),
        source_ref: id.to_owned(),
        title: title.to_owned(),
        uri: format!("{}/wiki{web_path}", client.base_url()),
        content_hash: content_hash(&body),
        body,
        mtime: updated,
        authored_date: updated.map(|d| d.date_naive()),
        grants: grants.unwrap_or_default(),
    });
    Ok(())
}

struct PageMeta<'a> {
    space_key: &'a str,
    id: &'a str,
    title: &'a str,
    updated: Option<chrono::DateTime<chrono::Utc>>,
}

fn build_document(client: &AtlassianClient, page: &PageMeta<'_>, body_html: &str) -> String {
    let updated = page
        .updated
        .map_or_else(String::new, |d| d.format("%Y-%m-%d").to_string());
    let (space_key, id, title) = (page.space_key, page.id, page.title);
    format!(
        "---\ntitle: \"{}\"\nsource_type: confluence\npage_id: {id}\nspace: {space_key}\nupdated: {updated}\nurl: {}/wiki/spaces/{space_key}/pages/{id}\n---\n\n# {title}\n\n{}\n",
        title.replace('"', "'"),
        client.base_url(),
        html_to_markdown(body_html)
    )
}

async fn fetch_comments(client: &AtlassianClient, id: &str) -> Result<String, KnowledgeBankError> {
    let payload = client
        .get(&format!(
            "/wiki/api/v2/pages/{id}/footer-comments?body-format=storage&limit={PAGE_SIZE}"
        ))
        .await?;
    let mut out = String::new();
    for comment in payload
        .get("results")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let author = json_str(comment, "/version/authorId").unwrap_or("unknown");
        let date = json_date(comment, "/version/createdAt").map_or_else(
            || "0000-00-00".to_owned(),
            |d| d.format("%Y-%m-%d").to_string(),
        );
        let text = html_to_markdown(json_str(comment, "/body/storage/value").unwrap_or_default());
        if text.trim().is_empty() {
            continue;
        }
        out.push_str(&format!("\n**{author}** ({date}):\n{text}\n"));
    }
    if out.is_empty() {
        Ok(String::new())
    } else {
        Ok(format!("\n## Comments\n{out}"))
    }
}

// Why: a failed restriction read yields `None`, which the caller turns into
// NO grants — fail closed. Returning "no restrictions" here would grant the
// page to everyone signed in on the strength of a network error.
async fn read_restrictions(client: &AtlassianClient, id: &str) -> Option<RawRestrictions> {
    let payload = client
        .get(&format!(
            "/wiki/rest/api/content/{id}/restriction/byOperation/read?expand=restrictions.user,restrictions.group"
        ))
        .await;
    let payload = match payload {
        Ok(payload) => payload,
        Err(e) => {
            tracing::warn!(page = id, error = %e, "could not read page restrictions; failing closed");
            return None;
        },
    };
    Some(parse_restrictions(&payload))
}

#[must_use]
pub fn parse_restrictions(payload: &Value) -> RawRestrictions {
    let mut raw = RawRestrictions::default();
    for user in payload
        .pointer("/restrictions/user/results")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if let Some(account) = json_str(user, "/accountId") {
            raw.account_ids.push(account.to_owned());
        }
        if let Some(email) = json_str(user, "/email") {
            raw.emails.push(email.to_owned());
        }
    }
    for group in payload
        .pointer("/restrictions/group/results")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if let Some(name) = json_str(group, "/name") {
            raw.group_names.push(name.to_owned());
        }
    }
    raw
}
