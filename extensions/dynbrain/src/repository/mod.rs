//! [`BrainRepository`]: the reads and writes over the Brain's tables.
//!
//! The queries are the checked `sqlx::query!` macros, with entries in the
//! workspace's `.sqlx/` cache; the integration suite runs every one of them
//! against the real schema.
//!
//! Every write that changes a space's tree or its grants first locks the
//! space row (`FOR UPDATE`), so two such writes on one space run one after
//! the other, and rewrites the access closure of the articles it affects
//! before it commits. Retrieval therefore never sees a tree and a closure
//! that disagree.

use std::collections::HashSet;
use std::sync::Arc;

use sqlx::{PgConnection, PgPool};
use systemprompt::identifiers::UserId;

use crate::access::{self, Grant, TreeNode};
use crate::error::BrainError;
use crate::ids::{ArticleId, SpaceId};

pub mod admin;
pub mod articles;
pub mod assets;
pub mod chunks;
pub mod grants;
pub mod ingest;
pub mod passages;
pub mod retrieval;
pub mod spaces;
pub mod types;
pub mod usage;
pub mod variants;

use types::{GrantRow, NodeRow, SpaceLockRow};

#[derive(Debug, Clone)]
pub struct BrainRepository {
    pool: Arc<PgPool>,
}

impl BrainRepository {
    pub const fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

async fn lock_space(conn: &mut PgConnection, space: SpaceId) -> Result<Option<UserId>, BrainError> {
    let row = sqlx::query_as!(
        SpaceLockRow,
        "SELECT id, created_by FROM dynbrain_space WHERE id = $1 FOR UPDATE",
        space.0
    )
    .fetch_optional(&mut *conn)
    .await?
    .ok_or(BrainError::SpaceNotFound(space))?;
    Ok(row.created_by.map(UserId::new))
}

async fn list_tree(conn: &mut PgConnection, space: SpaceId) -> Result<Vec<TreeNode>, BrainError> {
    let rows = sqlx::query_as!(
        NodeRow,
        "SELECT id, parent_id, inherit_access FROM dynbrain_article WHERE space_id = $1",
        space.0
    )
    .fetch_all(&mut *conn)
    .await?;
    Ok(rows.into_iter().map(TreeNode::from).collect())
}

async fn list_space_grants(
    conn: &mut PgConnection,
    space: SpaceId,
) -> Result<Vec<Grant>, BrainError> {
    let rows = sqlx::query_as!(
        GrantRow,
        "SELECT space_id, article_id, subject_kind, subject_id, level
           FROM dynbrain_grant WHERE space_id = $1",
        space.0
    )
    .fetch_all(&mut *conn)
    .await?;
    rows.into_iter().map(Grant::try_from).collect()
}

async fn rewrite_access(
    conn: &mut PgConnection,
    space: SpaceId,
    owner: Option<&UserId>,
    root: Option<ArticleId>,
) -> Result<(), BrainError> {
    let nodes = list_tree(conn, space).await?;
    let grants = list_space_grants(conn, space).await?;
    let scope: Vec<ArticleId> = root.map_or_else(
        || nodes.iter().map(|node| node.id).collect(),
        |root| access::subtree(&nodes, root),
    );
    let in_scope: HashSet<ArticleId> = scope.iter().copied().collect();
    let entries: Vec<access::AccessEntry> = access::closure(&nodes, &grants, owner)
        .into_iter()
        .filter(|entry| in_scope.contains(&entry.article))
        .collect();
    let scope_ids: Vec<i64> = scope.iter().map(|article| article.0).collect();
    let articles: Vec<i64> = entries.iter().map(|entry| entry.article.0).collect();
    let kinds: Vec<String> = entries
        .iter()
        .map(|entry| entry.subject.kind.as_db().to_owned())
        .collect();
    let subjects: Vec<String> = entries
        .iter()
        .map(|entry| entry.subject.id.clone())
        .collect();
    let levels: Vec<i16> = entries.iter().map(|entry| entry.level.as_db()).collect();
    sqlx::query!(
        "DELETE FROM dynbrain_access WHERE article_id = ANY($1::int8[])",
        scope_ids.as_slice()
    )
    .execute(&mut *conn)
    .await?;
    sqlx::query!(
        "INSERT INTO dynbrain_access (article_id, subject_kind, subject_id, level)
         SELECT * FROM UNNEST($1::int8[], $2::text[], $3::text[], $4::int2[])",
        articles.as_slice(),
        kinds.as_slice(),
        subjects.as_slice(),
        levels.as_slice()
    )
    .execute(&mut *conn)
    .await?;
    Ok(())
}
