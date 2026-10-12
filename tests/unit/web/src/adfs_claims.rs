//! The pure half of the Entra ID SSO login: assertion claim shapes, the
//! group→role map, and the config accessors — everything that decides a login
//! without a live tenant.
//!
//! Which GROUP a person lands in is not decided here. The directory's group
//! names are mapped onto DB groups and projects by
//! `services/web/config/groups.yaml`, so this config only says what roles a
//! membership additionally mints.

use std::collections::BTreeMap;

use systemprompt_web_admin_identity::{AdfsConfig, AssertionClaims};

const EMAIL: &str = "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/emailaddress";
const NAME: &str = "http://schemas.microsoft.com/identity/claims/displayname";
const GROUPS: &str = "http://schemas.microsoft.com/ws/2008/06/identity/claims/groups";
const OVERAGE: &str = "http://schemas.microsoft.com/claims/groups.link";
const ADMINS_GUID: &str = "6f1c2a9e-0b7d-4e33-9a51-2c4d8e7f1a10";
const CONSULTANTS_GUID: &str = "b2e4d6f8-1a3c-4e5f-8a7b-9c0d1e2f3a4b";

fn config() -> AdfsConfig {
    let mut cfg = AdfsConfig::disabled();
    cfg.enabled = true;
    cfg.entity_id = "https://app.example.com/saml/metadata".to_owned();
    cfg.acs_url = "https://app.example.com/admin/auth/adfs/acs".to_owned();
    cfg.idp_metadata_xml = "<EntityDescriptor/>".to_owned();
    cfg.group_roles = BTreeMap::from([
        (
            "Systemprompt-Admins".to_owned(),
            vec!["admin".to_owned(), "user".to_owned()],
        ),
        ("Systemprompt-Commerce".to_owned(), vec!["user".to_owned()]),
        ("Systemprompt-Core".to_owned(), vec!["user".to_owned()]),
        (
            ADMINS_GUID.to_owned(),
            vec!["platform_admin".to_owned(), "user".to_owned()],
        ),
    ]);
    cfg
}

fn claims(attrs: &[(&str, &[&str])]) -> AssertionClaims {
    AssertionClaims {
        name_id: None,
        name_id_is_email: false,
        attributes: attrs
            .iter()
            .map(|(n, vs)| {
                (
                    (*n).to_owned(),
                    vs.iter().map(|v| (*v).to_owned()).collect(),
                )
            })
            .collect(),
    }
}

#[test]
fn every_value_under_the_group_attribute_is_a_group() {
    let c = claims(&[(GROUPS, &["Domain Users", "Systemprompt-Core"])]);
    assert_eq!(
        c.values(GROUPS),
        vec!["Domain Users".to_owned(), "Systemprompt-Core".to_owned()]
    );
    assert!(c.values("http://temp/groups").is_empty());
}

#[test]
fn groups_are_read_from_the_configured_attribute_name() {
    let mut cfg = config();
    cfg.groups_attribute = "http://temp/groups".to_owned();
    let c = claims(&[("http://temp/groups", &["Systemprompt-Core"])]);
    assert_eq!(
        cfg.roles_for_groups(&c.values(&cfg.groups_attribute)),
        vec!["user".to_owned()]
    );
}

#[test]
fn roles_are_the_deduplicated_union_over_mapped_groups() {
    let cfg = config();
    let roles = cfg.roles_for_groups(&[
        "Systemprompt-Core".to_owned(),
        "Systemprompt-Admins".to_owned(),
        "Unrelated-Group".to_owned(),
    ]);
    assert_eq!(roles, vec!["admin".to_owned(), "user".to_owned()]);
    assert!(
        cfg.roles_for_groups(&["Unrelated-Group".to_owned()])
            .is_empty()
    );
}

#[test]
fn an_unmapped_group_mints_no_role() {
    let mut cfg = config();
    cfg.group_roles.remove("Systemprompt-Core");
    assert!(
        cfg.roles_for_groups(&["Systemprompt-Core".to_owned()])
            .is_empty(),
        "the DB mapping decides membership; the login proceeds as a plain user"
    );
}

#[test]
fn login_email_prefers_the_email_attribute_then_an_email_name_id() {
    let cfg = config();
    let with_attr = claims(&[(EMAIL, &[" Person@Example.TEST "])]);
    assert_eq!(
        with_attr.login_email(&cfg).as_deref(),
        Some("person@example.test")
    );

    let mut by_name_id = claims(&[]);
    by_name_id.name_id = Some("Person@Example.test".to_owned());
    by_name_id.name_id_is_email = true;
    assert_eq!(
        by_name_id.login_email(&cfg).as_deref(),
        Some("person@example.test")
    );

    let mut opaque_name_id = claims(&[]);
    opaque_name_id.name_id = Some("S-1-5-21-1234".to_owned());
    assert_eq!(opaque_name_id.login_email(&cfg), None);

    let nothing = claims(&[(NAME, &["Ana García"])]);
    assert_eq!(
        nothing.login_email(&cfg),
        None,
        "a display name is not an address"
    );
}

#[test]
fn external_sub_is_the_name_id_when_present_else_the_email() {
    let mut with_name_id = claims(&[]);
    with_name_id.name_id = Some("persistent-xyz".to_owned());
    assert_eq!(with_name_id.external_sub("a@x.test"), "persistent-xyz");
    assert_eq!(claims(&[]).external_sub("a@x.test"), "a@x.test");
}

#[test]
fn display_name_comes_from_the_name_attribute() {
    let cfg = config();
    let named = claims(&[(NAME, &[" Ana García "])]);
    assert_eq!(named.display_name(&cfg).as_deref(), Some("Ana García"));
    assert_eq!(claims(&[]).display_name(&cfg), None);
}

#[test]
fn the_config_accessors_and_defaults_are_the_secure_ones() {
    let cfg = config();
    assert!(cfg.is_usable());
    assert!(cfg.email_allowed("someone@dynapps.es"));
    assert!(cfg.email_allowed("someone@wearedynapps.com"));
    assert!(!cfg.email_allowed("someone@gmail.com"));
    assert!(!AdfsConfig::disabled().is_usable());

    let parsed: AdfsConfig = serde_yaml::from_str(
        "enabled: true\nentity_id: https://x/saml/metadata\nacs_url: https://x/cb\nidp_metadata_path: m.xml\n",
    )
    .expect("minimal config parses");
    assert_eq!(parsed.groups_attribute, GROUPS);
    assert_eq!(parsed.email_attribute, EMAIL);
    assert_eq!(parsed.name_attribute, NAME);
    assert_eq!(parsed.idp_metadata_url, None);
    assert!(parsed.deny_without_group);
    assert!(parsed.allow_idp_initiated);
    assert!(!parsed.auto_provision);
    assert!(
        !parsed.is_usable(),
        "usable only once the loader has read the metadata file"
    );

    let unknown = serde_yaml::from_str::<AdfsConfig>(
        "enabled: true\nentity_id: a\nacs_url: r\nidp_metadata_path: m\nclient_id: x\n",
    );
    assert!(
        unknown.is_err(),
        "an OIDC-era key must not be silently ignored"
    );
}

#[test]
fn entra_group_object_ids_map_through_group_roles() {
    let cfg = config();
    let c = claims(&[(GROUPS, &[ADMINS_GUID, CONSULTANTS_GUID])]);
    let groups = c.values(&cfg.groups_attribute);
    assert_eq!(
        groups,
        vec![ADMINS_GUID.to_owned(), CONSULTANTS_GUID.to_owned()]
    );
    assert_eq!(
        cfg.roles_for_groups(&groups),
        vec!["platform_admin".to_owned(), "user".to_owned()]
    );
    assert!(
        cfg.roles_for_groups(&[CONSULTANTS_GUID.to_owned()])
            .is_empty()
    );
}

#[test]
fn a_groups_overage_link_is_detected_with_or_without_a_groups_claim() {
    let link = "https://graph.windows.net/tenant/users/oid/getMemberObjects";
    assert_eq!(
        systemprompt_web_admin_identity::GROUPS_OVERAGE_ATTRIBUTE,
        OVERAGE
    );
    let overage_only = claims(&[(EMAIL, &["a@dynapps.es"]), (OVERAGE, &[link])]);
    assert!(overage_only.groups_overage());
    assert!(overage_only.values(GROUPS).is_empty());
    let both = claims(&[(GROUPS, &[ADMINS_GUID]), (OVERAGE, &[link])]);
    assert!(both.groups_overage());
    assert!(!claims(&[(GROUPS, &[ADMINS_GUID])]).groups_overage());
}

#[test]
fn a_metadata_url_parses_and_makes_the_config_usable_without_a_file() {
    let parsed: AdfsConfig = serde_yaml::from_str(
        "enabled: true\nentity_id: https://x/saml/metadata\nacs_url: https://x/admin/auth/adfs/acs\n\
         idp_metadata_url: https://login.microsoftonline.com/00000000-0000-4000-8000-000000000e17/federationmetadata/2007-06/federationmetadata.xml?appid=11111111-2222-4333-8444-555555555555\n",
    )
    .expect("a URL-only config parses");
    assert!(parsed.idp_metadata_path.is_empty());
    assert!(
        parsed
            .idp_metadata_url
            .as_deref()
            .is_some_and(|u| u.starts_with("https://login.microsoftonline.com/"))
    );
    assert!(
        parsed.is_usable(),
        "the metadata is fetched at run time, so no file is needed"
    );

    let mut off = parsed;
    off.enabled = false;
    assert!(!off.is_usable());
}
