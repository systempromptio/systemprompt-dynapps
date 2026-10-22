//! Database → `gateway.yaml`: the rows rendered into the file's `routes:`
//! sequence, everything else in the file kept.
//!
//! Two callers. [`regenerate_gateway_file`] rewrites the baked file in place
//! after every table write, so the next restart dispatches the table; it is
//! the only path by which a console decision reaches core.
//! [`render_routes_export`] renders the same document to a string for the sync
//! page's export and the kit patch, without touching disk.

use std::path::Path;

use serde_yaml::Value;
use sqlx::PgPool;

use super::rows::{RouteRow, list_gateway_routes};
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::repositories::gateway_catalog::{replace_routes, route_to_yaml};
use systemprompt_web_admin_core::types::GatewayRouteView;

fn views(rows: &[RouteRow]) -> Vec<GatewayRouteView> {
    rows.iter().map(|r| r.route.clone()).collect()
}

// Why: a serde_yaml `Value` round-trip drops every comment; only the block
// above `gateway:` can be re-attached, because nothing above it moves.
fn leading_comment_header(yaml: &str) -> String {
    yaml.lines()
        .take_while(|line| line.trim_start().starts_with('#'))
        .fold(String::new(), |mut acc, line| {
            acc.push_str(line);
            acc.push('\n');
            acc
        })
}

#[derive(Debug, thiserror::Error)]
pub enum RenderRoutesError {
    #[error("gateway.yaml does not parse: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("gateway YAML root is not a mapping")]
    RootNotMapping,
    #[error("gateway block is not a mapping")]
    GatewayNotMapping,
}

pub fn render_routes_export(
    file_yaml: &str,
    rows: &[RouteRow],
) -> Result<String, RenderRoutesError> {
    let mut doc: Value = serde_yaml::from_str(file_yaml)?;
    let root = doc
        .as_mapping_mut()
        .ok_or(RenderRoutesError::RootNotMapping)?;
    let gateway = root
        .entry(Value::from("gateway"))
        .or_insert_with(|| Value::Mapping(serde_yaml::Mapping::new()));
    let gateway = gateway
        .as_mapping_mut()
        .ok_or(RenderRoutesError::GatewayNotMapping)?;
    gateway.insert(
        Value::from("routes"),
        Value::Sequence(rows.iter().map(|r| route_to_yaml(&r.route)).collect()),
    );
    let body = serde_yaml::to_string(&doc)?;
    Ok(format!("{}{body}", leading_comment_header(file_yaml)))
}

pub async fn regenerate_gateway_file(pool: &PgPool, gateway_path: &Path) -> AdminResult<usize> {
    let rows = list_gateway_routes(pool).await?;
    replace_routes(gateway_path, &views(&rows)).map_err(AdminError::from)?;
    Ok(rows.len())
}
