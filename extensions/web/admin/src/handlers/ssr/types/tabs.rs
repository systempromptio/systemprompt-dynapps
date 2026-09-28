//! The tab strip shared by paged admin views.
//!
//! Every tab is a link, so a view is bookmarkable and the server renders only
//! the active tab's body.

use serde::Serialize;

#[derive(Debug, Serialize)]
pub(crate) struct TabLinkView {
    pub slug: &'static str,
    pub label: &'static str,
    pub href: String,
    pub is_active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<i64>,
}
