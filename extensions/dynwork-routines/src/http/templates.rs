//! `GET /templates` and `GET /templates/{id}/revisions/{revision}`.
//!
//! A person sees a published template when core's access resolution admits at
//! least one of its anchor plugins, exactly as the bridge manifest is
//! filtered: the subject's roles from `find_user_access_profile`, its
//! attributes from the services' subject sources, and the plugin →
//! marketplace parent chain of the services config. `links` names every
//! template the person has a live task of, visible or not. The answer carries
//! an `ETag` over its bytes, and `If-None-Match` with it answers `304`.

use std::collections::{BTreeSet, HashSet};
use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use dynwork_relay::auth::AuthError;
use sha2::{Digest, Sha256};
use systemprompt::governance::authz::{
    AccessControlRepository, BulkKeepQuery, ChainSources, EntityKind, ParentChainIndex, allowed_ids,
};
use systemprompt::identifiers::UserId;
use systemprompt_web_admin_core::repositories::principal::role::find_user_access_profile;

use crate::error::RoutinesError;
use crate::ids::TemplateId;
use crate::state::RoutinesState;
use crate::store::{self, StoreError, TemplateRecord};
use crate::template::{LinkState, TemplateState, TemplateView};
use crate::view::{LinkView, TemplatesPage};

pub async fn list(
    State(state): State<RoutinesState>,
    headers: HeaderMap,
) -> Result<Response, RoutinesError> {
    let desktop = super::caller(&state, &headers)?;
    let templates = store::list_templates(&state.pool).await?;
    let linked = store::list_linked_template_ids(&state.pool, &desktop.user).await?;
    let anchors: Vec<String> = templates
        .iter()
        .filter(|template| {
            template.state == TemplateState::Published || linked.contains(&template.id)
        })
        .flat_map(|template| template.plugins.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let allowed = visible_plugins(&state, &desktop.user, &anchors).await?;
    let visible = |template: &TemplateRecord| {
        template
            .plugins
            .iter()
            .any(|plugin| allowed.contains(plugin))
    };
    let links = templates
        .iter()
        .filter(|template| linked.contains(&template.id))
        .map(|template| LinkView {
            id: template.id.clone(),
            state: link_state(template.state, visible(template)),
            retire: template.retire_mode,
        })
        .collect();
    let page = TemplatesPage {
        templates: templates
            .into_iter()
            .filter(|template| template.state == TemplateState::Published && visible(template))
            .map(|template| TemplateView {
                id: template.id,
                revision: template.revision,
                mandatory: template.mandatory,
                body: template.body,
            })
            .collect(),
        links,
    };
    let body = serde_json::to_vec(&page)?;
    let etag = format!("\"{:x}\"", Sha256::digest(&body));
    if unchanged(&headers, &etag) {
        return Ok((StatusCode::NOT_MODIFIED, [(header::ETAG, etag)]).into_response());
    }
    let content_type = "application/json".to_owned();
    Ok((
        [(header::ETAG, etag), (header::CONTENT_TYPE, content_type)],
        body,
    )
        .into_response())
}

// Why: only someone who has, or had, a task linked to the template reads its
// revisions, their desktop's merge base; anyone else gets 404, as if the
// template did not exist.
pub async fn revision(
    State(state): State<RoutinesState>,
    Path((id, revision)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<TemplateView>, RoutinesError> {
    let desktop = super::caller(&state, &headers)?;
    let id = TemplateId::parse(&id).map_err(|error| {
        tracing::debug!(%error, "template id in the path is malformed");
        RoutinesError::NotFound
    })?;
    let revision = revision
        .parse::<i64>()
        .ok()
        .filter(|revision| *revision > 0)
        .ok_or(RoutinesError::NotFound)?;
    if !store::has_had_template(&state.pool, &desktop.user, &id).await? {
        return Err(RoutinesError::NotFound);
    }
    let found = store::find_template_revision(&state.pool, &id, revision)
        .await?
        .ok_or(RoutinesError::NotFound)?;
    Ok(Json(TemplateView {
        id: found.template_id,
        revision: found.revision,
        mandatory: found.mandatory,
        body: found.body,
    }))
}

async fn visible_plugins(
    state: &RoutinesState,
    user: &UserId,
    anchors: &[String],
) -> Result<HashSet<String>, RoutinesError> {
    let profile = find_user_access_profile(&state.pool, user)
        .await
        .map_err(StoreError::from)?
        .ok_or(AuthError::UnknownUser)?;
    let attributes = state.services.subject_attributes(user).await?;
    let repo = AccessControlRepository::from_pool(Arc::clone(&state.pool));
    let chains = ParentChainIndex::load(
        &repo,
        Arc::new(ChainSources::from_services(&state.services.config())),
    )
    .await?;
    let allowed = allowed_ids(
        &repo,
        BulkKeepQuery {
            user_id: user,
            roles: &profile.roles,
            kind: EntityKind::Plugin,
            ids: anchors,
            chains: &chains,
            attributes: &attributes,
            dimensions: state.services.dimensions(),
        },
    )
    .await?;
    Ok(allowed)
}

const fn link_state(state: TemplateState, visible: bool) -> LinkState {
    match (state, visible) {
        (TemplateState::Retired, _) => LinkState::Retired,
        (TemplateState::Closed, true) => LinkState::Closed,
        (TemplateState::Published, true) => LinkState::Published,
        _ => LinkState::NoAccess,
    }
}

fn unchanged(headers: &HeaderMap, etag: &str) -> bool {
    headers
        .get(header::IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|tags| tags.split(',').any(|tag| tag.trim() == etag))
}
