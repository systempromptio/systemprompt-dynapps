//! The reading operations: spaces, one article, flat lists, the outline, the
//! Markdown export and the metadata. They show only what the person reaches.

use std::collections::HashSet;

use systemprompt::identifiers::UserId;

use super::NativeBrain;
use super::shape::{blank, path_of, pick, variant_info};
use crate::access::Level;
use crate::brain::Reply;
use crate::brain::args::{
    GetArgs, ListArgs, ListSpacesArgs, MetadataArgs, MetadataSchemaArgs, Object,
};
use crate::brain::fault::BrainFault;
use crate::brain::views::{
    ArticleList, ArticleView, ChildSummary, ListedArticle, MetadataItem, MetadataList,
    MetadataSchema, SpaceList, SpaceSummary,
};
use crate::ids::ArticleId;
use crate::model::Variant;

impl NativeBrain {
    pub(super) async fn fetch_article(&self, user: &UserId, args: &GetArgs) -> Reply<ArticleView> {
        let id = self.resolve(args.article_id, args.path.as_deref()).await?;
        let (article, level) = self.reach(id, user).await?;
        let space = self.space_row(article.space).await?;
        let variant = self
            .variant_of(&article, &space, args.lang.as_deref())
            .await?;
        let mut children = Vec::new();
        for child in self.repo.list_children(space.id, Some(id)).await? {
            let reaches = self.repo.find_level(child.id, user).await?;
            if reaches.is_none_or(|level| level < Level::Read) {
                continue;
            }
            let shown = self
                .variant_of(&child, &space, args.lang.as_deref())
                .await?;
            children.push(ChildSummary {
                article_id: child.id,
                title: shown.title,
                description: shown.description,
            });
        }
        let info = variant_info(&variant, &space);
        Ok(ArticleView {
            found: true,
            article_id: id,
            space: space.id,
            path: self.path_for(&space, &article).await?,
            title: variant.title,
            description: variant.description,
            lang: variant.lang,
            variant: info,
            markdown: variant.body,
            front_matter: variant.front_matter,
            metadata: variant.properties,
            revision: variant.revision,
            inherit_access: article.inherit_access,
            can_write: level >= Level::Write,
            children,
        })
    }

    pub(super) async fn fetch_spaces(
        &self,
        user: &UserId,
        args: &ListSpacesArgs,
    ) -> Reply<SpaceList> {
        let needle = args.query.as_deref().map(str::to_lowercase);
        let mut visible = Vec::new();
        for space in self.repo.list_spaces().await? {
            let named = needle
                .as_ref()
                .is_none_or(|needle| space.name.to_lowercase().contains(needle.as_str()));
            let spoken = args
                .lang
                .as_ref()
                .is_none_or(|lang| space.langs.contains(lang));
            if !named || !spoken {
                continue;
            }
            let own = self
                .repo
                .find_space_level(space.id, user)
                .await?
                .unwrap_or(Level::NoAccess);
            let count = self.readable(space.id, user).await?.len();
            if own < Level::Read && count == 0 {
                continue;
            }
            visible.push(SpaceSummary {
                space: space.id,
                slug: space.slug,
                name: space.name,
                lang: space.default_lang,
                langs: space.langs,
                article_count: count,
                level: own,
            });
        }
        let (total, offset, spaces) = page(visible, args.offset, args.limit, 50, 200);
        Ok(SpaceList {
            total,
            shown: spaces.len(),
            offset,
            spaces,
        })
    }

    pub(super) async fn fetch_list(&self, user: &UserId, args: &ListArgs) -> Reply<ArticleList> {
        let (space, _) = self.space_reach(args.space, user).await?;
        let reached = self.readable(space.id, user).await?;
        let all = self.repo.list_articles(space.id).await?;
        let present: HashSet<ArticleId> = reached.iter().map(|(article, _)| article.id).collect();
        let mut documents = Vec::new();
        for (article, _) in &reached {
            let variant = self
                .variant_of(article, &space, args.lang.as_deref())
                .await?;
            if !matches_filter(&variant, args.filter.as_ref()) {
                continue;
            }
            documents.push(ListedArticle {
                article_id: article.id,
                path: path_of(&space, article, &all),
                title: variant.title,
                lang: variant.lang,
                parent_id: article.parent.filter(|parent| present.contains(parent)),
            });
        }
        let (total, offset, documents) = page(documents, args.offset, args.limit, 200, 500);
        Ok(ArticleList {
            total,
            shown: documents.len(),
            offset,
            documents,
        })
    }

    pub(super) async fn fetch_metadata(
        &self,
        user: &UserId,
        args: &MetadataArgs,
    ) -> Reply<MetadataList> {
        let Some(ids) = &args.article_ids else {
            let id = self.resolve(args.article_id, args.path.as_deref()).await?;
            let item = self.metadata_item(user, id, args.lang.as_deref()).await?;
            return Ok(MetadataList {
                documents: vec![item],
            });
        };
        let cap = args.limit.unwrap_or(50).min(200) as usize;
        let mut documents = Vec::new();
        for id in ids.iter().take(cap) {
            match self.metadata_item(user, *id, args.lang.as_deref()).await {
                Ok(item) => documents.push(item),
                Err(BrainFault::NotFound) => {},
                Err(other) => return Err(other),
            }
        }
        Ok(MetadataList { documents })
    }

    async fn metadata_item(
        &self,
        user: &UserId,
        id: ArticleId,
        lang: Option<&str>,
    ) -> Reply<MetadataItem> {
        let (article, level) = self.reach(id, user).await?;
        let space = self.space_row(article.space).await?;
        let all = self.repo.list_articles(space.id).await?;
        let rows = self.repo.list_variants(id).await?;
        let variant_langs = rows.iter().map(|variant| variant.lang.clone()).collect();
        let variant =
            pick(rows, lang, &space.default_lang).unwrap_or_else(|| blank(&article, &space));
        let child_count = all
            .iter()
            .filter(|child| child.parent == Some(id) && child.trashed_at.is_none())
            .count();
        let parent_id = self.visible_parent(user, &article).await?;
        Ok(MetadataItem {
            id,
            path: path_of(&space, &article, &all),
            title: variant.title,
            description: variant.description,
            lang: variant.lang,
            space: space.id,
            is_synced: false,
            can_write: level >= Level::Write,
            inherit_access: article.inherit_access,
            author: article.created_by,
            created: article.created_at,
            last_editor: variant.updated_by,
            updated: variant.updated_at,
            parent_id,
            child_count,
            variant_langs,
            revision: variant.revision,
            front_matter: variant.front_matter,
            metadata: variant.properties,
        })
    }

    pub(super) async fn fetch_schema(
        &self,
        user: &UserId,
        args: &MetadataSchemaArgs,
    ) -> Reply<MetadataSchema> {
        let space = if args.article_id.is_some() || args.path.is_some() {
            let id = self.resolve(args.article_id, args.path.as_deref()).await?;
            let (article, _) = self.reach(id, user).await?;
            self.space_row(article.space).await?
        } else {
            let id = args
                .space
                .ok_or(BrainFault::Invalid("pass article_id, path or space"))?;
            self.space_reach(id, user).await?.0
        };
        Ok(MetadataSchema {
            space: space.id,
            fields: space.properties.0,
        })
    }
}

fn page<T>(
    items: Vec<T>,
    offset: Option<u32>,
    limit: Option<u32>,
    default: u32,
    max: u32,
) -> (usize, usize, Vec<T>) {
    let total = items.len();
    let offset = offset.unwrap_or(0) as usize;
    let limit = limit.unwrap_or(default).min(max) as usize;
    (
        total,
        offset,
        items.into_iter().skip(offset).take(limit).collect(),
    )
}

fn matches_filter(variant: &Variant, filter: Option<&Object>) -> bool {
    filter.is_none_or(|wanted| {
        wanted
            .iter()
            .all(|(key, value)| variant.properties.get(key) == Some(value))
    })
}
