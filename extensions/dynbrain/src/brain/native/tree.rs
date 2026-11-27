//! The articles a person reaches, arranged as the tree they see.
//!
//! An article whose parent they cannot reach is shown at the top, so an
//! article granted on its own is still found.

use std::collections::{HashMap, HashSet};

use crate::access::Level;
use crate::ids::ArticleId;
use crate::model::Article;

pub(super) struct Tree<'a> {
    present: HashSet<ArticleId>,
    children: HashMap<Option<ArticleId>, Vec<&'a Article>>,
    max_depth: Option<u32>,
}

impl<'a> Tree<'a> {
    pub(super) fn new(reached: &'a [(Article, Level)], max_depth: Option<u32>) -> Self {
        let present: HashSet<ArticleId> = reached.iter().map(|(article, _)| article.id).collect();
        let mut children: HashMap<Option<ArticleId>, Vec<&'a Article>> = HashMap::new();
        for (article, _) in reached {
            let parent = article.parent.filter(|parent| present.contains(parent));
            children.entry(parent).or_default().push(article);
        }
        Self {
            present,
            children,
            max_depth,
        }
    }

    pub(super) fn has(&self, article: ArticleId) -> bool {
        self.present.contains(&article)
    }

    pub(super) fn child_count(&self, parent: ArticleId) -> usize {
        self.children.get(&Some(parent)).map_or(0, Vec::len)
    }

    pub(super) fn descend(
        &self,
        parent: Option<ArticleId>,
        skip: usize,
    ) -> Vec<(&'a Article, u32)> {
        let mut found = Vec::new();
        self.walk(parent, 1, skip, &mut found);
        found
    }

    fn walk(
        &self,
        parent: Option<ArticleId>,
        depth: u32,
        skip: usize,
        found: &mut Vec<(&'a Article, u32)>,
    ) {
        for &article in self.children.get(&parent).into_iter().flatten().skip(skip) {
            found.push((article, depth));
            if self.max_depth.is_none_or(|max| depth < max) {
                self.walk(Some(article.id), depth + 1, 0, found);
            }
        }
    }
}
