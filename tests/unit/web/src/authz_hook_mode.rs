//! Core refuses to boot when a registered in-process authz hook and the
//! profile's `governance.authz.hook.mode` disagree, so the hook, every binary
//! that builds an `AppContext`, and every profile this repo renders move
//! together: the hook is registered, both binaries link it, and each profile
//! source says `mode: extension` with no webhook URL left behind.

use crate::support::repo_root;

fn read(relative: &str) -> String {
    let path = repo_root().join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

#[test]
fn the_installation_registers_one_in_process_authz_hook() {
    let hook = read("extensions/web/admin/governance/src/authz_hook.rs");
    assert!(hook.contains("register_authz_hook!"));
    assert!(hook.contains("crate::authz_eval::evaluate"));
    let lib = read("extensions/web/admin/governance/src/lib.rs");
    assert!(lib.contains("pub mod authz_hook;"));
    let mcp = read("extensions/mcp/systemprompt/src/main.rs");
    assert!(
        mcp.contains("use systemprompt_web_admin_governance as _;"),
        "the MCP binary builds an AppContext and must link the hook"
    );
}

#[test]
fn every_rendered_profile_runs_the_hook_in_process() {
    for (source, needle) in [
        (
            "deploy/self-host/profile.yaml.template",
            "      mode: extension\n",
        ),
        (
            "tests/contract/admin/fixtures/profile.yaml",
            "      mode: extension\n",
        ),
        (
            "deploy/helm/systemprompt-dynapps/templates/_helpers.tpl",
            "\"mode\" \"extension\"",
        ),
        ("justfile", "admin config governance set --mode extension"),
    ] {
        let text = read(source);
        assert!(
            text.contains(needle),
            "{source} does not select mode: extension"
        );
        assert!(
            !text.contains("mode: webhook") && !text.contains("\"mode\" \"webhook\""),
            "{source} still selects the authz webhook"
        );
    }
}
