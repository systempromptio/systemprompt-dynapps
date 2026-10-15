//! The committed SSO config is load-bearing and unvalidated until boot:
//! nothing else in the suite reads `services/web/config/entra-oidc.yaml`.
//! These checks parse it the way the loader does, so a typo'd key or a role
//! map that signs an administrator in without a console fails here rather
//! than at a user's sign-in.

use std::path::PathBuf;

use systemprompt_web_admin_identity::sso::entra_oidc::EntraOidcConfig;

use crate::support::repo_root;

fn config_path() -> PathBuf {
    repo_root().join("services/web/config/entra-oidc.yaml")
}

fn parse() -> EntraOidcConfig {
    let path = config_path();
    let yaml = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{} is readable: {e}", path.display()));
    serde_yaml::from_str(&yaml)
        .unwrap_or_else(|e| panic!("{} does not deserialise: {e}", path.display()))
}

#[test]
fn the_committed_entra_oidc_config_parses() {
    let cfg = parse();
    if cfg.enabled {
        assert!(
            !cfg.entra_tenant_id.trim().is_empty() && !cfg.entra_client_id.trim().is_empty(),
            "{} is enabled without a tenant and client id",
            config_path().display()
        );
    }
}

// Why: core mints `Permission::Admin` from the `admin` role alone, so a group
// granting `platform_admin` without it signs a console user in with no
// console.
#[test]
fn every_platform_admin_grant_also_grants_admin() {
    let cfg = parse();
    for (group, roles) in cfg
        .policy
        .group_roles
        .iter()
        .chain(cfg.policy.group_role_patterns.iter())
    {
        if roles.iter().any(|r| r == "platform_admin") {
            assert!(
                roles.iter().any(|r| r == "admin"),
                "entra-oidc.yaml maps {group} to platform_admin without admin"
            );
        }
    }
}
