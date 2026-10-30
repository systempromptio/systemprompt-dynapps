//! `OpenCode`'s provider `baseURL` is always the loopback proxy, never the
//! gateway: `OpenCode` 1.19 sends its own `x-session-id`, which the gateway
//! refuses, and only the proxy replaces it with the bridge session.

use systemprompt_bridge::ids::LoopbackSecret;
use systemprompt_bridge::install::profile_inputs::{ModelProtocolOverrides, ProfileFacts};
use systemprompt_bridge::integration::{HostKind, find_host_by_id};
use systemprompt_bridge::mcp_registry::McpRegistry;
use systemprompt_bridge::proxy::LoopbackEndpoint;

const GATEWAY: &str = "https://gw.example.invalid";

fn with_gateway_config(f: impl FnOnce()) {
    crate::support::install_tls();
    let dir = tempfile::tempdir().expect("tempdir");
    let config = dir.path().join("config");
    let home = dir.path().join("home");
    std::fs::create_dir_all(config.join("systemprompt")).expect("config dir");
    std::fs::create_dir_all(&home).expect("home");
    std::fs::write(
        config.join("systemprompt").join("systemprompt-bridge.toml"),
        format!("gateway_url = \"{GATEWAY}\"\n"),
    )
    .expect("config file");
    temp_env::with_vars(
        [
            ("HOME", Some(home.display().to_string())),
            ("XDG_CONFIG_HOME", Some(config.display().to_string())),
            (
                "XDG_STATE_HOME",
                Some(dir.path().join("state").display().to_string()),
            ),
            (
                "XDG_CACHE_HOME",
                Some(dir.path().join("cache").display().to_string()),
            ),
            ("SP_BRIDGE_CONFIG", None),
        ],
        f,
    );
}

fn base_url_for(loopback: &LoopbackEndpoint) -> String {
    let facts = ProfileFacts::load(
        loopback,
        &McpRegistry::new(),
        ModelProtocolOverrides::new(),
        None,
    )
    .expect("profile facts");
    let inputs = facts.inputs_for(HostKind::OpenCode);
    let host = find_host_by_id(HostKind::OpenCode).expect("opencode host");
    let generated = host
        .generate_profile(&inputs)
        .expect("render opencode.json");
    let text = std::fs::read_to_string(&generated.path).expect("read rendered profile");
    let json: serde_json::Value = serde_json::from_str(&text).expect("rendered json");
    json["config"]["providers"]["systemprompt"]["settings"]["baseURL"]
        .as_str()
        .unwrap_or_else(|| panic!("no config.providers.systemprompt.settings.baseURL in {text}"))
        .to_owned()
}

#[test]
fn a_running_proxy_is_the_opencode_base_url() {
    with_gateway_config(|| {
        let loopback = LoopbackEndpoint::new(48217, Some(LoopbackSecret::new("s")));
        assert_eq!(base_url_for(&loopback), "http://127.0.0.1:48217/v1");
    });
}

#[test]
fn a_proxy_that_never_started_renders_nothing_rather_than_the_gateway() {
    with_gateway_config(|| {
        let loopback = LoopbackEndpoint::new(48300, None);
        let facts = ProfileFacts::load(
            &loopback,
            &McpRegistry::new(),
            ModelProtocolOverrides::new(),
            None,
        );
        assert!(
            facts.is_err(),
            "with no loopback secret there is no profile to write, and no fallback to {GATEWAY}"
        );
    });
}
