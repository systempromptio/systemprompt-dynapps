//! The permission operations: reading and replacing the grants of a space or
//! an article, and the inheritance switch. All of them need `manage`.

use systemprompt::identifiers::UserId;

use super::NativeBrain;
use super::shape::require;
use crate::access::{Grant, Level, Subject};
use crate::brain::Reply;
use crate::brain::args::{GrantInput, GrantTarget, InheritArgs};
use crate::brain::fault::BrainFault;
use crate::brain::views::{GrantView, Grants, InheritChanged};
use crate::ids::{ArticleId, SpaceId};

impl NativeBrain {
    pub(super) async fn grants_of(&self, user: &UserId, target: GrantTarget) -> Reply<Grants> {
        let (space, article) = self.manageable(user, target).await?;
        self.read_grants(space, article).await
    }

    pub(super) async fn replace_grants(
        &self,
        user: &UserId,
        target: GrantTarget,
        grants: &[GrantInput],
    ) -> Reply<Grants> {
        let (space, article) = self.manageable(user, target).await?;
        let wanted: Vec<Grant> = grants
            .iter()
            .map(|grant| Grant {
                space,
                article,
                subject: Subject {
                    kind: grant.kind,
                    id: grant.id.clone(),
                },
                level: grant.level,
            })
            .collect();
        for grant in &wanted {
            self.repo.set_grant(grant, user).await?;
        }
        for current in self.repo.list_grants(space, article).await? {
            if !wanted.iter().any(|grant| grant.subject == current.subject) {
                self.repo
                    .revoke_grant(space, article, &current.subject)
                    .await?;
            }
        }
        self.read_grants(space, article).await
    }

    pub(super) async fn switch_inherit(
        &self,
        user: &UserId,
        article: ArticleId,
        args: InheritArgs,
    ) -> Reply<InheritChanged> {
        let (found, level) = self.reach(article, user).await?;
        require(level, Level::Manage)?;
        let changed = self
            .repo
            .set_inherit(found.id, args.inherit, args.copy_inherited, user)
            .await?;
        Ok(InheritChanged {
            article_id: changed.id,
            inherit_access: changed.inherit_access,
        })
    }

    async fn manageable(
        &self,
        user: &UserId,
        target: GrantTarget,
    ) -> Result<(SpaceId, Option<ArticleId>), BrainFault> {
        match target {
            GrantTarget::Space(id) => {
                let (space, level) = self.space_reach(id, user).await?;
                require(level, Level::Manage)?;
                Ok((space.id, None))
            },
            GrantTarget::Article(id) => {
                let (article, level) = self.reach(id, user).await?;
                require(level, Level::Manage)?;
                Ok((article.space, Some(article.id)))
            },
        }
    }

    async fn read_grants(
        &self,
        space: SpaceId,
        article: Option<ArticleId>,
    ) -> Result<Grants, BrainFault> {
        let grants = self
            .repo
            .list_grants(space, article)
            .await?
            .into_iter()
            .map(GrantView::from)
            .collect();
        let Some(id) = article else {
            return Ok(Grants {
                grants,
                access: Vec::new(),
                inherit_access: None,
            });
        };
        let access = self
            .repo
            .list_access(id)
            .await?
            .into_iter()
            .map(GrantView::from)
            .collect();
        let inherit_access = self
            .repo
            .find_article(id)
            .await?
            .map(|found| found.inherit_access);
        Ok(Grants {
            grants,
            access,
            inherit_access,
        })
    }
}
