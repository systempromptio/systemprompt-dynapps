//! The one place SSR page data is handed to the template engine.

use crate::error::AdminHtmlError;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::response::{Html, IntoResponse, Response};
use serde::Serialize;

use super::context::{BrandingShell, PageChrome, PageShell};

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
    render_shell(engine, template, data, Viewer::new(user_ctx, mkt_ctx))
}

pub(crate) fn render_shell<T: Serialize>(
    engine: &AdminTemplateEngine,
    template: &str,
    data: &T,
    viewer: Viewer<'_>,
) -> Response {
    let started = std::time::Instant::now();
    // JSON: the shell reads the page's own `page` key to pick its help text,
    // which needs the page context as data rather than as a type. This is the
    // only Value conversion on the SSR render path.
    let value = serde_json::to_value(data).unwrap_or_else(|e| {
        tracing::warn!(template, error = %e, "Failed to serialize SSR page data");
        serde_json::Value::Object(serde_json::Map::new())
    });
    let page_id = value.get("page").and_then(serde_json::Value::as_str);
    let shell = PageShell::new(engine.branding(), viewer, PageChrome { page_id }, &value);

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
