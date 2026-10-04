//! Parsed read-only queries, executed inside a `PostgreSQL` read-only
//! transaction.
use super::{bad, read_only, transaction};
use futures_util::TryStreamExt;
use sqlparser::ast::{Expr, Query, Select, Statement, Visit, Visitor};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;
use sqlx::{PgPool, Row};
use std::ops::ControlFlow;
use systemprompt_web_admin_core::error::AdminResult;

struct ReadQuery;
impl Visitor for ReadQuery {
    type Break = &'static str;
    fn pre_visit_expr(&mut self, expr: &Expr) -> ControlFlow<Self::Break> {
        if let Expr::Function(function) = expr {
            let name = function.name.to_string().to_ascii_lowercase();
            let leaf = name.rsplit('.').next().unwrap_or(&name).trim_matches('"');
            if [
                "set_config",
                "pg_notify",
                "pg_cancel_backend",
                "pg_terminate_backend",
                "pg_reload_conf",
                "pg_rotate_logfile",
                "nextval",
                "setval",
            ]
            .contains(&leaf)
                || leaf.starts_with("pg_advisory_")
                || leaf.starts_with("dblink")
                || leaf.starts_with("lo_")
                || leaf.starts_with("pg_read_")
                || leaf.starts_with("pg_write_")
                || (leaf.starts_with("pg_")
                    && !leaf.starts_with("pg_get_")
                    && ![
                        "pg_sleep",
                        "pg_typeof",
                        "pg_size_pretty",
                        "pg_column_size",
                        "pg_table_size",
                        "pg_relation_size",
                        "pg_total_relation_size",
                        "pg_indexes_size",
                        "pg_is_in_recovery",
                    ]
                    .contains(&leaf))
            {
                return ControlFlow::Break(
                    "Session-changing, external-access and operational functions are not accepted in read-only queries.",
                );
            }
        }
        ControlFlow::Continue(())
    }
    fn pre_visit_statement(&mut self, stmt: &Statement) -> ControlFlow<Self::Break> {
        if matches!(stmt, Statement::Query(_)) {
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break("Only read-only SELECT queries are accepted.")
        }
    }
    fn pre_visit_query(&mut self, q: &Query) -> ControlFlow<Self::Break> {
        if q.locks.is_empty() {
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break("Row-locking queries are not accepted.")
        }
    }
    fn pre_visit_select(&mut self, s: &Select) -> ControlFlow<Self::Break> {
        if s.into.is_none() {
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break("SELECT INTO is not accepted.")
        }
    }
}

pub fn validate_query(input: &str) -> AdminResult<String> {
    if input.len() > 32_768 {
        return Err(bad("Queries are limited to 32 KiB."));
    }
    let statements = Parser::parse_sql(&PostgreSqlDialect {}, input)
        .map_err(|_error| bad("Could not parse the SQL query."))?;
    if statements.len() != 1 {
        return Err(bad("Enter exactly one SELECT query."));
    }
    if let ControlFlow::Break(message) = statements.visit(&mut ReadQuery) {
        return Err(bad(message));
    }
    Ok(statements[0].to_string())
}

#[derive(Debug, serde::Serialize)]
pub struct QueryResult {
    pub rows: Vec<String>,
    pub limited: bool,
}

pub async fn get_query_result(pool: &PgPool, input: &str) -> AdminResult<QueryResult> {
    let query = validate_query(input)?;
    let mut tx = transaction(pool, "3s").await?;
    read_only(&mut tx).await?;
    let sql = format!("SELECT left(row_to_json(q)::text,65536) AS body FROM ({query}) q LIMIT 201");
    let mut result = QueryResult {
        rows: Vec::new(),
        limited: false,
    };
    let mut bytes = 0;
    {
        let mut stream = sqlx::query(sqlx::AssertSqlSafe(sql.as_str())).fetch(&mut *tx);
        while let Some(row) = stream.try_next().await? {
            let body: String = row.try_get("body")?;
            if result.rows.len() == 200 || bytes + body.len() > 1_048_576 {
                result.limited = true;
                break;
            }
            bytes += body.len();
            if body.chars().count() == 65_536 {
                result.limited = true;
            }
            result.rows.push(body);
        }
    }
    tx.rollback().await?;
    Ok(result)
}
