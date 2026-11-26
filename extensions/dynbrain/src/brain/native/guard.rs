//! The access checks and the small helpers every operation shares: reaching
//! an article or a space as a person, choosing the variant to show, building
//! a path and a slug, and merging a change into a variant.
//!
//! What a person reads in a space is one query (`list_article_levels`), not
//! one per article, so lists, the outline and the export cost the same
//! whatever the size of the tree.

use std::collections::{BTreeSet, HashMap, HashSet};

use systemprompt::identifiers::UserId;

use super::NativeBrain;
use super::shape::{blank, path_of, pick, slugify};
use crate::access::Level;
use crate::brain::args::Position;
use crate::brain::fault::BrainFault;
use crate::ids::{ArticleId, SpaceId};
use crate::model::{Article, Space, Variant};

impl NativeBrain {
    pub(super) async fn reach(
        &self,
        article: ArticleId,
        user: &UserId,
    ) -> Result<(Article, Level), BrainFault> {
        let found = self
            .repo
            .find_article(article)
            .await?
            .filter(|row| row.trashed_at.is_none())
            .ok_or(BrainFault::NotFound)?;
        let level = self
            .repo
            .find_level(article, user)
            .await?
            .filter(|level| *level >= Level::Read)
            .ok_or(BrainFault::NotFound)?;
        Ok((found, level))
    }

    pub(super) async fn visible_parent(
        &self,
        user: &UserId,
        article: &Article,
    ) -> Result<Option<ArticleId>, BrainFault> {
        let Some(parent) = article.parent else {
            return Ok(None);
        };
        match self.reach(parent, user).await {
            Ok((found, _)) => Ok(Some(found.id)),
            Err(BrainFault::NotFound) => Ok(None),
            Err(other) => Err(other),
        }
    }

    pub(super) async fn space_row(&self, space: SpaceId) -> Result<Space, BrainFault> {
        self.repo
            .find_space(space)
            .await?
            .ok_or(BrainFault::NotFound)
    }

    pub(super) async fn space_reach(
        &self,
        space: SpaceId,
        user: &UserId,
    ) -> Result<(Space, Level), BrainFault> {
        let found = self.space_row(space).await?;
        let own = self
            .repo
            .find_space_level(space, user)
            .await?
            .unwrap_or(Level::NoAccess);
        if own >= Level::Read || !self.readable(space, user).await?.is_empty() {
            return Ok((found, own));
        }
        Err(BrainFault::NotFound)
    }

    pub(super) async fn readable(
        &self,
        space: SpaceId,
        user: &UserId,
    ) -> Result<Vec<(Article, Level)>, BrainFault> {
        let levels: HashMap<ArticleId, Level> = self
            .repo
            .list_article_levels(user, Some(space))
            .await?
            .into_iter()
            .map(|reached| (reached.article, reached.level))
            .collect();
        Ok(self
            .repo
            .list_articles(space)
            .await?
            .into_iter()
            .filter(|article| article.trashed_at.is_none())
            .filter_map(|article| {
                levels
                    .get(&article.id)
                    .copied()
                    .map(|level| (article, level))
            })
            .collect())
    }

    pub(super) async fn seen(
        &self,
        space: SpaceId,
        user: &UserId,
    ) -> Result<BTreeSet<ArticleId>, BrainFault> {
        Ok(self
            .repo
            .list_article_levels(user, Some(space))
            .await?
            .into_iter()
            .map(|reached| reached.article)
            .collect())
    }

    pub(super) async fn variant_of(
        &self,
        article: &Article,
        space: &Space,
        lang: Option<&str>,
    ) -> Result<Variant, BrainFault> {
        let variants = self.repo.list_variants(article.id).await?;
        Ok(pick(variants, lang, &space.default_lang).unwrap_or_else(|| blank(article, space)))
    }

    pub(super) async fn path_for(
        &self,
        space: &Space,
        article: &Article,
        user: &UserId,
    ) -> Result<String, BrainFault> {
        let all = self.repo.list_articles(space.id).await?;
        let seen = self.seen(space.id, user).await?;
        Ok(path_of(space, article, &all, &seen))
    }

    pub(super) async fn resolve(
        &self,
        id: Option<ArticleId>,
        path: Option<&str>,
    ) -> Result<ArticleId, BrainFault> {
        match (id, path) {
            (Some(id), None) => Ok(id),
            (None, Some(path)) => self.by_path(path).await,
            _ => Err(BrainFault::Invalid(
                "pass exactly one of article_id and path",
            )),
        }
    }

    async fn by_path(&self, path: &str) -> Result<ArticleId, BrainFault> {
        let mut segments = path.trim_matches('/').split('/');
        let slug = segments.next().unwrap_or_default();
        let space = self
            .repo
            .list_spaces()
            .await?
            .into_iter()
            .find(|space| space.slug == slug)
            .ok_or(BrainFault::NotFound)?;
        let rows = self.repo.list_articles(space.id).await?;
        let mut current: Option<&Article> = None;
        for segment in segments {
            let parent = current.map(|article| article.id);
            current = Some(
                rows.iter()
                    .find(|article| {
                        article.trashed_at.is_none()
                            && article.parent == parent
                            && article.slug == segment
                    })
                    .ok_or(BrainFault::NotFound)?,
            );
        }
        current
            .map(|article| article.id)
            .ok_or(BrainFault::NotFound)
    }

    pub(super) async fn unique_slug(
        &self,
        space: SpaceId,
        parent: Option<ArticleId>,
        title: &str,
    ) -> Result<String, BrainFault> {
        let taken: HashSet<String> = self
            .repo
            .list_children(space, parent)
            .await?
            .into_iter()
            .map(|article| article.slug)
            .collect();
        let base = slugify(title);
        let mut slug = base.clone();
        let mut attempt = 2;
        while taken.contains(&slug) {
            slug = format!("{base}-{attempt}");
            attempt += 1;
        }
        Ok(slug)
    }

    pub(super) async fn place(
        &self,
        space: SpaceId,
        parent: Option<ArticleId>,
        position: Option<Position>,
    ) -> Result<Option<i32>, BrainFault> {
        if position != Some(Position::First) {
            return Ok(None);
        }
        let lowest = self
            .repo
            .list_children(space, parent)
            .await?
            .into_iter()
            .map(|article| article.position)
            .min();
        Ok(Some(lowest.map_or(0, |lowest| lowest - 1)))
    }
}
