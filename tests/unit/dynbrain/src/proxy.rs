//! Proxy mode without a source: the config file and what it refuses, the
//! translation of arguments, the operation names, the source's answers and
//! refusals as faults, the file references of a source's Markdown, and the
//! links key both processes share.

use chrono::{Duration, Utc};
use dynbrain::assets::link::{LinkPurpose, LinkSigner};
use dynbrain::brain::fault::{BrainFault, SourceFault};
use dynbrain::brain::operation::Operation;
use dynbrain::brain::proxy::files::{link_targets, rewrite, source_files};
use dynbrain::brain::proxy::source::decode;
use dynbrain::config::{BrainConfig, ConfigError, Mode, ToolMapping, check_server};
use dynbrain::router::error::HttpFault;
use serde_json::json;
use systemprompt::manifest::ServicesConfig;
use systemprompt::models::CallToolResult;

const PROXY: &str = r"
dynbrain:
  mode: proxy
  proxy:
    server: brain-source
    file_urls: ['/files/image/{id}', '/files/content/{id}']
    operations:
      search:
        tool: docs_search
        args: { query: query, where: filter }
      answer:
        tool: docs_answer
        args: { question: question }
        fixed: { synthesize: false }
";

#[test]
fn without_a_file_or_a_block_the_brain_is_native() {
    assert_eq!(BrainConfig::parse("").expect("empty").mode, Mode::Native);
    let mounted = BrainConfig::from_json(&json!({"paths": ["/api/dynbrain/v1"]})).expect("mount");
    assert_eq!(mounted.mode, Mode::Native);
    let shipped = std::fs::read_to_string(repo_file("services/config/dynbrain.yaml"))
        .expect("the shipped file");
    let shipped = BrainConfig::parse(&shipped).expect("the shipped file parses");
    assert_eq!(shipped.mode, Mode::Native);
    assert!(shipped.proxy.is_none(), "the product ships no mapping");
}

#[test]
fn the_mapping_reads_its_tool_arguments_and_constants() {
    let config = BrainConfig::parse(PROXY).expect("a proxy config");
    assert_eq!(config.mode, Mode::Proxy);
    let proxy = config.proxy.expect("the proxy block");
    assert_eq!(proxy.server, "brain-source");
    assert_eq!(proxy.file_urls.len(), 2);
    let answer = &proxy.operations[&Operation::Answer];
    assert_eq!(answer.tool, "docs_answer");
    assert_eq!(answer.fixed["synthesize"], json!(false));
    assert!(!proxy.operations.contains_key(&Operation::Get));
}

#[test]
fn an_unknown_mode_or_key_is_refused_at_startup() {
    let mode = BrainConfig::parse("dynbrain:\n  mode: both\n");
    assert!(matches!(mode, Err(ConfigError::Yaml(_))), "{mode:?}");
    let key = BrainConfig::from_json(&json!({"dynbrain": {"mode": "native", "sources": []}}));
    assert!(matches!(key, Err(ConfigError::Json(_))), "{key:?}");
    let operation = BrainConfig::parse(
        "dynbrain:\n  mode: proxy\n  proxy:\n    server: s\n    operations:\n      delete_all: { tool: x }\n",
    );
    assert!(operation.is_err(), "{operation:?}");
    let top = BrainConfig::from_json(&json!({"brain": {"mode": "proxy"}}));
    assert!(top.is_err(), "{top:?}");
}

#[test]
fn proxy_mode_needs_a_source_server_and_tools() {
    let none = BrainConfig::parse("dynbrain:\n  mode: proxy\n");
    assert!(matches!(none, Err(ConfigError::NoSource)), "{none:?}");
    let blank =
        BrainConfig::from_json(&json!({"dynbrain": {"mode": "proxy", "proxy": {"server": " "}}}));
    assert!(matches!(blank, Err(ConfigError::NoSource)), "{blank:?}");
    let tool = BrainConfig::from_json(&json!({"dynbrain": {"mode": "proxy", "proxy": {
        "server": "s", "operations": {"get": {"tool": ""}}
    }}}));
    assert!(
        matches!(tool, Err(ConfigError::EmptyTool("get"))),
        "{tool:?}"
    );
}

#[test]
fn a_source_server_must_be_declared_and_enabled() {
    let mut services = ServicesConfig::default();
    let missing = check_server(&services, "brain-source");
    assert!(
        matches!(missing, Err(ConfigError::UndeclaredServer(_))),
        "{missing:?}"
    );
    let declared = |enabled: bool| {
        serde_json::from_value(json!({
            "server_type": "external",
            "endpoint": "https://source.test/mcp/docs",
            "enabled": enabled,
            "display_in_web": false,
            "oauth": {"required": false, "scopes": [], "audience": "mcp", "client_id": null}
        }))
        .expect("a deployment")
    };
    services
        .mcp_servers
        .insert("brain-source".to_owned(), declared(false));
    assert!(
        check_server(&services, "brain-source").is_err(),
        "a disabled server"
    );
    services
        .mcp_servers
        .insert("brain-source".to_owned(), declared(true));
    check_server(&services, "brain-source").expect("a declared server");
    assert!(check_server(&services, "other").is_err());
}

#[test]
fn arguments_are_renamed_nulls_dropped_and_constants_added() {
    let mapping: ToolMapping = serde_json::from_value(json!({
        "tool": "docs_search",
        "args": {"where": "filter"},
        "fixed": {"synthesize": false, "limit": 3}
    }))
    .expect("a mapping");
    let sent = mapping.translate(json!({
        "query": "backups", "where": {"status": "decided"}, "lang": null, "limit": 50
    }));
    assert_eq!(
        sent,
        json!({"query": "backups", "filter": {"status": "decided"}, "synthesize": false, "limit": 3})
    );
    assert_eq!(
        mapping.translate(json!(null)),
        json!({"synthesize": false, "limit": 3})
    );
}

#[test]
fn the_operations_are_the_eighteen_tools() {
    let names: Vec<&str> = Operation::ALL.iter().map(|op| op.name()).collect();
    assert_eq!(names.len(), 18);
    for name in &names {
        assert_eq!(Operation::from_tool(name).map(Operation::name), Some(*name));
        let parsed: Operation = serde_json::from_value(json!(name)).expect("a config key");
        assert_eq!(parsed.name(), *name);
    }
    assert!(Operation::from_tool("list_grants").is_none());
    assert!(Operation::from_tool("set_grants").is_none());
}

#[test]
fn the_answer_of_the_source_is_its_structured_content_or_its_json_text() {
    let structured = decode(CallToolResult::structured(json!({"found": true})));
    assert_eq!(structured.expect("structured"), json!({"found": true}));
    let text: CallToolResult = serde_json::from_value(json!({
        "content": [{"type": "text", "text": "{\"results\": [], \"total\": 0}"}]
    }))
    .expect("a text result");
    assert_eq!(
        decode(text).expect("json text"),
        json!({"results": [], "total": 0})
    );
    let plain: CallToolResult =
        serde_json::from_value(json!({"content": [{"type": "text", "text": "hello"}]}))
            .expect("a plain result");
    assert_eq!(decode(plain).expect("plain"), json!("hello"));
    let refused = decode(CallToolResult::structured_error(json!({"error": "no"})));
    assert!(
        matches!(refused, Err(SourceFault::Invalid(_))),
        "{refused:?}"
    );
}

#[test]
fn a_refusal_of_the_source_answers_like_the_native_one() {
    let cases = [
        (
            SourceFault::Refused("token revoked".to_owned()),
            403,
            "forbidden",
        ),
        (
            SourceFault::NotFound("no such document".to_owned()),
            404,
            "not_found",
        ),
        (
            SourceFault::Invalid("bad argument".to_owned()),
            400,
            "invalid",
        ),
    ];
    for (fault, status, key) in cases {
        let http = HttpFault::from(BrainFault::from(fault));
        assert_eq!(http.status().as_u16(), status, "{http}");
        assert_eq!(http.key(), key, "{http}");
    }
    let refused = BrainFault::from(SourceFault::Refused("token revoked".to_owned()));
    assert!(
        refused.to_string().contains("token revoked"),
        "the source's message is kept"
    );
    let managed = HttpFault::from(BrainFault::ManagedAtSource);
    assert_eq!(managed.status().as_u16(), 409);
    assert_eq!(managed.key(), "managed_at_source");
    let unmapped = HttpFault::from(BrainFault::Unavailable("no tool"));
    assert_eq!(unmapped.key(), "unavailable");
}

#[test]
fn the_link_targets_of_a_body_are_found_once_in_order() {
    let body = "![a](https://src.test/files/image/41/a.png \"A\")\n[b](img/b.svg) ![a](https://src.test/files/image/41/a.png)\n[top](#top)";
    assert_eq!(
        link_targets(body),
        vec!["https://src.test/files/image/41/a.png", "img/b.svg", "#top"]
    );
}

#[test]
fn source_urls_and_listed_paths_become_asset_references() {
    let patterns = vec![
        "/files/image/{id}".to_owned(),
        "/files/content/{id}".to_owned(),
    ];
    let paths = vec![
        ("docs/backups/img/red.svg".to_owned(), 43),
        ("notes.pdf".to_owned(), 44),
    ];
    let targets: Vec<String> = [
        "https://src.test/files/image/41/a.png",
        "/files/content/42?download=true",
        "/files/image/415x",
        "./img/red.svg",
        "../notes.pdf",
        "missing.png",
        "asset://9",
        "https://elsewhere.test/page",
    ]
    .iter()
    .map(|target| (*target).to_owned())
    .collect();
    let found = source_files(&targets, &patterns, &paths);
    let ids: Vec<i64> = found.iter().map(|(_, id)| *id).collect();
    assert_eq!(ids, vec![41, 42, 43, 44]);
    let body = "![a](https://src.test/files/image/41/a.png) [m](/files/content/42?download=true) ![r](./img/red.svg) [x](missing.png) ![k](asset://9)";
    assert_eq!(
        rewrite(body, &found),
        "![a](asset://41) [m](asset://42) ![r](asset://43) [x](missing.png) ![k](asset://9)"
    );
}

#[test]
fn both_processes_sign_with_the_same_instance_secret() {
    let api =
        LinkSigner::from_secret("0f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f0")
            .expect("a signer");
    let mcp =
        LinkSigner::from_secret("0f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f0")
            .expect("a signer");
    let other = LinkSigner::from_secret("another secret").expect("a signer");
    let now = Utc::now();
    let link = mcp
        .sign(LinkPurpose::Download, "12", now + Duration::minutes(10))
        .expect("a link");
    assert!(api.verify(LinkPurpose::Download, "12", &link, now));
    assert!(!other.verify(LinkPurpose::Download, "12", &link, now));
    assert!(LinkSigner::from_secret("").is_none());
    assert!(LinkSigner::from_secret("   ").is_none());
}

fn repo_file(path: &str) -> std::path::PathBuf {
    let mut dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        if dir.join(path).is_file() {
            return dir.join(path);
        }
        assert!(dir.pop(), "{path} not found above CARGO_MANIFEST_DIR");
    }
}

#[test]
fn an_admin_read_carries_the_admins_session_token() {
    let admin = systemprompt::identifiers::UserId::new("admin-1");
    let token = systemprompt::identifiers::JwtToken::new("session-of-admin-1");
    let context = dynbrain::console::admin_context(&admin, Some(&token));
    assert_eq!(
        context
            .auth_token()
            .map(systemprompt::identifiers::JwtToken::as_str),
        Some("session-of-admin-1"),
        "a proxy source answers as the admin only with their token"
    );
    let bare = dynbrain::console::admin_context(&admin, None);
    assert!(bare.auth_token().is_none());
}
