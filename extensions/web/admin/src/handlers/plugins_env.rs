//! HTTP handlers for plugin environment variables.
//!
//! A `config.yaml` that fails to parse is an error, never an empty definition
//! list: the definitions decide `valid`, which unlocks export, so degrading
//! would fail open.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::config::ProfileBootstrap;

use systemprompt::identifiers::{PluginId, UserId};

use crate::error::{AdminError, AdminResult};
use crate::repositories;
use crate::types::UserQuery;

use super::responses::PluginEnvResponse;

pub(crate) async fn list_plugin_env_handler(
    State(pool): State<Arc<PgPool>>,
    Path(plugin_id): Path<PluginId>,
    headers: HeaderMap,
    Query(query): Query<UserQuery>,
) -> AdminResult<Response> {
    let cookie_uid = super::extract_user_from_cookie(&headers)
        .ok()
        .map(|s| s.user_id);
    let Some(user_id) = resolve_principal(
        cookie_uid.as_ref().map(UserId::as_str),
        query.user_id.as_ref().map(UserId::as_str),
    ) else {
        return Err(AdminError::Unauthorized("missing principal".to_owned()));
    };

    let definitions = load_plugin_variable_defs(&plugin_id)?;

    let stored =
        repositories::marketplace::plugin_env::list_plugin_env_vars(&pool, &user_id, &plugin_id)
            .await?;

    let stored_names: std::collections::HashSet<String> = stored
        .iter()
        .filter(|v| !v.var_value.is_empty())
        .map(|v| v.var_name.clone())
        .collect();
    let missing_required: Vec<String> = definitions
        .iter()
        .filter(|def| def.required && !stored_names.contains(&def.name))
        .map(|def| def.name.clone())
        .collect();

    Ok(Json(PluginEnvResponse {
        definitions,
        stored,
        valid: missing_required.is_empty(),
        missing_required,
    })
    .into_response())
}

pub fn resolve_principal(
    cookie_user_id: Option<&str>,
    query_user_id: Option<&str>,
) -> Option<UserId> {
    cookie_user_id.or(query_user_id).map(UserId::new)
}

fn load_plugin_variable_defs(
    plugin_id: &PluginId,
) -> Result<Vec<PluginVariableDef>, systemprompt_web_shared::error::MarketplaceError> {
    let services_path =
        ProfileBootstrap::get().map(|p| std::path::PathBuf::from(&p.paths.services))?;
    let config_path = services_path
        .join("plugins")
        .join(plugin_id.as_str())
        .join("config.yaml");
    if !config_path.exists() {
        return Ok(vec![]);
    }
    let content = std::fs::read_to_string(&config_path)?;
    let config: PluginConfigFile = serde_yaml::from_str(&content)?;
    Ok(config.plugin.map(|p| p.variables).unwrap_or_default())
}

#[derive(Debug, Deserialize)]
struct PluginConfigFile {
    plugin: Option<PluginSection>,
}

#[derive(Debug, Deserialize)]
struct PluginSection {
    #[serde(default)]
    variables: Vec<PluginVariableDef>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct PluginVariableDef {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "required_by_default")]
    pub required: bool,
    #[serde(default)]
    pub secret: bool,
    #[serde(default)]
    pub example: String,
}

const fn required_by_default() -> bool {
    true
}
