//! `/drive/{document_id}/live/frame`: a live page as its sandboxed frame
//! loads it, the stored HTML with the page runtime at the top of its head
//! (after the doctype, or first, when it has no head), so the runtime runs
//! before any script of the page.
//!
//! The runtime (`runtime-boot.js`, `runtime-core.js`, `runtime-mcp.js` and
//! `runtime-use.js` under `storage/files/js/workspace/live/`, compiled in
//! and joined into one script, each a complete function expression) is the
//! page's only way out: `window.governance.use`,
//! and its alias `window.claude`, post to the frame's parent, the viewer
//! shell. The response's policy keeps the page there: `sandbox
//! allow-scripts` without `allow-same-origin` makes it an opaque-origin
//! document that holds no session and reads no cookie, framed or opened on
//! its own; `connect-src 'none'` and `form-action 'none'` give it no
//! network; and the frame options let only this site frame it. Whether the
//! page's own scripts run, and which origins it loads styles, fonts and scripts
//! from, is the instance's rendered-HTML tier for the document
//! (`files.documents.renderedHtml`), as on `/rendered`; with the tier's
//! scripts off only the runtime runs, by its nonce, and the page stays
//! static. `?revision=` serves the version the shell loaded, so a publish
//! compares against what the viewer saw.

use axum::Extension;
use axum::body::Body;
use axum::extract::{Path, Query};
use axum::http::{HeaderValue, header};
use axum::response::Response;
use serde::Deserialize;
use systemprompt::extension::FrameOptionsOverride;
use systemprompt::files::documents::DocumentsError;
use systemprompt::identifiers::{DocumentRevisionId, TraceId};
use systemprompt::manifest::profile::FrameOptions;
use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::extension_boot::ExtensionBoot;
use systemprompt_web_admin_core::ssr::page::Page;

use super::live::live_document;
use super::user::{DriveDocuments, caller};
use crate::artifacts::manifest::CONTRACT_VERSION;
use crate::routes::drive_actor::site_origins;

const RUNTIME: [&str; 4] = [
    include_str!("../../../../../../../../storage/files/js/workspace/live/runtime-boot.js"),
    include_str!("../../../../../../../../storage/files/js/workspace/live/runtime-core.js"),
    include_str!("../../../../../../../../storage/files/js/workspace/live/runtime-mcp.js"),
    include_str!("../../../../../../../../storage/files/js/workspace/live/runtime-use.js"),
];

#[derive(Debug, Default, Deserialize)]
pub(crate) struct FrameQuery {
    #[serde(default)]
    revision: Option<String>,
}

pub(crate) async fn drive_live_frame(
    shell: Page,
    Extension(documents): Extension<DriveDocuments>,
    Extension(boot): Extension<ExtensionBoot>,
    Path(raw_id): Path<String>,
    Query(query): Query<FrameQuery>,
) -> AdminHtmlResult<Response> {
    let not_found = || AdminError::NotFound("Document not found.".to_owned());
    let (document_id, _) = live_document(&shell, &documents, raw_id).await?;
    let revision = query
        .revision
        .filter(|raw| !raw.is_empty())
        .map(DocumentRevisionId::try_new)
        .transpose()
        .map_err(|_invalid| not_found())?;
    let service = &documents.0;
    let (_, stored) = service
        .read_bytes(
            &caller(&shell.user),
            &document_id,
            revision.as_ref(),
            &TraceId::generate(),
        )
        .await
        .map_err(|error| match error {
            DocumentsError::Forbidden { .. }
            | DocumentsError::NotFound { .. }
            | DocumentsError::Trashed { .. } => not_found(),
            other => AdminError::internal(other),
        })?;
    let agent_content = service
        .agent_content(&document_id)
        .await
        .map_err(AdminError::internal)?;
    let tier = service.config().rendered_html.tier(agent_content);
    let origins: Vec<&str> = tier
        .remote_origins
        .iter()
        .map(systemprompt::files::RemoteOrigin::as_str)
        .collect();
    let nonce = uuid::Uuid::new_v4().simple().to_string();
    let runtime = runtime_tag(&nonce, &site_origins(&boot));
    let csp = HeaderValue::from_str(&policy(tier.scripts, &origins, &nonce))
        .map_err(AdminError::internal)?;
    Ok(framed(with_runtime(&stored, &runtime), csp))
}

fn runtime_tag(nonce: &str, origins: &[String]) -> String {
    let host = serde_json::json!({ "origins": origins, "contract": CONTRACT_VERSION })
        .to_string()
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026");
    let [boot, core, mcp, usage] = RUNTIME;
    format!("<script nonce=\"{nonce}\">({boot})({core}, {mcp}, {usage}, {host})</script>")
}

fn policy(scripts: bool, origins: &[&str], nonce: &str) -> String {
    let listed: String = origins.iter().flat_map(|origin| [" ", origin]).collect();
    let script_src = if scripts {
        format!("'unsafe-inline' 'unsafe-eval'{listed}")
    } else {
        format!("'nonce-{nonce}'")
    };
    format!(
        "sandbox allow-scripts; default-src 'none'; script-src {script_src}; style-src \
         'unsafe-inline'{listed}; img-src data: blob:{listed}; font-src data:{listed}; media-src \
         data: blob:{listed}; connect-src 'none'; form-action 'none'; base-uri 'none'"
    )
}

fn with_runtime(page: &[u8], runtime: &str) -> Vec<u8> {
    let at = tag_end(page, b"<head")
        .or_else(|| tag_end(page, b"<!doctype"))
        .unwrap_or(0);
    let mut out = Vec::with_capacity(page.len() + runtime.len());
    out.extend_from_slice(&page[..at]);
    out.extend_from_slice(runtime.as_bytes());
    out.extend_from_slice(&page[at..]);
    out
}

fn tag_end(page: &[u8], tag: &[u8]) -> Option<usize> {
    let start = page.windows(tag.len() + 1).position(|window| {
        window[..tag.len()].eq_ignore_ascii_case(tag)
            && matches!(window[tag.len()], b'>' | b' ' | b'\t' | b'\n' | b'\r')
    })?;
    page[start..]
        .iter()
        .position(|byte| *byte == b'>')
        .map(|offset| start + offset + 1)
}

fn framed(body: Vec<u8>, csp: HeaderValue) -> Response {
    let mut response = Response::new(Body::from(body));
    response
        .extensions_mut()
        .insert(FrameOptionsOverride(FrameOptions::SameOrigin));
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(header::CONTENT_SECURITY_POLICY, csp);
    response
}
