//! The server-side authority and attribution rules, which apply identically to
//! a person calling with their linked Odoo account and to a farm service
//! account calling with its machine credential.

use systemprompt::identifiers::UserId;
use systemprompt_mcp_odoo::audit::{model_for_tool, record_ids};
use systemprompt_mcp_odoo::caller::{
    Caller, DESTRUCTIVE_ROLE, farm_run_id, needs_destructive_authority,
};
use systemprompt_mcp_odoo::client::crud::{as_html, with_footer};
use systemprompt_mcp_odoo::credential::stored_credential;
use systemprompt_mcp_odoo::error::OdooError;
use systemprompt_mcp_odoo::server::activity::scheduled_activity_id;
use systemprompt_mcp_odoo::server::gate::{
    action_phrase, is_write_tool, refusal_reason, settle_reason,
};
use systemprompt_mcp_odoo::tools::{GUARDED_TOOLS, TOOL_GDPR_ERASE_INVENTORY};
use systemprompt_web_admin_platform::machine_credentials::{
    OdooCredentialLookup, OdooMachineCredential,
};

fn person(roles: &[&str]) -> Caller {
    Caller {
        user_id: UserId::new("person-1".to_owned()),
        label: "Pat Person <pat@dynapps.test>".to_owned(),
        roles: roles.iter().map(|r| (*r).to_owned()).collect(),
        run_id: None,
    }
}

fn service_account(roles: &[&str]) -> Caller {
    Caller {
        user_id: UserId::new("svc-odoo-farm".to_owned()),
        label: "svc-odoo-farm@dynapps.test".to_owned(),
        roles: roles.iter().map(|r| (*r).to_owned()).collect(),
        run_id: farm_run_id(Some("farm:run-42")),
    }
}

#[test]
fn a_person_without_the_role_is_refused_and_with_it_allowed() {
    assert!(!person(&["user"]).may_act_destructively());
    assert!(person(&["user", DESTRUCTIVE_ROLE]).may_act_destructively());
}

#[test]
fn a_service_account_without_the_role_is_refused_and_with_it_allowed() {
    assert!(!service_account(&["user", "service_account"]).may_act_destructively());
    assert!(service_account(&["user", DESTRUCTIVE_ROLE]).may_act_destructively());
}

#[test]
fn an_administrator_needs_no_extra_role() {
    assert!(person(&["admin"]).may_act_destructively());
    assert!(service_account(&["super_admin"]).may_act_destructively());
}

#[test]
fn every_guarded_tool_and_the_inventory_need_destructive_authority() {
    for tool in GUARDED_TOOLS {
        assert!(needs_destructive_authority(tool), "{tool}");
    }
    assert!(needs_destructive_authority(TOOL_GDPR_ERASE_INVENTORY));
    assert!(!needs_destructive_authority("crm_lead_update"));
    assert!(!needs_destructive_authority("crm_lead_search"));
}

#[test]
fn the_refusal_names_the_role_and_says_nothing_changed() {
    let reason = refusal_reason("partner_delete");
    assert!(reason.contains(DESTRUCTIVE_ROLE), "{reason}");
    assert!(
        reason.contains("nothing in Odoo was read or changed"),
        "{reason}"
    );
}

#[test]
fn the_farm_run_id_comes_only_from_a_farm_purpose() {
    assert_eq!(farm_run_id(Some("farm:run-42")).as_deref(), Some("run-42"));
    assert_eq!(farm_run_id(Some("workflow:abc")), None);
    assert_eq!(farm_run_id(Some("farm:  ")), None);
    assert_eq!(farm_run_id(None), None);
}

#[test]
fn the_footer_names_the_caller_and_the_run() {
    assert_eq!(
        service_account(&[]).footer(),
        "via Dynapps · svc-odoo-farm@dynapps.test · farm run run-42"
    );
    assert_eq!(
        person(&[]).footer(),
        "via Dynapps · Pat Person <pat@dynapps.test>"
    );
}

#[test]
fn a_chatter_post_carries_the_footer_when_one_is_set() {
    assert_eq!(
        with_footer("<p>Qualified.</p>", Some("via Dynapps · x")),
        "<p>Qualified.</p><p><em>via Dynapps · x</em></p>"
    );
    assert_eq!(with_footer("<p>Qualified.</p>", None), "<p>Qualified.</p>");
}

#[test]
fn the_audit_row_names_the_model_and_the_record_ids() {
    assert_eq!(model_for_tool("crm_lead_delete"), "crm.lead");
    assert_eq!(model_for_tool("partner_anonymise"), "res.partner");
    assert_eq!(model_for_tool("gdpr_erase_inventory"), "res.partner");
    let args = serde_json::json!({ "id": 41, "reason": "dup" });
    assert_eq!(record_ids(args.as_object()), vec![41]);
    assert!(record_ids(None).is_empty());
}

#[test]
fn reads_are_not_write_tools_and_updates_are() {
    assert!(is_write_tool("crm_lead_update"));
    assert!(is_write_tool("partner_anonymise"));
    assert!(!is_write_tool("crm_lead_search"));
    assert!(!is_write_tool(TOOL_GDPR_ERASE_INVENTORY));
}

fn stored() -> OdooMachineCredential {
    OdooMachineCredential {
        url: "https://odoo.example".to_owned(),
        db: "demo".to_owned(),
        login: "farm-bot".to_owned(),
        api_key: "k".to_owned(),
    }
}

#[test]
fn each_kind_of_caller_without_a_credential_gets_its_own_remedy() {
    let person =
        stored_credential(OdooCredentialLookup::PersonUnlinked, None).expect_err("unlinked person");
    assert!(person.to_string().contains("/admin/connectors"), "{person}");

    let account = stored_credential(OdooCredentialLookup::ServiceAccountMissing, None)
        .expect_err("bare service account");
    assert!(
        account
            .to_string()
            .contains("service account has no Odoo credential"),
        "{account}"
    );
}

#[test]
fn a_service_account_key_is_spent_only_for_a_declared_purpose() {
    let refused = stored_credential(OdooCredentialLookup::ServiceAccount(stored()), None);
    assert!(matches!(refused, Err(OdooError::AccessDenied(_))));

    let farm = stored_credential(
        OdooCredentialLookup::ServiceAccount(stored()),
        Some("farm:run-42"),
    )
    .expect("a farm run may spend it");
    assert_eq!(farm.login, "farm-bot");
}

#[test]
fn a_person_uses_their_linked_account_without_a_purpose() {
    let linked = stored_credential(OdooCredentialLookup::Person(stored()), None)
        .expect("a linked person signs in as themselves");
    assert_eq!(linked.db, "demo");
}

#[test]
fn plain_chatter_text_is_escaped_into_a_paragraph() {
    assert_eq!(
        as_html("Budget <ok> & signed\nNext: demo"),
        "<p>Budget &lt;ok&gt; &amp; signed<br>Next: demo</p>"
    );
    assert_eq!(as_html("<p>already html</p>"), "<p>already html</p>");
    assert_eq!(
        with_footer("Qualified.", Some("via Dynapps · x")),
        "<p>Qualified.</p><p><em>via Dynapps · x</em></p>"
    );
}

#[test]
fn the_attribution_note_says_what_changed() {
    let args =
        serde_json::json!({ "id": 41, "fields": { "priority": "2", "tag_ids": [[6, 0, [12]]] } });
    assert_eq!(
        action_phrase("crm_lead_update", args.as_object()),
        "Updated priority, tags"
    );
    let archive = serde_json::json!({ "id": 9, "active": false });
    assert_eq!(
        action_phrase("partner_update", archive.as_object()),
        "Updated active"
    );
    assert_eq!(action_phrase("crm_lead_mark_won", None), "Marked won");
}

#[test]
fn activity_schedule_results_yield_the_new_activity_id() {
    assert_eq!(scheduled_activity_id(&serde_json::json!([57])), Some(57));
    assert_eq!(scheduled_activity_id(&serde_json::json!(57)), Some(57));
    assert_eq!(scheduled_activity_id(&serde_json::json!(true)), None);
}

#[test]
fn a_failed_call_carries_the_odoo_error_into_the_audit_reason() {
    assert_eq!(settle_reason(None), "executed against Odoo");
    assert_eq!(settle_reason(Some("  \n ")), "Odoo call failed");
    let reason = settle_reason(Some(
        "You are not allowed to delete 'Lead/Opportunity' (crm.lead) records.\n\nThis \
         operation is allowed for the following groups:\n\t- Sales/Administrator",
    ));
    assert!(
        reason.starts_with("Odoo call failed: You are not allowed to delete"),
        "{reason}"
    );
    assert!(reason.contains("Sales/Administrator"), "{reason}");
    assert!(!reason.contains('\n'), "the reason is one line: {reason}");
    let long = settle_reason(Some(&"x".repeat(5_000)));
    assert!(long.chars().count() < 900, "the reason is clipped");
}
