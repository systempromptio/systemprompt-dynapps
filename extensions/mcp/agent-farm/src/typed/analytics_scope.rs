//! How an analytics tool's `scope` and `project` words become the user list
//! a dashboard query binds. `mine` binds the caller's own user id; `project`
//! resolves one project the caller belongs to into its member list
//! (exclusive attribution, so project figures partition the instance);
//! `all` is the instance and is refused without an administrator role.
//!
//! Nothing here can name another person's user id.

use systemprompt_web_admin_analytics::repositories::analytics::site::{
    SiteScope, resolve_site_scope,
};
use systemprompt_web_admin_core::repositories::scope::{Attribution, Scope};
use systemprompt_web_admin_core::util::time_range::TimeRange;
use systemprompt_web_shared::ProjectId;

use super::{AnalyticsInput, ToolCtx, ToolError, ToolResult, window};
use crate::server::Viewer;

pub fn analytics_scope(viewer: &Viewer, scope: &str, project: &str) -> ToolResult<Scope> {
    match scope.trim() {
        "" | "mine" => Ok(Scope::User(viewer.user_id.clone())),
        "project" => Ok(Scope::Project(ProjectId::new(chosen_project(
            viewer, project,
        )?))),
        "all" => {
            if viewer.is_admin {
                Ok(Scope::All)
            } else {
                Err(ToolError::Forbidden(
                    "`scope: all` spans every user and needs an administrator role".to_owned(),
                ))
            }
        },
        other => Err(ToolError::Invalid(format!(
            "`scope` must be `mine`, `project` or `all` (got {other:?})"
        ))),
    }
}

fn chosen_project(viewer: &Viewer, project: &str) -> ToolResult<String> {
    let project = project.trim();
    if project.is_empty() {
        return match viewer.project_ids.as_slice() {
            [only] => Ok(only.clone()),
            [] => Err(ToolError::Invalid(
                "you belong to no project; use `scope: mine`".to_owned(),
            )),
            many => Err(ToolError::Invalid(format!(
                "set `project` to one of: {}",
                many.join(", ")
            ))),
        };
    }
    if !viewer.is_admin && !viewer.is_member_of(project) {
        return Err(ToolError::Forbidden(format!(
            "you are not a member of project {project}"
        )));
    }
    Ok(project.to_owned())
}

pub(crate) async fn resolve(
    ctx: &ToolCtx<'_>,
    input: &AnalyticsInput,
) -> ToolResult<(TimeRange, SiteScope)> {
    let range = window(&input.window)?;
    let scope = analytics_scope(&ctx.viewer, &input.scope, &input.project)?;
    let site = resolve_site_scope(ctx.pool()?, &scope, Attribution::Exclusive).await?;
    Ok((range, site))
}
