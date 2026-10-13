//! The identity gate every Entra ID sign-in passes, now core's
//! `SsoIdentityPolicy::gate`: allow-listed domain, groups overage, no group,
//! and the group→role map with its default.
//!
//! Which GROUP a person lands in is not decided here. The directory's group
//! names are mapped onto DB groups and projects by
//! `services/web/config/groups.yaml`, so the policy only says what roles a
//! membership additionally mints.

use std::collections::BTreeMap;

use systemprompt::sso::{ExternalIdentity, SsoIdentityPolicy, SsoRefusal};

const ADMINS_GUID: &str = "6f1c2a9e-0b7d-4e33-9a51-2c4d8e7f1a10";
const CONSULTANTS_GUID: &str = "b2e4d6f8-1a3c-4e5f-8a7b-9c0d1e2f3a4b";

fn strings(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| (*s).to_owned()).collect()
}

fn policy() -> SsoIdentityPolicy {
    SsoIdentityPolicy {
        allowed_email_domains: strings(&["dynapps.es", "wearedynapps.com"]),
        deny_without_group: true,
        group_roles: BTreeMap::from([
            (
                "Systemprompt-Admins".to_owned(),
                strings(&["admin", "user"]),
            ),
            ("Systemprompt-Commerce".to_owned(), strings(&["user"])),
            ("Systemprompt-Core".to_owned(), strings(&["user"])),
            (
                ADMINS_GUID.to_owned(),
                strings(&["platform_admin", "admin", "user"]),
            ),
        ]),
        ..SsoIdentityPolicy::default()
    }
}

fn identity(groups: &[&str]) -> ExternalIdentity {
    ExternalIdentity {
        subject: "sub-1".to_owned(),
        email: Some("a@dynapps.es".to_owned()),
        email_verified: true,
        display_name: None,
        groups: strings(groups),
        groups_overage: false,
        tenant_id: None,
        assertion_id: None,
        assertion_expires_at: None,
        raw_claims: serde_json::Value::Null,
    }
}

#[test]
fn roles_are_the_deduplicated_union_over_mapped_groups() {
    let p = policy();
    let mut roles = p.roles_for_groups(&strings(&[
        "Systemprompt-Core",
        "Systemprompt-Admins",
        "Unrelated-Group",
    ]));
    roles.sort();
    assert_eq!(roles, strings(&["admin", "user"]));
    assert!(
        p.roles_for_groups(&strings(&["Unrelated-Group"]))
            .is_empty()
    );
}

#[test]
fn an_unmapped_group_signs_in_with_the_default_roles() {
    let gated = policy()
        .gate(&identity(&["Unrelated-Group"]), "a@dynapps.es")
        .expect("a group the map does not name is not a refusal");
    assert_eq!(gated.roles, strings(&["user"]));
    assert_eq!(gated.display_name, "a@dynapps.es", "the email stands in");
}

#[test]
fn entra_group_object_ids_map_and_still_grant_the_admin_role() {
    let gated = policy()
        .gate(&identity(&[ADMINS_GUID, CONSULTANTS_GUID]), "a@dynapps.es")
        .expect("admitted");
    assert!(
        gated.roles.iter().any(|r| r == "admin"),
        "core mints admin permission from `admin` alone, so the map must name it"
    );
    assert_eq!(gated.groups, strings(&[ADMINS_GUID, CONSULTANTS_GUID]));
}

#[test]
fn every_refusal_reason_is_reached() {
    let p = policy();
    assert_eq!(
        p.gate(&identity(&["Systemprompt-Core"]), "a@gmail.com"),
        Err(SsoRefusal::DomainNotAllowed)
    );
    assert_eq!(
        p.gate(&identity(&[]), "a@dynapps.es"),
        Err(SsoRefusal::NoGroup)
    );
    let mut overage = identity(&[ADMINS_GUID]);
    overage.groups_overage = true;
    assert_eq!(
        p.gate(&overage, "a@dynapps.es"),
        Err(SsoRefusal::GroupOverage)
    );
    assert!(p.email_allowed("someone@wearedynapps.com"));
}

#[test]
fn a_regional_pattern_adds_to_the_exact_groups() {
    let mut p = policy();
    p.group_role_patterns = BTreeMap::from([(
        "Systemprompt-ProjectManagers-*".to_owned(),
        strings(&["project_manager", "user"]),
    )]);
    let mut roles = p.roles_for_groups(&strings(&[
        "Systemprompt-Admins",
        "Systemprompt-ProjectManagers-UK",
    ]));
    roles.sort();
    assert_eq!(roles, strings(&["admin", "project_manager", "user"]));
    assert!(
        p.roles_for_groups(&strings(&["Systemprompt-ProjectManagers-"]))
            .len()
            == 2
    );
    assert!(
        p.roles_for_groups(&strings(&["Other-ProjectManagers-UK"]))
            .is_empty()
    );
}
