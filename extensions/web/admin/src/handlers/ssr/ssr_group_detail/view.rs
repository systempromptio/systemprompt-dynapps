//! The group Access tab: effective grant plus the group's own toggle state.
//!
//! Everything else on this page is shaped by `people_view`, which the project
//! pages share.

use systemprompt_web_admin_core::types::access_control::{AccessControlRule, AccessDecision};
use systemprompt_web_admin_core::types::groups::GroupMemberRow;
use systemprompt_web_shared::GroupId;

use super::super::people_view::{MemberContext, MemberInput, member_rows};
use super::super::types::{AccessRowView, AccessSectionView, MemberRowView};
use super::data::MembersData;

pub(super) fn group_member_rows(data: &MembersData, can_manage: bool) -> Vec<MemberRowView> {
    let inputs: Vec<MemberInput<'_>> = data.rows.iter().map(as_member_input).collect();
    member_rows(
        &inputs,
        &MemberContext {
            usage: &data.usage,
            active: &data.active,
            can_manage,
        },
    )
}

fn as_member_input(row: &GroupMemberRow) -> MemberInput<'_> {
    MemberInput {
        user_id: row.user_id.as_str(),
        display_name: row.display_name.as_deref(),
        email: row.email.as_deref(),
        sources: &row.sources,
        source_ad_groups: &row.source_ad_groups,
        valid_until: row.valid_until,
    }
}

pub(super) fn access_sections(
    resolved: Vec<systemprompt_web_admin_core::repositories::entitlement::MatrixSection>,
    rules: &[AccessControlRule],
    group_id: &GroupId,
) -> Vec<AccessSectionView> {
    resolved
        .into_iter()
        .map(|section| {
            let rows = section
                .rows
                .into_iter()
                .map(|row| AccessRowView {
                    state: own_rule_state(rules, &section.entity_type, &row.entity_id, group_id),
                    entity_type: section.entity_type.clone(),
                    entity_id: row.entity_id,
                    entity_name: row.entity_name,
                    effective: row.effective,
                    layer: row.source.layer,
                    detail: row.source.detail,
                })
                .collect();
            AccessSectionView {
                rows,
                entity_type: section.entity_type,
                label: section.label,
            }
        })
        .collect()
}

fn own_rule_state(
    rules: &[AccessControlRule],
    entity_type: &str,
    entity_id: &str,
    group_id: &GroupId,
) -> &'static str {
    rules
        .iter()
        .find(|r| {
            r.entity_type == entity_type
                && r.entity_id == entity_id
                && r.rule_type.as_str() == "group"
                && r.rule_value == group_id.as_str()
        })
        .map_or("inherit", |r| match r.access {
            AccessDecision::Allow => "allow",
            AccessDecision::Deny => "deny",
        })
}
