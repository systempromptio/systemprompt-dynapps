//! The `/admin/gateway` editor's writes: create, update, delete and reorder,
//! addressed by position as the page addresses them.
//!
//! Every write lands in `gateway_routes` stamped `dashboard`, renumbers the
//! positions so they stay `0..n`, and regenerates the file core boots from.
//! Every route carries a declared id: a new route without one takes the slug
//! of its label, alias or first glob, and an update keeps the row's id unless
//! the form names a new one, so the access rules that name a route still
//! match. An update preserves the fields the form does not render.

use std::path::Path;

use sqlx::PgPool;

use super::render::regenerate_gateway_file;
use super::rows::{
    RouteRow, RouteWrite, SOURCE_DASHBOARD, delete_gateway_route, list_gateway_routes,
    set_gateway_route_position, upsert_gateway_route,
};
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::repositories::gateway_catalog::{
    normalise_metadata, slugify_pattern, validate_route,
};
use systemprompt_web_admin_core::types::GatewayRouteView;

async fn renumber(pool: &PgPool, rows: &[RouteRow]) -> AdminResult<()> {
    for (i, r) in rows.iter().enumerate() {
        let position = i32::try_from(i).unwrap_or(i32::MAX);
        if r.position != position {
            set_gateway_route_position(pool, &r.route.id, position).await?;
        }
    }
    Ok(())
}

fn at(rows: &[RouteRow], index: usize) -> AdminResult<&RouteRow> {
    rows.get(index)
        .ok_or_else(|| AdminError::NotFound("Route not found".to_owned()))
}

pub async fn create_route_entry(
    pool: &PgPool,
    gateway_path: &Path,
    route: &GatewayRouteView,
) -> AdminResult<usize> {
    validate_route(route)?;
    let mut to_insert = route.clone();
    normalise_metadata(&mut to_insert);
    if to_insert.id.trim().is_empty() {
        to_insert.id = derived_id(&to_insert);
    }
    let rows = list_gateway_routes(pool).await?;
    if rows.iter().any(|r| r.route.id == to_insert.id) {
        return Err(AdminError::BadRequest(format!(
            "route id `{}` already exists",
            to_insert.id
        )));
    }
    let position = i32::try_from(rows.len()).unwrap_or(i32::MAX);
    upsert_gateway_route(
        pool,
        &RouteWrite {
            route: &to_insert,
            position,
            source: SOURCE_DASHBOARD,
        },
    )
    .await?;
    regenerate_gateway_file(pool, gateway_path).await?;
    Ok(rows.len())
}

fn derived_id(route: &GatewayRouteView) -> String {
    let stem = route
        .name
        .as_deref()
        .or_else(|| route.alias.as_ref().map(|alias| alias.requested.as_str()))
        .or_else(|| route.models.first().map(String::as_str))
        .unwrap_or("route");
    slugify_pattern(stem)
}

fn preserve_hidden_fields(merged: &mut GatewayRouteView, keep: &GatewayRouteView) {
    merged.when = merged.when.take().or_else(|| keep.when.clone());
    merged.requires = merged.requires.take().or_else(|| keep.requires.clone());
    merged.strategy = merged.strategy.take().or_else(|| keep.strategy.clone());
    if merged.context_fallbacks.is_empty() {
        merged.context_fallbacks.clone_from(&keep.context_fallbacks);
    }
}

pub async fn update_route_at(
    pool: &PgPool,
    gateway_path: &Path,
    index: usize,
    route: &GatewayRouteView,
) -> AdminResult<()> {
    validate_route(route)?;
    let rows = list_gateway_routes(pool).await?;
    let existing = at(&rows, index)?;
    let mut merged = route.clone();
    normalise_metadata(&mut merged);
    let keep = &existing.route;
    preserve_hidden_fields(&mut merged, keep);
    let body_id = merged.id.trim().to_owned();
    merged.id = if body_id.is_empty() {
        keep.id.clone()
    } else {
        body_id
    };
    if merged.id != keep.id {
        if rows.iter().any(|r| r.route.id == merged.id) {
            return Err(AdminError::BadRequest(format!(
                "route id `{}` already exists",
                merged.id
            )));
        }
        delete_gateway_route(pool, &keep.id).await?;
    }
    upsert_gateway_route(
        pool,
        &RouteWrite {
            route: &merged,
            position: existing.position,
            source: SOURCE_DASHBOARD,
        },
    )
    .await?;
    regenerate_gateway_file(pool, gateway_path).await?;
    Ok(())
}

pub async fn delete_route_at(pool: &PgPool, gateway_path: &Path, index: usize) -> AdminResult<()> {
    let rows = list_gateway_routes(pool).await?;
    let target = at(&rows, index)?;
    delete_gateway_route(pool, &target.route.id).await?;
    renumber(pool, &list_gateway_routes(pool).await?).await?;
    regenerate_gateway_file(pool, gateway_path).await?;
    Ok(())
}

pub async fn reorder_route_positions(
    pool: &PgPool,
    gateway_path: &Path,
    order: &[usize],
) -> AdminResult<()> {
    let rows = list_gateway_routes(pool).await?;
    let n = rows.len();
    if order.len() != n {
        return Err(AdminError::BadRequest(format!(
            "order has {} entries but there are {n} routes",
            order.len()
        )));
    }
    let mut seen = vec![false; n];
    for &i in order {
        if i >= n || seen[i] {
            return Err(AdminError::BadRequest(
                "order must be a permutation of route indices".to_owned(),
            ));
        }
        seen[i] = true;
    }
    for (position, &i) in order.iter().enumerate() {
        let position = i32::try_from(position).unwrap_or(i32::MAX);
        set_gateway_route_position(pool, &rows[i].route.id, position).await?;
    }
    regenerate_gateway_file(pool, gateway_path).await?;
    Ok(())
}
