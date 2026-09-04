//! View-model assembly for the user roster and detail pages.
//!
//! Pure transforms from repository rows into the serde structs the `users` /
//! `user-detail` templates render: marketplace resolution, per-user
//! enrichment, and project grouping. `UsersPageData` / `UserDetailPageData`
//! are fully self-contained typed structs; callers in `mod.rs` populate them
//! directly and hand them to `render_typed_page` with no post-hoc value
//! mutation.

use crate::repositories;

use crate::types::Project;

use super::super::types::{
    EnrichedUserView, ProjectGroup, UserAiUsageView, UserConversationView, UserMarketplaceRef,
    UserModelShareView,
};
use crate::handlers::ssr::entity_urls::context_detail_url;
use crate::handlers::ssr::format::{format_cost, format_token_total};
use crate::handlers::ssr::transcript_view::short_id;
use crate::repositories::analytics::contexts_list::ContextListItem;
use crate::repositories::users::usage::{ModelShare, UserGatewayUsage};

fn local_stamp(t: chrono::DateTime<chrono::Utc>) -> String {
    t.with_timezone(&chrono::Local)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

pub(super) fn ai_usage_view(summary: &UserGatewayUsage, models: &[ModelShare]) -> UserAiUsageView {
    UserAiUsageView {
        requests: summary.requests,
        conversations: summary.conversations,
        failed: summary.failed,
        tokens: summary.tokens,
        tokens_display: format_token_total(summary.tokens),
        cost_display: format_cost(summary.cost_microdollars),
        first_request_at: summary.first_request_at.map(local_stamp),
        last_request_at: summary.last_request_at.map(local_stamp),
        has_top_models: !models.is_empty(),
        top_models: models
            .iter()
            .map(|m| UserModelShareView {
                model: m.model.clone(),
                requests: m.requests,
                tokens_display: format_token_total(m.tokens),
                cost_display: format_cost(m.cost_microdollars),
            })
            .collect(),
    }
}

// Why: the title falls back to the opening user turn and then to the short id —
// a gateway conversation has no name unless `ensure_context` managed to write
// one, and core swallows that failure, so an unnamed row is the common case.
pub(super) fn conversation_view(item: &ContextListItem) -> UserConversationView {
    let short = short_id(item.context_id.as_str());
    let title = item
        .name
        .clone()
        .or_else(|| item.summary.clone())
        .unwrap_or_else(|| format!("Conversation {short}"));
    UserConversationView {
        context_id: item.context_id.clone(),
        context_id_short: short,
        title,
        summary: item.summary.clone(),
        model: item.model.clone(),
        request_count: item.request_count,
        message_count: item.message_count,
        error_count: item.error_count,
        has_errors: item.error_count > 0,
        cost_display: format_cost(item.total_cost_microdollars),
        first_seen_local: item.first_request_at.map(local_stamp),
        last_seen_local: item.last_activity_at.map(local_stamp),
        context_url: context_detail_url(&item.context_id),
    }
}

fn freshness_for(ts: Option<chrono::DateTime<chrono::Utc>>) -> &'static str {
    ts.map_or("never", |t| {
        let age = chrono::Utc::now() - t;
        if age < chrono::Duration::minutes(5) {
            "fresh"
        } else if age < chrono::Duration::hours(1) {
            "idle"
        } else {
            "stale"
        }
    })
}

// Why: Resolve effective marketplaces for a user: every YAML-defined
// marketplace is granted by default, then `access_control_rules` rows (matching
// the user's id or AD group) are applied as allow/deny overrides.
pub(super) fn resolve_marketplaces(
    yaml_defaults: &[(String, String)],
    overrides: &[&repositories::users::aggregates::UserMarketplaceOverride],
) -> Vec<UserMarketplaceRef> {
    let mut entries: Vec<UserMarketplaceRef> = yaml_defaults
        .iter()
        .map(|(id, name)| UserMarketplaceRef {
            id: id.clone(),
            name: name.clone(),
            source: "default",
        })
        .collect();

    for ovr in overrides {
        match ovr.access.as_str() {
            "allow" if !entries.iter().any(|e| e.id == ovr.entity_id) => {
                let name = yaml_defaults
                    .iter()
                    .find(|(id, _)| id == &ovr.entity_id)
                    .map_or_else(|| ovr.entity_id.clone(), |(_, n)| n.clone());
                entries.push(UserMarketplaceRef {
                    id: ovr.entity_id.clone(),
                    name,
                    source: "override",
                });
            },
            "deny" => entries.retain(|e| e.id != ovr.entity_id),
            _ => {},
        }
    }
    entries
}

pub(super) fn enrich_users(
    users: &[crate::types::UserSummary],
    aggregates: &[repositories::users::aggregates::UserManagementAggregate],
    runtime: &[repositories::users::queries::UserRuntimeAggregate],
    overrides: &[repositories::users::aggregates::UserMarketplaceOverride],
    yaml_marketplaces: &[(String, String)],
) -> Vec<EnrichedUserView> {
    let agg_map: std::collections::HashMap<
        &str,
        &repositories::users::aggregates::UserManagementAggregate,
    > = aggregates.iter().map(|a| (a.user_id.as_str(), a)).collect();
    let rt_map: std::collections::HashMap<
        &str,
        &repositories::users::queries::UserRuntimeAggregate,
    > = runtime.iter().map(|r| (r.user_id.as_str(), r)).collect();
    let mut ovr_map: std::collections::HashMap<
        &str,
        Vec<&repositories::users::aggregates::UserMarketplaceOverride>,
    > = std::collections::HashMap::new();
    for o in overrides {
        ovr_map.entry(o.user_id.as_str()).or_default().push(o);
    }

    users
        .iter()
        .map(|u| {
            let agg = agg_map.get(u.user_id.as_str());
            let rt = rt_map.get(u.user_id.as_str());
            let device_freshness =
                freshness_for(rt.and_then(|r| r.newest_device_seen_at)).to_owned();
            let user_overrides = ovr_map.get(u.user_id.as_str()).cloned().unwrap_or_default();
            let marketplaces = resolve_marketplaces(yaml_marketplaces, &user_overrides);
            EnrichedUserView {
                user_id: u.user_id.clone(),
                display_name: u.display_name.clone(),
                email: u.email.as_ref().map(ToString::to_string),
                roles: u.roles.clone(),
                is_active: u.is_active,
                last_active: u.last_active.map(|t| t.to_rfc3339()).unwrap_or_default(),
                total_events: u.total_events,
                last_tool: u.last_tool.clone(),
                custom_skills_count: u.custom_skills_count,
                preferred_client: u.preferred_client.clone(),
                prompts: u.prompts,
                sessions: u.sessions,
                bytes: u.bytes,
                logins: u.logins,
                project: agg
                    .and_then(|a| Project::parse_opt(a.project.as_deref()))
                    .map_or_else(String::new, |p| p.label().to_owned()),
                created_at: agg.map(|a| a.created_at.to_rfc3339()).unwrap_or_default(),
                marketplaces,
                assigned_skills_count: agg.map_or(0, |a| a.assigned_skills_count),
                devices_count: agg.map_or(0, |a| a.devices_count),
                connected_agents: rt.map_or(0, |r| r.connected_agents),
                total_agents: rt.map_or(0, |r| r.total_agents),
                lifetime_tokens: rt.map_or(0, |r| r.lifetime_tokens),
                device_freshness,
            }
        })
        .collect()
}

// Why: Group enriched users by project, alphabetical; users the directory
// has not placed in a project go last as "Unassigned".
pub(super) fn group_by_project(users: Vec<EnrichedUserView>) -> Vec<ProjectGroup> {
    let mut buckets: std::collections::BTreeMap<String, Vec<EnrichedUserView>> =
        std::collections::BTreeMap::new();
    for u in users {
        let key = if u.project.is_empty() {
            "Unassigned".to_owned()
        } else {
            u.project.clone()
        };
        buckets.entry(key).or_default().push(u);
    }

    let mut groups: Vec<ProjectGroup> = buckets
        .into_iter()
        .map(|(project, mut users)| {
            users.sort_by(|a, b| {
                let an = a.display_name.as_deref().unwrap_or(a.user_id.as_str());
                let bn = b.display_name.as_deref().unwrap_or(b.user_id.as_str());
                an.to_lowercase().cmp(&bn.to_lowercase())
            });
            let total_tokens = users.iter().map(|u| u.lifetime_tokens).sum();
            let total_sessions = users.iter().map(|u| u.sessions).sum();
            ProjectGroup {
                user_count: users.len(),
                total_tokens,
                total_sessions,
                project,
                users,
            }
        })
        .collect();

    groups.sort_by(|a, b| {
        let rank = |name: &str| u8::from(name == "Unassigned");
        rank(&a.project)
            .cmp(&rank(&b.project))
            .then_with(|| a.project.to_lowercase().cmp(&b.project.to_lowercase()))
    });
    groups
}
