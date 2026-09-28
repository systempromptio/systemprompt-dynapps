//! Capture the agreed four-skill baseline without changing active skills.

use axum::http::{HeaderMap, StatusCode};
use axum::{Extension, Json};
use std::sync::Arc;
use systemprompt::marketplace::managed::{ImportedSkills, SourceSpec, capture_skills};
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::request::require_write_origin;
use systemprompt_web_admin_core::state::ManagedState;
use systemprompt_web_admin_core::types::UserContext;

pub(crate) async fn capture_super_admin(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    headers: HeaderMap,
) -> AdminResult<(StatusCode, Json<ImportedSkills>)> {
    require_write_origin(&headers)?;
    let imported = capture(&user, &state).await?;
    Ok((StatusCode::CREATED, Json(imported)))
}

async fn capture(_user: &UserContext, state: &ManagedState) -> AdminResult<ImportedSkills> {
    let root = systemprompt_web_admin_core::request::get_services_path()?;
    let source = state
        .repository
        .register_source(
            &state.owner,
            "super-admin-services",
            &SourceSpec::LocalTree {
                root: root.to_string_lossy().into_owned(),
            },
        )
        .await?;
    let captured = tokio::task::spawn_blocking(move || {
        capture_skills(
            &root,
            &[
                "admin_daily_brief".to_owned(),
                "admin_critical_projects".to_owned(),
                "admin_ai_usage".to_owned(),
                "systemprompt_cli".to_owned(),
            ],
        )
    })
    .await
    .map_err(AdminError::internal)??;
    let imported = state
        .repository
        .import_skills(&state.owner, &source, &captured, None)
        .await?;
    Ok(imported)
}
