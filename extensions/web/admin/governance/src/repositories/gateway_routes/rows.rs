//! `gateway_routes` reads and writes, in file order.
//!
//! Each row holds one route as core serialises a `GatewayRoute`, so the
//! console reads back exactly what it wrote and what the gateway loads.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;

use systemprompt::manifest::services::GatewayRoute;
use systemprompt_web_admin_core::repositories::gateway_catalog::route_view;
use systemprompt_web_admin_core::types::{GatewayRouteView, RouteAliasView};

pub const SOURCE_CODE: &str = "code";
pub const SOURCE_DASHBOARD: &str = "dashboard";

#[derive(Debug, Clone, Serialize)]
pub struct RouteRow {
    pub route: GatewayRouteView,
    pub position: i32,
    pub source: String,
    pub updated_at: DateTime<Utc>,
}

fn decode_error(e: impl std::error::Error + Send + Sync + 'static) -> sqlx::Error {
    sqlx::Error::ColumnDecode {
        index: "route".to_owned(),
        source: Box::new(e),
    }
}

// JSON: the route column, core's own `GatewayRoute` serialisation
fn route_of(value: serde_json::Value) -> Result<GatewayRouteView, sqlx::Error> {
    let route: GatewayRoute = serde_json::from_value(value).map_err(decode_error)?;
    Ok(route_view(route))
}

// JSON: the route column written for storage
fn route_json(route: &GatewayRouteView) -> Result<serde_json::Value, sqlx::Error> {
    let yaml = systemprompt_web_admin_core::repositories::gateway_catalog::route_to_yaml(route)
        .map_err(decode_error)?;
    serde_json::to_value(yaml).map_err(decode_error)
}

pub async fn list_gateway_routes(pool: &PgPool) -> Result<Vec<RouteRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r"SELECT id, position, route, source, updated_at
            FROM gateway_routes
           ORDER BY position ASC, id ASC"
    )
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|r| {
            let mut route = route_of(r.route)?;
            route.id = r.id;
            Ok(RouteRow {
                route,
                position: r.position,
                source: r.source,
                updated_at: r.updated_at,
            })
        })
        .collect()
}

pub async fn count_gateway_routes(pool: &PgPool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(r#"SELECT COUNT(*) AS "count!" FROM gateway_routes"#)
        .fetch_one(pool)
        .await
}

#[derive(Debug, Clone)]
pub struct RouteWrite<'a> {
    pub route: &'a GatewayRouteView,
    pub position: i32,
    pub source: &'a str,
}

pub async fn upsert_gateway_route(
    pool: &PgPool,
    write: &RouteWrite<'_>,
) -> Result<(), sqlx::Error> {
    let route = route_json(write.route)?;
    sqlx::query!(
        r"INSERT INTO gateway_routes (id, position, route, source, updated_at)
          VALUES ($1, $2, $3, $4, NOW())
          ON CONFLICT (id) DO UPDATE
             SET position = EXCLUDED.position,
                 route = EXCLUDED.route,
                 source = EXCLUDED.source,
                 updated_at = NOW()",
        write.route.id,
        write.position,
        route,
        write.source,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_gateway_route(pool: &PgPool, id: &str) -> Result<u64, sqlx::Error> {
    Ok(sqlx::query!("DELETE FROM gateway_routes WHERE id = $1", id)
        .execute(pool)
        .await?
        .rows_affected())
}

pub async fn set_gateway_route_position(
    pool: &PgPool,
    id: &str,
    position: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE gateway_routes SET position = $2, updated_at = NOW() WHERE id = $1",
        id,
        position
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// The alias line a page shows for a route: `requested → model`.
#[must_use]
pub fn alias_line(alias: Option<&RouteAliasView>) -> String {
    alias.map_or_else(String::new, |a| format!("{} → {}", a.requested, a.model))
}
