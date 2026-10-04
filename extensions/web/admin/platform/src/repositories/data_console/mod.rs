//! Catalog-driven SQL for the platform data console. Identifiers come only
//! from PostgreSQL's catalog; values are bound. Dynamic queries are necessary
//! because operators select application tables at runtime.
pub mod browse;
pub mod catalog;
pub mod clear;
mod clear_materialize;
pub mod clear_plan;
pub(crate) mod projection;
pub mod query;

use sqlx::{PgConnection, PgPool, Postgres, Transaction};
use systemprompt_web_admin_core::error::{AdminError, AdminResult};

pub(crate) async fn transaction<'pool>(
    pool: &'pool PgPool,
    timeout: &str,
) -> AdminResult<Transaction<'pool, Postgres>> {
    transaction_with_isolation(pool, timeout, true).await
}

pub(crate) async fn transaction_with_isolation<'pool>(
    pool: &'pool PgPool,
    timeout: &str,
    repeatable: bool,
) -> AdminResult<Transaction<'pool, Postgres>> {
    let mut tx = pool.begin().await?;
    if repeatable {
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ")
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query(
        "SELECT set_config('statement_timeout', $1, true), set_config('lock_timeout', '1s', true)",
    )
    .bind(timeout)
    .execute(&mut *tx)
    .await?;
    Ok(tx)
}

pub(crate) async fn read_only(conn: &mut PgConnection) -> AdminResult<()> {
    sqlx::query("SET TRANSACTION READ ONLY")
        .execute(conn)
        .await?;
    Ok(())
}

pub(crate) fn bad(message: impl Into<String>) -> AdminError {
    AdminError::BadRequest(message.into())
}

#[must_use]
pub fn quote_identifier(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

pub(crate) fn read_error(error: sqlx::Error) -> AdminError {
    if let sqlx::Error::Database(database) = &error {
        if database.code().is_some_and(|code| code.starts_with("22")) {
            return bad("A filter or cursor value is not valid for this table.");
        }
    }
    error.into()
}
