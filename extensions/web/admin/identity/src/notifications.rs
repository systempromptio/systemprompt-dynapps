//! The `group` notification audience resolver.
//!
//! A `group:<id>` subject is the active users in that group's effective
//! membership (`user_groups`). Core caps the expansion at the profile's
//! `max_audience` and withholds a larger group.

use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;
use systemprompt::database::Database;
use systemprompt::extension::notifications::{
    Membership, NotificationAudienceError, NotificationAudienceResolver,
};
use systemprompt::identifiers::UserId;
use systemprompt::models::notifications::AudienceSubject;
use systemprompt_web_shared::GroupId;

use crate::repositories::groups::members::{is_active_member, list_active_member_ids};

const NAME: &str = "dynapps-groups";
const DIMENSION: &str = "group";

#[derive(Debug, Clone)]
pub struct GroupAudienceResolver {
    pool: Option<Arc<PgPool>>,
}

impl GroupAudienceResolver {
    #[must_use]
    pub const fn new(pool: Option<Arc<PgPool>>) -> Self {
        Self { pool }
    }

    fn pool(&self) -> Result<&PgPool, NotificationAudienceError> {
        self.pool
            .as_deref()
            .ok_or(NotificationAudienceError::Other {
                resolver: NAME,
                source: Box::new(NoDatabase),
            })
    }
}

#[derive(Debug, thiserror::Error)]
#[error("group audience resolver has no database pool")]
struct NoDatabase;

fn group(subject: &AudienceSubject) -> Option<GroupId> {
    match subject {
        AudienceSubject::Dimension { kind, value } if kind == DIMENSION => {
            Some(GroupId::new(value))
        },
        _ => None,
    }
}

fn failed(error: sqlx::Error) -> NotificationAudienceError {
    NotificationAudienceError::Other {
        resolver: NAME,
        source: Box::new(error),
    }
}

#[async_trait]
impl NotificationAudienceResolver for GroupAudienceResolver {
    fn name(&self) -> &'static str {
        NAME
    }

    fn handles(&self, subject: &AudienceSubject) -> bool {
        group(subject).is_some()
    }

    async fn members(
        &self,
        subject: &AudienceSubject,
        limit: usize,
    ) -> Result<Vec<UserId>, NotificationAudienceError> {
        let Some(group_id) = group(subject) else {
            return Ok(Vec::new());
        };
        list_active_member_ids(
            self.pool()?,
            &group_id,
            i64::try_from(limit).unwrap_or(i64::MAX),
        )
        .await
        .map_err(failed)
    }

    async fn is_member(
        &self,
        subject: &AudienceSubject,
        user: &UserId,
    ) -> Result<Membership, NotificationAudienceError> {
        let Some(group_id) = group(subject) else {
            return Ok(Membership::Unknown);
        };
        let member = is_active_member(self.pool()?, &group_id, user)
            .await
            .map_err(failed)?;
        Ok(if member {
            Membership::Member
        } else {
            Membership::NotMember
        })
    }
}

systemprompt::extension::register_notification_audience!(
    factory = |ctx| {
        let handle = ctx.database();
        let pool = handle
            .as_any()
            .downcast_ref::<Database>()
            .map(Database::pool);
        Arc::new(GroupAudienceResolver::new(pool))
    },
    priority = 100
);
