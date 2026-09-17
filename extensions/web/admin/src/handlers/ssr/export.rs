//! `?format=json` — every AI-activity page as a downloadable JSON file.
//!
//! An operator debugging a number on a page ("18M tokens in five turns?")
//! needs the rows behind it, not a screenshot. Rather than a bespoke export
//! per page, the page's own typed context — the exact data the template
//! rendered, raw counts beside the formatted strings — is serialised and sent
//! as an attachment. The same handler serves both: [`ViewMode`] is an
//! extractor that reads the query, and [`ViewMode::respond`] picks the
//! representation. A page opts in by taking the extractor, so the export can
//! never drift from what the page shows.

use axum::extract::{FromRequestParts, OriginalUri};
use axum::http::request::Parts;
use axum::http::{Uri, header};
use axum::response::Response;
use serde::Serialize;
use std::convert::Infallible;
use std::future::{Future, ready};

use super::ssr_helpers::Viewer;
use crate::templates::AdminTemplateEngine;

const FORMAT_PARAM: &str = "format";
const FORMAT_JSON: &str = "json";
const SCHEMA: &str = "systemprompt.admin.page-export/1";

// Why: the shell needs the URL the button links to; the handler needs to know
// whether it was followed. Both come from the request URI, so one extractor
// owns both and no handler builds a URL by hand.
#[derive(Debug, Clone)]
pub(crate) struct ViewMode {
    export: bool,
    path: String,
    export_url: String,
}

impl<S: Send + Sync> FromRequestParts<S> for ViewMode {
    type Rejection = Infallible;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl Future<Output = Result<Self, Infallible>> + Send {
        // Why: `nest_service` strips its prefix from `parts.uri`, so the link
        // must come from `OriginalUri` or it points at `/sessions/…` instead
        // of `/admin/sessions/…`.
        let uri = parts
            .extensions
            .get::<OriginalUri>()
            .map_or_else(|| parts.uri.clone(), |o| o.0.clone());
        ready(Ok(Self::from_uri(&uri)))
    }
}

impl ViewMode {
    fn from_uri(uri: &Uri) -> Self {
        let query = uri.query().unwrap_or_default();
        let kept: Vec<&str> = query
            .split('&')
            .filter(|pair| !pair.is_empty() && !is_format_pair(pair))
            .collect();
        let export = query.split('&').any(|pair| pair == "format=json");
        let mut export_url = format!("{}?", uri.path());
        for pair in &kept {
            export_url.push_str(pair);
            export_url.push('&');
        }
        export_url.push_str(FORMAT_PARAM);
        export_url.push('=');
        export_url.push_str(FORMAT_JSON);
        Self {
            export,
            path: uri.path().to_owned(),
            export_url,
        }
    }

    pub(crate) fn export_url(&self) -> &str {
        &self.export_url
    }

    pub(crate) fn respond<T: Serialize>(
        &self,
        engine: &AdminTemplateEngine,
        template: &str,
        data: &T,
        viewer: Viewer<'_>,
    ) -> Response {
        if self.export {
            return self.download(template, data);
        }
        super::ssr_helpers::render_shell(engine, template, data, viewer, Some(self))
    }

    fn download<T: Serialize>(&self, page: &str, data: &T) -> Response {
        let exported_at = chrono::Utc::now();
        let envelope = Envelope {
            schema: SCHEMA,
            page,
            path: &self.path,
            exported_at: exported_at.to_rfc3339(),
            app_version: env!("CARGO_PKG_VERSION"),
            data,
        };
        let body = serde_json::to_vec_pretty(&envelope).unwrap_or_else(|e| {
            tracing::warn!(page, error = %e, "Failed to serialize page export");
            Vec::new()
        });
        Response::builder()
            .header(header::CONTENT_TYPE, "application/json; charset=utf-8")
            .header(
                header::CONTENT_DISPOSITION,
                format!(
                    "attachment; filename=\"{}\"",
                    filename(page, &self.path, exported_at)
                ),
            )
            .body(body.into())
            .unwrap_or_default()
    }
}

fn is_format_pair(pair: &str) -> bool {
    pair.split_once('=')
        .is_some_and(|(key, _)| key == FORMAT_PARAM)
}

// Why: the file is named for the page and the entity it shows, so a folder
// of exports stays legible —
// `session-detail-sess_8e686cd-20260916T094800Z.json` rather than a row of
// `export.json`s.
fn filename(page: &str, path: &str, at: chrono::DateTime<chrono::Utc>) -> String {
    let entity = path
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or_default();
    let entity: String = entity
        .chars()
        .take(24)
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    let stamp = at.format("%Y%m%dT%H%M%SZ");
    if entity.is_empty() || entity == page {
        format!("{page}-{stamp}.json")
    } else {
        format!("{page}-{entity}-{stamp}.json")
    }
}

#[derive(Debug, Serialize)]
struct Envelope<'a, T> {
    schema: &'static str,
    page: &'a str,
    path: &'a str,
    exported_at: String,
    app_version: &'static str,
    data: &'a T,
}
