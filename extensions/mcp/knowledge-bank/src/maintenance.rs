//! Keeping the shared index small and honest: `prune_knowledge`.
//!
//! The index accumulates grants from everyone who syncs and documents that
//! only they could read. When a user leaves, or stops syncing, their grants
//! are the only thing keeping some documents alive. Pruning drops the grants
//! of users whose account is gone or whose last sync is older than
//! `stale_days`, deletes the documents nobody holds any more, and clears the
//! cache rows nothing references. It runs in one transaction; a dry run
//! rolls it back and reports what it would have done.

use crate::error::KnowledgeBankError;
use crate::principal::SUBJECT_USER;
use chrono::{Duration, Utc};
use sqlx::PgPool;
use std::sync::Arc;

pub const DEFAULT_STALE_DAYS: u32 = 90;

/// What a prune should do.
#[derive(Debug, Clone, Copy)]
pub struct PruneOptions {
    pub stale_days: u32,
    pub dry_run: bool,
}

impl Default for PruneOptions {
    fn default() -> Self {
        Self {
            stale_days: DEFAULT_STALE_DAYS,
            dry_run: false,
        }
    }
}

/// What a prune did — or, on a dry run, would have done.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PruneReport {
    pub dry_run: bool,
    pub stale_users: Vec<String>,
    pub missing_users: Vec<String>,
    pub grants_removed: u64,
    pub sync_states_removed: u64,
    pub documents_deleted: u64,
    pub date_cache_removed: u64,
    pub embedding_cache_removed: u64,
}

/// The pruner: needs the knowledge pool for the index and the instance pool
/// for the `users` table, which lives with identity, not with the index.
pub struct Pruner {
    knowledge: Arc<PgPool>,
    instance: Arc<PgPool>,
}

impl std::fmt::Debug for Pruner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pruner").finish_non_exhaustive()
    }
}

impl Pruner {
    #[must_use]
    pub const fn new(knowledge: Arc<PgPool>, instance: Arc<PgPool>) -> Self {
        Self {
            knowledge,
            instance,
        }
    }

    pub async fn prune(&self, options: PruneOptions) -> Result<PruneReport, KnowledgeBankError> {
        let cutoff = Utc::now() - Duration::days(i64::from(options.stale_days));
        let mut report = PruneReport {
            dry_run: options.dry_run,
            ..PruneReport::default()
        };
        self.find_doomed_users(cutoff, &mut report).await?;
        let mut doomed: Vec<String> = report
            .missing_users
            .iter()
            .chain(&report.stale_users)
            .cloned()
            .collect();
        doomed.sort();
        doomed.dedup();

        let mut tx = self.knowledge.begin().await?;
        Self::sweep(&mut tx, &doomed, cutoff, &mut report).await?;
        if options.dry_run {
            tx.rollback().await?;
        } else {
            tx.commit().await?;
        }
        Ok(report)
    }

    async fn find_doomed_users(
        &self,
        cutoff: chrono::DateTime<Utc>,
        report: &mut PruneReport,
    ) -> Result<(), KnowledgeBankError> {
        let granted_users = sqlx::query_scalar!(
            r#"SELECT DISTINCT subject_value AS "user_id!" FROM knowledge_document_grants
               WHERE subject_kind = $1
               UNION
               SELECT DISTINCT user_id FROM knowledge_user_sync_state"#,
            SUBJECT_USER,
        )
        .fetch_all(&*self.knowledge)
        .await?;
        let existing =
            sqlx::query_scalar!(r#"SELECT id FROM users WHERE id = ANY($1)"#, &granted_users)
                .fetch_all(&*self.instance)
                .await?;
        report.missing_users = granted_users
            .iter()
            .filter(|id| !existing.contains(id))
            .cloned()
            .collect();
        report.stale_users = sqlx::query_scalar!(
            r#"SELECT user_id FROM knowledge_user_sync_state
               GROUP BY user_id
               HAVING MAX(last_run_at) < $1"#,
            cutoff,
        )
        .fetch_all(&*self.knowledge)
        .await?;
        Ok(())
    }

    async fn sweep(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        doomed: &[String],
        cutoff: chrono::DateTime<Utc>,
        report: &mut PruneReport,
    ) -> Result<(), KnowledgeBankError> {
        report.grants_removed = sqlx::query!(
            r#"DELETE FROM knowledge_document_grants
               WHERE subject_kind = $1 AND subject_value = ANY($2)"#,
            SUBJECT_USER,
            doomed,
        )
        .execute(&mut **tx)
        .await?
        .rows_affected();
        report.sync_states_removed = sqlx::query!(
            r#"DELETE FROM knowledge_user_sync_state WHERE user_id = ANY($1)"#,
            doomed,
        )
        .execute(&mut **tx)
        .await?
        .rows_affected();
        report.documents_deleted = sqlx::query!(
            r#"DELETE FROM knowledge_documents d
               WHERE NOT EXISTS (
                   SELECT 1 FROM knowledge_document_grants g WHERE g.doc_id = d.doc_id
               )"#
        )
        .execute(&mut **tx)
        .await?
        .rows_affected();
        report.date_cache_removed = sqlx::query!(
            r#"DELETE FROM knowledge_date_cache c
               WHERE NOT EXISTS (
                   SELECT 1 FROM knowledge_documents d WHERE d.doc_id = c.doc_id
               )"#
        )
        .execute(&mut **tx)
        .await?
        .rows_affected();
        report.embedding_cache_removed = sqlx::query!(
            r#"DELETE FROM knowledge_embedding_cache c
               WHERE c.created_at < $1
                 AND NOT EXISTS (
                     SELECT 1 FROM knowledge_chunks k
                     WHERE encode(sha256(convert_to(k.text, 'UTF8')), 'hex') = c.text_hash
                 )"#,
            cutoff,
        )
        .execute(&mut **tx)
        .await?
        .rows_affected();

        Ok(())
    }
}
