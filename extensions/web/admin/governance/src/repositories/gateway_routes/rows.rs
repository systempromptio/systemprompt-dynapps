//! `gateway_routes` reads and writes, in dispatch order.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use systemprompt_web_admin_core::types::{GatewayRouteView, RouteFallbackView};

pub const SOURCE_CODE: &str = "code";
pub const SOURCE_DASHBOARD: &str = "dashboard";

#[derive(Debug, Clone, Serialize)]
pub struct RouteRow {
    pub route: GatewayRouteView,
    pub position: i32,
    pub explicit_id: bool,
    pub source: String,
    pub updated_at: DateTime<Utc>,
}

// JSON: variable-shape route blocks (pricing/when/requires/by_scope) as stored
fn yaml_of(value: Option<serde_json::Value>) -> Option<serde_yaml::Value> {
    value.and_then(|v| serde_yaml::to_value(v).ok())
}

// JSON: variable-shape route block written for storage
fn json_of(value: Option<&serde_yaml::Value>) -> Option<serde_json::Value> {
    value.and_then(|v| serde_json::to_value(v).ok())
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct RoutingColumn {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    strategy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    weight: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    context_fallbacks: Vec<RouteFallbackView>,
}

// JSON: fixed-shape routing column as stored
fn routing_of(value: Option<serde_json::Value>) -> Result<RoutingColumn, sqlx::Error> {
    value.map_or_else(
        || Ok(RoutingColumn::default()),
        |v| {
            serde_json::from_value(v).map_err(|e| sqlx::Error::ColumnDecode {
                index: "routing".to_owned(),
                source: Box::new(e),
            })
        },
    )
}

// JSON: fixed-shape routing column written for storage
fn routing_json(route: &GatewayRouteView) -> Option<serde_json::Value> {
    let column = RoutingColumn {
        strategy: route.strategy.clone(),
        weight: route.weight,
        context_fallbacks: route.context_fallbacks.clone(),
    };
    let empty =
        column.strategy.is_none() && column.weight.is_none() && column.context_fallbacks.is_empty();
    if empty {
        return None;
    }
    // Why: discard-ok: a struct of strings, an integer and string pairs always
    // serialises
    serde_json::to_value(column).ok()
}

pub async fn list_gateway_routes(pool: &PgPool) -> Result<Vec<RouteRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r"SELECT id, position, name, description, model_pattern, provider, upstream_model,
                 extra_headers, pricing, when_match, requires, fallbacks, by_scope,
                 routing, explicit_id, source, updated_at
            FROM gateway_routes
           ORDER BY position ASC, id ASC"
    )
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|r| {
            // JSON: variable-shape header map column
            let extra_headers =
                serde_json::from_value(r.extra_headers).map_err(|e| sqlx::Error::ColumnDecode {
                    index: "extra_headers".to_owned(),
                    source: Box::new(e),
                })?;
            // JSON: ordered failover deployments column
            let fallbacks =
                serde_json::from_value(r.fallbacks).map_err(|e| sqlx::Error::ColumnDecode {
                    index: "fallbacks".to_owned(),
                    source: Box::new(e),
                })?;
            let routing = routing_of(r.routing)?;
            Ok(RouteRow {
                route: GatewayRouteView {
                    id: r.id,
                    name: r.name,
                    description: r.description,
                    model_pattern: r.model_pattern,
                    provider: r.provider,
                    upstream_model: r.upstream_model,
                    extra_headers,
                    pricing: yaml_of(r.pricing),
                    when: yaml_of(r.when_match),
                    requires: yaml_of(r.requires),
                    fallbacks,
                    by_scope: yaml_of(r.by_scope),
                    strategy: routing.strategy,
                    weight: routing.weight,
                    context_fallbacks: routing.context_fallbacks,
                },
                position: r.position,
                explicit_id: r.explicit_id,
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
    pub explicit_id: bool,
    pub source: &'a str,
}

pub async fn upsert_gateway_route(
    pool: &PgPool,
    write: &RouteWrite<'_>,
) -> Result<(), sqlx::Error> {
    let r = write.route;
    // JSON: outgoing fixed-shape
    // Why: discard-ok: a string-keyed map of strings always serialises
    let headers = serde_json::to_value(&r.extra_headers).unwrap_or_default();
    // JSON: outgoing fixed-shape
    // Why: discard-ok: a list of provider/model string pairs always serialises
    let fallbacks = serde_json::to_value(&r.fallbacks).unwrap_or_default();
    sqlx::query!(
        r"INSERT INTO gateway_routes
              (id, position, model_pattern, provider, upstream_model, extra_headers, pricing,
               when_match, requires, fallbacks, by_scope, explicit_id,
               source, name, description, routing, updated_at)
          VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, NOW())
          ON CONFLICT (id) DO UPDATE
             SET position = EXCLUDED.position,
                 name = EXCLUDED.name,
                 description = EXCLUDED.description,
                 model_pattern = EXCLUDED.model_pattern,
                 provider = EXCLUDED.provider,
                 upstream_model = EXCLUDED.upstream_model,
                 extra_headers = EXCLUDED.extra_headers,
                 pricing = EXCLUDED.pricing,
                 when_match = EXCLUDED.when_match,
                 requires = EXCLUDED.requires,
                 fallbacks = EXCLUDED.fallbacks,
                 by_scope = EXCLUDED.by_scope,
                 routing = EXCLUDED.routing,
                 explicit_id = EXCLUDED.explicit_id,
                 source = EXCLUDED.source,
                 updated_at = NOW()",
        r.id,
        write.position,
        r.model_pattern,
        r.provider,
        r.upstream_model,
        headers,
        json_of(r.pricing.as_ref()),
        json_of(r.when.as_ref()),
        json_of(r.requires.as_ref()),
        fallbacks,
        json_of(r.by_scope.as_ref()),
        write.explicit_id,
        write.source,
        r.name,
        r.description,
        routing_json(r),
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
