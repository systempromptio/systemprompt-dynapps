//! The two candidate arms, with REQ-052 access control inside the queries.
//!
//! Grants are joined in the candidate queries themselves rather than applied
//! to their results. A post-filter would still read unauthorized rows, and —
//! because the candidate limit is fixed — would leak how many were removed
//! through the result count. Filtering in the query means an unauthorized
//! document is never read at all.
//!
//! The embedding is bound as its pgvector text form and cast in SQL, so the
//! crate needs no vector-aware sqlx type.

use crate::error::KnowledgeBankError;
use crate::principal::Subject;
use chrono::NaiveDate;
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub(super) struct Candidate {
    pub(super) chunk_id: String,
    pub(super) doc_id: String,
    pub(super) text: String,
    pub(super) lines: Option<String>,
    pub(super) chunk_index: i32,
    pub(super) chunk_total: i32,
    pub(super) source_type: String,
    pub(super) source: String,
    pub(super) title: String,
    pub(super) uri: String,
    pub(super) doc_date: Option<NaiveDate>,
    pub(super) date_source: Option<String>,
}

pub(super) struct Subjects {
    pub(super) kinds: Vec<String>,
    pub(super) values: Vec<String>,
}

impl Subjects {
    pub(super) fn from_slice(subjects: &[Subject]) -> Self {
        Self {
            kinds: subjects.iter().map(|s| s.kind.clone()).collect(),
            values: subjects.iter().map(|s| s.value.clone()).collect(),
        }
    }
}

pub(super) fn to_pgvector_literal(embedding: &[f32]) -> String {
    let joined = embedding
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

pub(super) async fn vector_candidates(
    pool: &PgPool,
    embedding: &str,
    source_types: &[String],
    subjects: &Subjects,
    limit: i64,
) -> Result<Vec<Candidate>, KnowledgeBankError> {
    let rows = sqlx::query_as!(
        Candidate,
        r#"
        SELECT c.chunk_id, c.doc_id, c.text, c.lines, c.chunk_index, c.chunk_total,
               d.source_type, d.source, d.title, d.uri, d.doc_date, d.date_source
        FROM knowledge_chunks c
        JOIN knowledge_documents d ON d.doc_id = c.doc_id
        WHERE d.source_type = ANY($2)
          AND EXISTS (
              SELECT 1 FROM knowledge_document_grants g
              WHERE g.doc_id = d.doc_id
                AND (g.subject_kind, g.subject_value) IN (
                    SELECT * FROM UNNEST($3::text[], $4::text[])
                )
          )
        ORDER BY c.embedding <=> ($1::text)::vector
        LIMIT $5
        "#,
        embedding,
        source_types,
        &subjects.kinds,
        &subjects.values,
        limit,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub(super) async fn text_candidates(
    pool: &PgPool,
    query: &str,
    source_types: &[String],
    subjects: &Subjects,
    limit: i64,
) -> Result<Vec<Candidate>, KnowledgeBankError> {
    let rows = sqlx::query_as!(
        Candidate,
        r#"
        SELECT c.chunk_id, c.doc_id, c.text, c.lines, c.chunk_index, c.chunk_total,
               d.source_type, d.source, d.title, d.uri, d.doc_date, d.date_source
        FROM knowledge_chunks c
        JOIN knowledge_documents d ON d.doc_id = c.doc_id
        WHERE d.source_type = ANY($2)
          AND c.tsv @@ plainto_tsquery('english', $1)
          AND EXISTS (
              SELECT 1 FROM knowledge_document_grants g
              WHERE g.doc_id = d.doc_id
                AND (g.subject_kind, g.subject_value) IN (
                    SELECT * FROM UNNEST($3::text[], $4::text[])
                )
          )
        ORDER BY ts_rank_cd(c.tsv, plainto_tsquery('english', $1)) DESC
        LIMIT $5
        "#,
        query,
        source_types,
        &subjects.kinds,
        &subjects.values,
        limit,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub(super) async fn count_denied_documents(
    pool: &PgPool,
    source_types: &[String],
    subjects: &Subjects,
) -> Result<i64, KnowledgeBankError> {
    let denied = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) AS "count!" FROM knowledge_documents d
        WHERE d.source_type = ANY($1)
          AND NOT EXISTS (
              SELECT 1 FROM knowledge_document_grants g
              WHERE g.doc_id = d.doc_id
                AND (g.subject_kind, g.subject_value) IN (
                    SELECT * FROM UNNEST($2::text[], $3::text[])
                )
          )
        "#,
        source_types,
        &subjects.kinds,
        &subjects.values,
    )
    .fetch_one(pool)
    .await?;
    Ok(denied)
}
