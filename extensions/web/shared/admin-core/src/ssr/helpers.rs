//! The one place SSR page data is handed to the template engine.

use crate::error::AdminHtmlError;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::response::{Html, IntoResponse, Response};
use serde::Serialize;

use super::context::{BrandingShell, PageChrome, PageShell};
use super::faults::Faults;

pub const fn branding_context(engine: &AdminTemplateEngine) -> BrandingShell<'_> {
    BrandingShell {
        branding: engine.branding(),
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Viewer<'a> {
    pub user: &'a UserContext,
    pub marketplace: &'a MarketplaceContext,
}

impl<'a> Viewer<'a> {
    pub const fn new(user: &'a UserContext, marketplace: &'a MarketplaceContext) -> Self {
        Self { user, marketplace }
    }
}

pub fn render_typed_page<T: Serialize>(
    engine: &AdminTemplateEngine,
    template: &str,
    data: &T,
    user_ctx: &UserContext,
    mkt_ctx: &MarketplaceContext,
) -> Response {
    render_shell(
        engine,
        template,
        data,
        Viewer::new(user_ctx, mkt_ctx),
        &Faults::new(),
    )
}

pub fn render_page_with_faults<T: Serialize>(
    engine: &AdminTemplateEngine,
    template: &str,
    data: &T,
    viewer: Viewer<'_>,
    faults: &Faults,
) -> Response {
    render_shell(engine, template, data, viewer, faults)
}

pub(crate) fn render_shell<T: Serialize>(
    engine: &AdminTemplateEngine,
    template: &str,
    data: &T,
    viewer: Viewer<'_>,
    faults: &Faults,
) -> Response {
    let started = std::time::Instant::now();
    // JSON: the shell reads the page's own `page` key to pick its help text,
    // which needs the page context as data rather than as a type. This is the
    // only Value conversion on the SSR render path.
    // Why: fail-soft-ok: a page context that cannot serialize is a programming
    // error, not a failed read; the empty shell still renders the error page.
    let value = serde_json::to_value(data).unwrap_or_else(|e| {
        tracing::warn!(template, error = %e, "Failed to serialize SSR page data");
        serde_json::Value::Object(serde_json::Map::new())
    });
    let page_id = value.get("page").and_then(serde_json::Value::as_str);
    let chrome = PageChrome {
        page_id,
        unavailable: faults.as_slice(),
    };
    let shell = PageShell::new(engine.branding(), viewer, chrome, &value);

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
