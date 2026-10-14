//! The tool-layer guardrail's pure rules: demo scope's default, the reason
//! requirement, the `demo-*` match, and the erasure overwrite values.

use systemprompt_mcp_odoo::guard::{
    Target, carries_demo_tag, check_demo_scope, irreversible_notice, parse_demo_scope,
    require_reason,
};
use systemprompt_mcp_odoo::server::guarded::erasure_shape::{
    LEAD_PII, PARTNER_PII, anonymised_lead_values, anonymised_partner_values, linked_leads_domain,
};

const LEAD: Target<'static> = Target {
    label: "lead",
    id: 41,
    tag_kind: "tag",
};

fn tags(names: &[&str]) -> Vec<String> {
    names.iter().map(|n| (*n).to_owned()).collect()
}

#[test]
fn demo_scope_is_on_unless_explicitly_turned_off() {
    assert!(
        parse_demo_scope(None),
        "an unset setting must not open real data"
    );
    assert!(parse_demo_scope(Some("")));
    assert!(parse_demo_scope(Some("true")));
    assert!(parse_demo_scope(Some("anything")));
    for off in ["false", "FALSE", "0", "no", " off "] {
        assert!(!parse_demo_scope(Some(off)), "{off}");
    }
}

#[test]
fn a_blank_reason_is_refused() {
    for blank in ["", "   ", "\n\t"] {
        let err = require_reason("crm_lead_delete", blank).expect_err("blank reason refused");
        assert!(err.message.contains("reason"), "{}", err.message);
    }
}

#[test]
fn a_reason_is_kept_trimmed() {
    assert_eq!(
        require_reason("partner_delete", "  duplicate of 12 ").expect("accepted"),
        "duplicate of 12"
    );
}

#[test]
fn only_a_demo_prefixed_tag_counts() {
    assert!(carries_demo_tag(&tags(&["vip", "demo-triage"])));
    assert!(carries_demo_tag(&tags(&["Demo-Dedup"])));
    assert!(!carries_demo_tag(&tags(&["demo", "customer", "nodemo-x"])));
    assert!(!carries_demo_tag(&[]));
}

#[test]
fn demo_scope_refuses_an_untagged_record_and_names_what_it_has() {
    let err = check_demo_scope(true, LEAD, &tags(&["customer"])).expect_err("refused");

    assert!(err.message.contains("lead 41"), "{}", err.message);
    assert!(err.message.contains("customer"), "{}", err.message);
    assert!(err.message.contains("ODOO_DEMO_SCOPE"), "{}", err.message);
}

#[test]
fn demo_scope_passes_a_demo_record_and_everything_when_off() {
    assert!(check_demo_scope(true, LEAD, &tags(&["demo-dedup"])).is_ok());
    assert!(check_demo_scope(false, LEAD, &[]).is_ok());
}

#[test]
fn the_notice_says_irreversible_and_carries_the_reason() {
    let notice = irreversible_notice("lead 41 was unlinked", "duplicate");
    assert!(notice.starts_with("IRREVERSIBLE"));
    assert!(notice.contains("\"duplicate\""));
}

#[test]
fn anonymising_a_partner_renames_it_and_clears_every_pii_field() {
    let values = anonymised_partner_values(9);

    assert_eq!(values["name"], "Anonymised 9");
    for key in PARTNER_PII {
        assert_eq!(values[key], false, "{key} must be cleared");
    }
}

#[test]
fn anonymising_a_lead_clears_its_contact_fields() {
    let values = anonymised_lead_values(9);

    assert_eq!(values["contact_name"], "Anonymised 9");
    for key in LEAD_PII.iter().filter(|k| **k != "contact_name") {
        assert_eq!(values[*key], false, "{key} must be cleared");
    }
}

#[test]
fn the_linked_lead_search_includes_archived_leads() {
    let domain = linked_leads_domain(9).to_string();
    assert!(domain.contains(r#"["active","=",false]"#), "{domain}");
}
