//! The pure half of the Entra ID SAML sign-in: how a verified assertion's
//! subject and attribute statement become core's `ExternalIdentity`, and the
//! config the provider is built from.
//!
//! What the identity is then allowed to do — domains, groups, roles — is
//! core's `SsoIdentityPolicy`, pinned in `sso_policy`.

use systemprompt_web_admin_identity::{AssertionClaims, EntraSamlConfig};

const EMAIL: &str = "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/emailaddress";
const NAME: &str = "http://schemas.microsoft.com/identity/claims/displayname";
const GROUPS: &str = "http://schemas.microsoft.com/ws/2008/06/identity/claims/groups";
const OVERAGE: &str = "http://schemas.microsoft.com/claims/groups.link";
const ADMINS_GUID: &str = "6f1c2a9e-0b7d-4e33-9a51-2c4d8e7f1a10";

fn config() -> EntraSamlConfig {
    let mut cfg = EntraSamlConfig::disabled();
    cfg.enabled = true;
    cfg.entity_id = "https://app.example.com/saml/metadata".to_owned();
    cfg.acs_url = "https://app.example.com/admin/auth/adfs/acs".to_owned();
    cfg.idp_metadata_xml = "<EntityDescriptor/>".to_owned();
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
    let c = claims(&[
        ("http://temp/groups", &["Systemprompt-Core"]),
        (GROUPS, &["x"]),
    ]);
    assert_eq!(
        c.to_external_identity(&cfg, None, None).groups,
        vec!["Systemprompt-Core".to_owned()]
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
fn the_external_identity_carries_every_field_the_driver_gates_on() {
    let cfg = config();
    let mut c = claims(&[
        (EMAIL, &["Ana@Dynapps.es"]),
        (NAME, &["Ana García"]),
        (GROUPS, &[ADMINS_GUID]),
    ]);
    c.name_id = Some("persistent-ana".to_owned());
    let expires = chrono::Utc::now();
    let identity = c.to_external_identity(&cfg, Some("_assertion-1".to_owned()), Some(expires));
    assert_eq!(identity.subject, "persistent-ana");
    assert_eq!(identity.email.as_deref(), Some("ana@dynapps.es"));
    assert!(
        identity.email_verified,
        "Entra ID asserts the directory address"
    );
    assert_eq!(identity.display_name.as_deref(), Some("Ana García"));
    assert_eq!(identity.groups, vec![ADMINS_GUID.to_owned()]);
    assert!(!identity.groups_overage);
    assert_eq!(identity.assertion_id.as_deref(), Some("_assertion-1"));
    assert_eq!(identity.assertion_expires_at, Some(expires));
    assert!(identity.raw_claims.get(GROUPS).is_some());
}

#[test]
fn with_no_name_id_the_email_is_the_subject() {
    let identity =
        claims(&[(EMAIL, &["a@dynapps.es"])]).to_external_identity(&config(), None, None);
    assert_eq!(identity.subject, "a@dynapps.es");
    let nobody = claims(&[]).to_external_identity(&config(), None, None);
    assert!(
        nobody.subject.is_empty(),
        "the driver refuses an empty subject"
    );
    assert!(!nobody.email_verified);
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
    assert!(
        overage_only
            .to_external_identity(&config(), None, None)
            .groups_overage
    );
    assert!(overage_only.values(GROUPS).is_empty());
    let both = claims(&[(GROUPS, &[ADMINS_GUID]), (OVERAGE, &[link])]);
    assert!(both.groups_overage());
    assert!(!claims(&[(GROUPS, &[ADMINS_GUID])]).groups_overage());
}

#[test]
fn the_config_defaults_and_usability_are_the_secure_ones() {
    assert!(config().is_usable());
    assert!(!EntraSamlConfig::disabled().is_usable());

    let parsed: EntraSamlConfig = serde_yaml::from_str(
        "enabled: true\nentity_id: https://x/saml/metadata\nacs_url: https://x/cb\nidp_metadata_path: m.xml\n",
    )
    .expect("minimal config parses");
    assert_eq!(parsed.groups_attribute, GROUPS);
    assert_eq!(parsed.email_attribute, EMAIL);
    assert_eq!(parsed.name_attribute, NAME);
    assert_eq!(parsed.idp_metadata_url, None);
    assert!(parsed.allow_idp_initiated);
    assert!(!parsed.policy.auto_provision);
    assert_eq!(parsed.policy.default_roles, vec!["user".to_owned()]);
    assert!(
        !parsed.is_usable(),
        "usable only once the loader has read the metadata file"
    );
}

#[test]
fn unknown_keys_are_refused_on_either_side_of_the_flattened_policy() {
    for yaml in [
        "enabled: true\nentity_id: a\nacs_url: r\nidp_metadata_path: m\nclient_id: x\n",
        "enabled: true\nallowed_email_domains: [a.test]\ngroup_rolez: {}\n",
    ] {
        assert!(
            serde_yaml::from_str::<EntraSamlConfig>(yaml).is_err(),
            "an unknown key must not be silently ignored: {yaml}"
        );
    }
    let policy_keys: EntraSamlConfig = serde_yaml::from_str(
        "enabled: false\nallowed_email_domains: [dynapps.es]\ndeny_without_group: true\n\
         group_roles: {G: [admin]}\ngroup_role_patterns: {'P-*': [user]}\nauto_provision: true\n",
    )
    .expect("the policy keys parse at the top level, as adfs.yaml writes them");
    assert_eq!(
        policy_keys.policy.allowed_email_domains,
        vec!["dynapps.es".to_owned()]
    );
    assert!(policy_keys.policy.deny_without_group);
    assert!(policy_keys.policy.auto_provision);
}

#[test]
fn a_metadata_url_parses_and_makes_the_config_usable_without_a_file() {
    let parsed: EntraSamlConfig = serde_yaml::from_str(
        "enabled: true\nentity_id: https://x/saml/metadata\nacs_url: https://x/admin/auth/adfs/acs\n\
         idp_metadata_url: https://login.microsoftonline.com/00000000-0000-4000-8000-000000000e17/federationmetadata/2007-06/federationmetadata.xml?appid=11111111-2222-4333-8444-555555555555\n",
    )
    .expect("a URL-only config parses");
    assert!(parsed.idp_metadata_path.is_empty());
    assert!(
        parsed.is_usable(),
        "the metadata is fetched at run time, so no file is needed"
    );

    let mut off = parsed;
    off.enabled = false;
    assert!(!off.is_usable());
}
