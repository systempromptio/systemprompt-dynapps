//! The outline and the Markdown export: the tree a person sees, bounded.

use systemprompt::identifiers::UserId;

use super::NativeBrain;
use super::shape::path_of;
use super::tree::Tree;
use crate::brain::Reply;
use crate::brain::args::{ExportArgs, OutlineArgs};
use crate::brain::fault::BrainFault;
use crate::brain::views::{Anchor, Export, Outline, OutlineNode};
use crate::model::{Article, Space};

impl NativeBrain {
    pub(super) async fn fetch_outline(&self, user: &UserId, args: &OutlineArgs) -> Reply<Outline> {
        if args.mode.as_deref().is_some_and(|mode| mode != "descend") {
            return Err(BrainFault::Unavailable(
                "only the `descend` outline mode is available on this instance",
            ));
        }
        let max_depth = match args.depth {
            None => Some(2),
            Some(-1) => None,
            Some(depth) if depth >= 1 => Some(depth.unsigned_abs()),
            Some(_) => {
                return Err(BrainFault::Invalid(
                    "depth must be at least 1, or -1 for the whole subtree",
                ));
            },
        };
        let (space, node, anchor) = self.anchor(user, args).await?;
        let reached = self.readable(space.id, user).await?;
        let tree = Tree::new(&reached, max_depth);
        let offset = args.offset.unwrap_or(0) as usize;
        let walked = tree.descend(node.as_ref().map(|article| article.id), offset);
        let total = walked.len();
        let cap = args.max_nodes.unwrap_or(200).clamp(1, 1000) as usize;
        let mut nodes = Vec::new();
        for (article, depth) in walked.into_iter().take(cap) {
            let variant = self
                .variant_of(article, &space, args.lang.as_deref())
                .await?;
            let child_count = tree.child_count(article.id);
            nodes.push(OutlineNode {
                id: article.id,
                title: variant.title,
                lang: variant.lang,
                depth,
                parent_id: article.parent.filter(|parent| tree.has(*parent)),
                has_children: child_count > 0,
                child_count,
            });
        }
        let truncated = total > nodes.len();
        let hint = truncated.then(|| {
            format!(
                "{total} nodes match and {} are shown: pass `node` with one of them to narrow, or `offset` to page",
                nodes.len()
            )
        });
        Ok(Outline {
            anchor,
            shown: nodes.len(),
            nodes,
            total,
            truncated,
            has_more: truncated,
            offset,
            hint,
        })
    }

    async fn anchor(
        &self,
        user: &UserId,
        args: &OutlineArgs,
    ) -> Result<(Space, Option<Article>, Anchor), BrainFault> {
        if let Some(node) = args.node {
            let (article, _) = self.reach(node, user).await?;
            let space = self.space_row(article.space).await?;
            let name = self
                .variant_of(&article, &space, args.lang.as_deref())
                .await?
                .title;
            let anchor = Anchor { id: node.0, name };
            return Ok((space, Some(article), anchor));
        }
        let id = args
            .space
            .ok_or(BrainFault::Invalid("pass node or space"))?;
        let (space, _) = self.space_reach(id, user).await?;
        let anchor = Anchor {
            id: space.id.0,
            name: space.name.clone(),
        };
        Ok((space, None, anchor))
    }

    pub(super) async fn fetch_export(&self, user: &UserId, args: &ExportArgs) -> Reply<Export> {
        let format = args.format.as_deref().unwrap_or("full");
        if !matches!(format, "full" | "index") {
            return Err(BrainFault::Invalid("format must be `full` or `index`"));
        }
        let (space, _) = self.space_reach(args.space, user).await?;
        let reached = self.readable(space.id, user).await?;
        let all = self.repo.list_articles(space.id).await?;
        let walked = Tree::new(&reached, None).descend(None, 0);
        let mut markdown = String::new();
        for (article, depth) in &walked {
            let variant = self
                .variant_of(article, &space, args.lang.as_deref())
                .await?;
            if format == "index" {
                let indent = "  ".repeat(*depth as usize - 1);
                let path = path_of(&space, article, &all);
                markdown.push_str(&format!("{indent}- {} (`{path}`)\n", variant.title));
            } else {
                let marks = "#".repeat((*depth as usize).min(6));
                markdown.push_str(&format!(
                    "{marks} {}\n\n{}\n\n",
                    variant.title, variant.body
                ));
            }
        }
        Ok(Export {
            space: space.id,
            format: format.to_owned(),
            mimetype: "text/markdown",
            size_bytes: markdown.len(),
            markdown,
            article_count: walked.len(),
        })
    }
}
