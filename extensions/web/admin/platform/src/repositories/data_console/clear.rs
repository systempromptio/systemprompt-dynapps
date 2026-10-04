//! Actor-bound, expiring confirmation tickets and atomic deletion receipts.
use super::browse::Selection;
use super::clear_plan::{ClearPlan, PreparedPlan, prepare, temp};
use super::{bad, transaction, transaction_with_isolation};
use serde::Serialize;
use sqlx::{PgConnection, PgPool, Row};
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct ClearPreview {
    pub token: String,
    pub table: String,
    pub plan: ClearPlan,
}

async fn insert_clear_preview_inner(
    pool: &PgPool,
    actor: &str,
    selection: &Selection,
) -> AdminResult<ClearPreview> {
    let mut tx = transaction(pool, "5s").await?;
    // Why: Expired tickets are small operational state, collected only on preview.
    sqlx::query("DELETE FROM platform_data_clear_previews WHERE expires_at < now()")
        .execute(&mut *tx)
        .await?;
    let ticket = Uuid::new_v4();
    let plan = prepare(&mut tx, selection, false, ticket).await?.receipt;
    sqlx::query(
        "INSERT INTO platform_data_clear_previews(id,actor_id,selection,plan) VALUES($1,$2,$3,$4)",
    )
    .bind(ticket)
    .bind(actor)
    .bind(serde_json::to_value(selection).map_err(AdminError::internal)?)
    .bind(serde_json::to_value(&plan).map_err(AdminError::internal)?)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(ClearPreview {
        token: ticket.to_string(),
        table: selection.table.clone(),
        plan,
    })
}

async fn delete_confirmed_rows_inner(
    conn: &mut PgConnection,
    actor: &str,
    token: &str,
    confirmation: &str,
) -> AdminResult<ClearPlan> {
    let ticket = Uuid::parse_str(token).map_err(|_| bad("Invalid confirmation ticket."))?;
    let row = sqlx::query("SELECT selection,plan FROM platform_data_clear_previews WHERE id=$1 AND actor_id=$2 AND expires_at > now() FOR UPDATE")
        .bind(ticket).bind(actor).fetch_optional(&mut *conn).await?
        .ok_or_else(|| bad("Confirmation expired or was already used. Preview the clearing operation again."))?;
    let selection: Selection =
        serde_json::from_value(row.try_get("selection")?).map_err(AdminError::internal)?;
    let approved: ClearPlan =
        serde_json::from_value(row.try_get("plan")?).map_err(AdminError::internal)?;
    if confirmation != selection.table {
        return Err(bad(
            "Type the exact table name to confirm irreversible deletion and related-table effects.",
        ));
    }
    let prepared = prepare(conn, &selection, true, ticket).await?;
    if prepared.receipt != approved {
        return Err(AdminError::Conflict(
            "The affected data or schema changed. No data was deleted; create a fresh preview."
                .into(),
        ));
    }
    sqlx::query("SET CONSTRAINTS ALL DEFERRED")
        .execute(&mut *conn)
        .await?;
    delete_planned_rows(conn, &prepared).await?;
    verify_deleted_rows(conn, &prepared).await?;
    sqlx::query("SET CONSTRAINTS ALL IMMEDIATE")
        .execute(&mut *conn)
        .await?;
    sqlx::query("DELETE FROM platform_data_clear_previews WHERE id=$1")
        .bind(ticket)
        .execute(&mut *conn)
        .await?;
    let mut result = prepared.receipt;
    result.effects.retain(|e| e.deleted > 0 || e.updated > 0);
    sqlx::query("INSERT INTO platform_data_clear_audit(id,actor_id,selected_table,selection,effects) VALUES($1,$2,$3,$4,$5)")
        .bind(Uuid::new_v4()).bind(actor).bind(&selection.table)
        .bind(serde_json::to_value(&selection).map_err(AdminError::internal)?)
        .bind(serde_json::to_value(&result).map_err(AdminError::internal)?).execute(&mut *conn).await?;
    Ok(result)
}

async fn delete_planned_rows(conn: &mut PgConnection, prepared: &PreparedPlan) -> AdminResult<()> {
    for oid in &prepared.order {
        let table = &prepared.tables[oid];
        let sql = format!(
            "DELETE FROM {} t USING {} x WHERE x.deleting AND t.tableoid=x.rel AND t.ctid=x.tid",
            table.sql(),
            temp(*oid)
        );
        let count = sqlx::query(sqlx::AssertSqlSafe(sql.as_str()))
            .execute(&mut *conn)
            .await?
            .rows_affected();
        // Why: Postgres rows reached by an earlier CASCADE may already be gone; the
        // approved closure is the total affected count, not just direct command
        // counts.
        let expected = prepared
            .receipt
            .effects
            .iter()
            .find(|e| e.table == table.label())
            .map_or(0, |e| e.deleted);
        if count > u64::try_from(expected).unwrap_or(0) {
            return Err(AdminError::Conflict(
                "Deletion exceeded its approved preview; rolled back.".into(),
            ));
        }
    }
    Ok(())
}

async fn verify_deleted_rows(conn: &mut PgConnection, prepared: &PreparedPlan) -> AdminResult<()> {
    for table in prepared.tables.values() {
        let sql = format!(
            "SELECT count(*)::bigint FROM {} t JOIN {} x ON t.tableoid=x.rel AND t.ctid=x.tid WHERE x.deleting",
            table.sql(),
            temp(table.oid)
        );
        let remaining: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
            .fetch_one(&mut *conn)
            .await?;
        if remaining != 0 {
            return Err(AdminError::Conflict(
                "A database trigger prevented approved row deletion; all changes were rolled back."
                    .into(),
            ));
        }
    }
    Ok(())
}

// Why: Bound the entire preview, rather than allowing each dependency query
// five seconds.
pub async fn insert_clear_preview(
    pool: &PgPool,
    actor: &str,
    selection: &Selection,
) -> AdminResult<ClearPreview> {
    tokio::time::timeout(
        std::time::Duration::from_secs(5),
        insert_clear_preview_inner(pool, actor, selection),
    )
    .await
    .map_err(|_| bad("Preview exceeded five seconds. No data was cleared; narrow the selection."))?
}
// Why: SQLx rolls back dropped transactions before their connections return
// to the pool, including timeout cancellation.
pub async fn delete_confirmed_rows(
    pool: &PgPool,
    actor: &str,
    token: &str,
    confirmation: &str,
) -> AdminResult<ClearPlan> {
    // Why: Postgres READ COMMITTED sees changes committed while table locks were
    // awaited; the locked tables then keep the rechecked receipt stable.
    let mut tx = transaction_with_isolation(pool, "10s", false).await?;
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        delete_confirmed_rows_inner(&mut tx, actor, token, confirmation),
    )
    .await
    .map_err(|_| {
        bad("Clearing exceeded ten seconds. No changes were committed; narrow the selection.")
    })??;
    // Why: Postgres COMMIT must finish outside cancellation of the work timeout,
    // so a successful commit cannot be mistaken for a rolled-back timeout.
    tx.commit().await?;
    Ok(result)
}
