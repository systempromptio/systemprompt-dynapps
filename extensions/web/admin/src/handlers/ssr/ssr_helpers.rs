//! The one place SSR page data is handed to the template engine.

use crate::error::AdminHtmlError;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::response::{Html, IntoResponse, Response};
use serde::Serialize;

use super::context::{BrandingShell, PageChrome, PageShell};
use super::export::ViewMode;

pub(crate) const fn branding_context(engine: &AdminTemplateEngine) -> BrandingShell<'_> {
    BrandingShell {
        branding: engine.branding(),
    }
}

// Why: who is looking — the two per-request contexts every shell render
// reads, carried as one value so the render helpers stay inside the
// argument cap.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Viewer<'a> {
    pub(crate) user: &'a UserContext,
    pub(crate) marketplace: &'a MarketplaceContext,
}

impl<'a> Viewer<'a> {
    pub(crate) const fn new(user: &'a UserContext, marketplace: &'a MarketplaceContext) -> Self {
        Self { user, marketplace }
    }
}

pub(crate) fn render_typed_page<T: Serialize>(
    engine: &AdminTemplateEngine,
    template: &str,
    data: &T,
    user_ctx: &UserContext,
    mkt_ctx: &MarketplaceContext,
) -> Response {
    render_shell(engine, template, data, Viewer::new(user_ctx, mkt_ctx), None)
}

// Why: the exportable pages hand their `ViewMode` through so the shell can
// draw the download control; every other page renders with no such control.
pub(crate) fn render_shell<T: Serialize>(
    engine: &AdminTemplateEngine,
    template: &str,
    data: &T,
    viewer: Viewer<'_>,
    view: Option<&ViewMode>,
) -> Response {
    // Why: lint-ok: http-error — renders the page directly; the failure arm is
    // already the typed AdminHtmlError response.
    let started = std::time::Instant::now();
    // JSON: the shell reads the page's own `page` key to pick its help text,
    // which needs the page context as data rather than as a type. This is the
    // only Value conversion on the SSR render path.
    let value = serde_json::to_value(data).unwrap_or_else(|e| {
        tracing::warn!(template, error = %e, "Failed to serialize SSR page data");
        serde_json::Value::Object(serde_json::Map::new())
    });
    let page_id = value.get("page").and_then(serde_json::Value::as_str);
    let shell = PageShell::new(
        engine.branding(),
        viewer,
        PageChrome {
            page_id,
            export_url: view.map(ViewMode::export_url),
        },
        &value,
    );

    let response = match engine.render(template, &shell) {
        Ok(html) => Html(html).into_response(),
        Err(e) => AdminHtmlError::from(e).into_response(),
    };
    tracing::debug!(
        template,
        render_ms = started.elapsed().as_secs_f64() * 1000.0,
        "SSR rendered"
    );
    response
}
