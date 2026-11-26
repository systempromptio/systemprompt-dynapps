//! The collapsible tree of spaces and articles.

use serde::Serialize;

use crate::read::{Listed, Space};

/// A node of the tree. A space has no link; an article links to its page.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Node {
    pub title: String,
    pub href: Option<String>,
    pub selected: bool,
    pub open: bool,
    pub children: Vec<Node>,
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
