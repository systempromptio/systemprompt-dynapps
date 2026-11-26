//! The collapsible tree of spaces and articles.
//!
//! It is drawn here as HTML, with every title and link escaped, because a
//! recursive Handlebars partial overflowed the stack in the renderer.

use serde::Serialize;

use crate::read::{Listed, Space};

/// A node of the tree. A space has no link; an article links to its page.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Node {
    pub title: String,
    pub href: Option<String>,
    pub selected: bool,
    pub open: bool,
    pub children: Vec<Self>,
}

pub fn build(spaces: &[Space], selected: Option<i64>) -> Vec<Node> {
    spaces
        .iter()
        .map(|space| Node {
            title: space.item.name.clone(),
            href: None,
            selected: false,
            open: true,
            children: branch(&space.articles, None, selected),
        })
        .collect()
}

fn branch(articles: &[Listed], parent: Option<i64>, selected: Option<i64>) -> Vec<Node> {
    articles
        .iter()
        .filter(|article| article.parent_id == parent)
        .map(|article| {
            let children = branch(articles, Some(article.article_id), selected);
            let here = selected == Some(article.article_id);
            Node {
                title: article.title.clone(),
                href: Some(format!("/admin/brain/articles/{}", article.article_id)),
                selected: here,
                open: selected.is_none() || here || children.iter().any(|child| child.open),
                children,
            }
        })
        .collect()
}

pub fn render(nodes: &[Node]) -> String {
    nodes.iter().map(item).collect()
}

fn item(node: &Node) -> String {
    let class = if node.selected { " is-selected" } else { "" };
    let label = label(node);
    if node.children.is_empty() {
        return format!("<li class=\"sp-brain-node{class}\">{label}</li>");
    }
    let open = if node.open { " open" } else { "" };
    let children = render(&node.children);
    format!(
        "<li class=\"sp-brain-node{class}\"><details{open}><summary>{label}</summary><ul class=\"sp-brain-tree\">{children}</ul></details></li>"
    )
}

fn label(node: &Node) -> String {
    let title = escape(&node.title);
    node.href.as_ref().map_or_else(
        || format!("<span class=\"sp-brain-space\">{title}</span>"),
        |href| format!("<a href=\"{}\">{title}</a>", escape(href)),
    )
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
