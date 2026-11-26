//! `/admin/brain/spaces`: every space with its articles as a collapsible tree.

use axum::Extension;
use axum::response::Response;
use serde::Serialize;
use systemprompt_web_admin_core::error::AdminHtmlResult;
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::ssr::helpers::{Viewer, render_page_with_faults};
use systemprompt_web_admin_core::ssr::page::Page;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;

use crate::read;
use crate::source::{Source, require_admin};
use crate::tree::{self, Node};
use crate::views::{NOT_MOUNTED, crumbs};

#[derive(Debug, Serialize)]
pub struct SpacesPage {
    pub page: &'static str,
    pub title: &'static str,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub unavailable: Option<&'static str>,
    pub tree: Vec<Node>,
}

pub async fn spaces_page(
    shell: Page,
    Extension(source): Extension<Source>,
) -> AdminHtmlResult<Response> {
    require_admin(&shell)?;
    let mut faults = Faults::new();
    let mut page = SpacesPage {
        page: "brain-spaces",
        title: "Spaces",
        breadcrumbs: crumbs("Spaces"),
        unavailable: None,
        tree: Vec::new(),
    };
    match source.console() {
        Some(console) => {
            let spaces = faults.take(
                "Spaces",
                read::read_spaces(&console, &shell.user.user_id).await,
            );
            page.tree = tree::build(&spaces, None);
        },
        None => page.unavailable = Some(NOT_MOUNTED),
    }
    Ok(render_page_with_faults(
        &shell.engine,
        "brain-spaces",
        &page,
        Viewer::new(&shell.user, &shell.marketplace),
        &faults,
    ))
}
