//! The writing operations: create, create a whole tree, update, move, trash,
//! grants and the inheritance switch. Each checks the access it needs before
//! it writes anything, and checks the content against the space before it
//! creates the first article, so a refused request leaves nothing behind.

use std::collections::VecDeque;

use systemprompt::identifiers::UserId;

use super::NativeBrain;
use super::shape::{Change, checked, compose, lang_of, require, variant_info};
use crate::access::Level;
use crate::brain::Reply;
use crate::brain::args::{CreateArgs, MoveArgs, NewNode, TreeArgs, UpdateArgs};
use crate::brain::fault::BrainFault;
use crate::brain::views::{
    Created, CreatedNode, Moved, Trashed, TreeCreated, Updated, VariantInfo,
};
use crate::ids::{ArticleId, SpaceId};
use crate::model::{Article, ArticleMove, NewArticle, RagMode, Space, VariantWrite};

const MAX_TREE_NODES: usize = 200;

struct Draft<'a> {
    space: &'a Space,
    parent: Option<ArticleId>,
    position: Option<i32>,
    lang: &'a str,
    write: VariantWrite,
}

struct Made {
    title: String,
    article_id: ArticleId,
    path: String,
    parent: Option<usize>,
}

impl NativeBrain {
    pub(super) async fn make_article(&self, user: &UserId, args: &CreateArgs) -> Reply<Created> {
        let (space, parent) = self.container(user, args.parent_id, args.space).await?;
        let lang = lang_of(args.lang.as_deref(), &space);
        let write = compose(
            &Change {
                title: Some(&args.title),
                markdown: args.markdown.as_deref(),
                front_matter: args.front_matter.as_ref(),
                metadata: args.metadata.as_ref(),
            },
            None,
        );
        checked(&space, &lang, &write)?;
        let draft = Draft {
            space: &space,
            parent,
            position: self.place(space.id, parent, args.position).await?,
            lang: &lang,
            write,
        };
        let article = self.insert_one(user, &draft).await?;
        Ok(Created {
            article_id: article.id,
            space: space.id,
            parent_id: parent,
            title: draft.write.title,
            path: self.path_for(&space, &article).await?,
            variant: VariantInfo {
                is_default: lang == space.default_lang,
                lang,
            },
        })
    }

    pub(super) async fn make_tree(&self, user: &UserId, args: &TreeArgs) -> Reply<TreeCreated> {
        let (space, parent) = self.container(user, args.parent_id, args.space).await?;
        let lang = lang_of(args.lang.as_deref(), &space);
        if args.nodes.is_empty() || count(&args.nodes) > MAX_TREE_NODES {
            return Err(BrainFault::Invalid(
                "a tree needs between 1 and 200 nodes per call",
            ));
        }
        validate(&space, &lang, &args.nodes)?;
        let base = self.base_path(user, &space, parent).await?;
        let mut made: Vec<Made> = Vec::new();
        let mut queue: VecDeque<(&NewNode, Option<usize>)> =
            args.nodes.iter().map(|node| (node, None)).collect();
        while let Some((node, up)) = queue.pop_front() {
            let (parent_id, parent_path) = up.map_or_else(
                || (parent, base.clone()),
                |index| (Some(made[index].article_id), made[index].path.clone()),
            );
            let draft = Draft {
                space: &space,
                parent: parent_id,
                position: None,
                lang: &lang,
                write: compose(&node_change(node), None),
            };
            let article = self.insert_one(user, &draft).await?;
            made.push(Made {
                title: node.title.clone(),
                article_id: article.id,
                path: format!("{parent_path}/{}", article.slug),
                parent: up,
            });
            let index = made.len() - 1;
            queue.extend(node.children.iter().map(|child| (child, Some(index))));
        }
        Ok(TreeCreated {
            parent_id: parent,
            created: made.len(),
            nodes: nest(&made, None),
        })
    }

    pub(super) async fn change_article(&self, user: &UserId, args: &UpdateArgs) -> Reply<Updated> {
        if args.title.is_none()
            && args.markdown.is_none()
            && args.front_matter.is_none()
            && args.metadata.is_none()
        {
            return Err(BrainFault::Invalid(
                "pass a title, markdown, front_matter or metadata",
            ));
        }
        let (article, level) = self.reach(args.article_id, user).await?;
        require(level, Level::Write)?;
        let space = self.space_row(article.space).await?;
        let lang = lang_of(args.lang.as_deref(), &space);
        let base = self.repo.find_variant(article.id, &lang).await?;
        let write = compose(
            &Change {
                title: args.title.as_deref(),
                markdown: args.markdown.as_deref(),
                front_matter: args.front_matter.as_ref(),
                metadata: args.metadata.as_ref(),
            },
            base.as_ref(),
        );
        checked(&space, &lang, &write)?;
        let variant = self
            .repo
            .write_variant(article.id, &lang, &write, user)
            .await?;
        Ok(Updated {
            article_id: article.id,
            variant: variant_info(&variant, &space),
            title: variant.title,
            applied: true,
            revision: variant.revision,
        })
    }

    pub(super) async fn relocate(&self, user: &UserId, args: &MoveArgs) -> Reply<Moved> {
        let (article, level) = self.reach(args.article_id, user).await?;
        require(level, Level::Write)?;
        let (space, parent) = self
            .container(user, args.new_parent_id, Some(article.space))
            .await?;
        let position = self.place(space.id, parent, args.position).await?;
        self.repo
            .move_article(article.id, ArticleMove { parent, position })
            .await?;
        Ok(Moved {
            article_id: article.id,
            new_parent_id: parent,
            applied: true,
        })
    }

    pub(super) async fn discard(&self, user: &UserId, article: ArticleId) -> Reply<Trashed> {
        let (found, level) = self.reach(article, user).await?;
        require(level, Level::Write)?;
        self.repo.trash_article(found.id).await?;
        Ok(Trashed {
            article_id: found.id,
            applied: true,
        })
    }

    async fn container(
        &self,
        user: &UserId,
        parent: Option<ArticleId>,
        space: Option<SpaceId>,
    ) -> Result<(Space, Option<ArticleId>), BrainFault> {
        if let Some(parent) = parent {
            let (article, level) = self.reach(parent, user).await?;
            require(level, Level::Write)?;
            return Ok((self.space_row(article.space).await?, Some(article.id)));
        }
        let id = space.ok_or(BrainFault::Invalid("pass parent_id or space"))?;
        let (found, level) = self.space_reach(id, user).await?;
        require(level, Level::Write)?;
        Ok((found, None))
    }

    async fn base_path(
        &self,
        user: &UserId,
        space: &Space,
        parent: Option<ArticleId>,
    ) -> Result<String, BrainFault> {
        let Some(id) = parent else {
            return Ok(space.slug.clone());
        };
        let (article, _) = self.reach(id, user).await?;
        self.path_for(space, &article).await
    }

    async fn insert_one(&self, user: &UserId, draft: &Draft<'_>) -> Result<Article, BrainFault> {
        let slug = self
            .unique_slug(draft.space.id, draft.parent, &draft.write.title)
            .await?;
        let article = self
            .repo
            .insert_article(
                &NewArticle {
                    space: draft.space.id,
                    parent: draft.parent,
                    position: draft.position,
                    slug,
                    icon: None,
                    rag_mode: RagMode::Default,
                },
                user,
            )
            .await?;
        self.repo
            .write_variant(article.id, draft.lang, &draft.write, user)
            .await?;
        Ok(article)
    }
}

fn node_change(node: &NewNode) -> Change<'_> {
    Change {
        title: Some(&node.title),
        markdown: node.markdown.as_deref(),
        front_matter: node.front_matter.as_ref(),
        metadata: node.metadata.as_ref(),
    }
}

fn count(nodes: &[NewNode]) -> usize {
    nodes.iter().map(|node| 1 + count(&node.children)).sum()
}

fn validate(space: &Space, lang: &str, nodes: &[NewNode]) -> Result<(), BrainFault> {
    for node in nodes {
        checked(space, lang, &compose(&node_change(node), None))?;
        validate(space, lang, &node.children)?;
    }
    Ok(())
}

fn nest(made: &[Made], parent: Option<usize>) -> Vec<CreatedNode> {
    made.iter()
        .enumerate()
        .filter(|(_, row)| row.parent == parent)
        .map(|(index, row)| CreatedNode {
            title: row.title.clone(),
            article_id: row.article_id,
            path: row.path.clone(),
            children: nest(made, Some(index)),
        })
        .collect()
}
