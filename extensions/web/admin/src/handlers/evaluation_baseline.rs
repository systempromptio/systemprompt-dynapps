//! Capture the agreed four-skill baseline without changing active skills.

use super::evaluation_experiments::require_write_origin;
use crate::error::{AdminError, AdminResult};
use crate::routes::managed_state::ManagedState;
use crate::types::UserContext;
use axum::http::{HeaderMap, StatusCode};
use axum::{Extension, Json};
use std::sync::Arc;
use systemprompt::marketplace::managed::{ImportedSkills, SourceSpec, capture_skills};

pub(crate) async fn capture_super_admin(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    headers: HeaderMap,
) -> AdminResult<(StatusCode, Json<ImportedSkills>)> {
    require_write_origin(&headers)?;
    let imported = capture(&user, &state).await?;
    Ok((StatusCode::CREATED, Json(imported)))
}

pub(crate) async fn capture_super_admin_page(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    headers: HeaderMap,
) -> AdminResult<axum::response::Redirect> {
    if !user.is_admin {
        return Err(AdminError::Forbidden(
            "Administrator access required".to_owned(),
        ));
    }
    require_write_origin(&headers)?;
    capture(&user, &state).await?;
    Ok(axum::response::Redirect::to("/admin/analysis/versions"))
}

async fn capture(_user: &UserContext, state: &ManagedState) -> AdminResult<ImportedSkills> {
    let root = super::shared::get_services_path()?;
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

#[path = "evaluation_suite.rs"]
mod suite;
pub(crate) use suite::seed_suite;
