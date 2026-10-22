//! The Fly Machine runner: `runner.mode` parsing and its env override, the
//! Machines API payloads, the worker wire contract (bundles, path rebasing,
//! MCP endpoint rewriting) and the run-scoped worker token registry.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use systemprompt::agent_runs::service::DbEventSink;
use systemprompt::agent_runs::{
    AgentRunRepository, HostKind, McpServerSpec, PermissionMode, RunRequest,
};
use systemprompt::identifiers::AgentRunId;
use systemprompt_web_admin_farm::config::{
    FlyRunnerConfig, RunnerMode, parse_farms_doc, with_mode_override,
};
use systemprompt_web_admin_farm::services::runner::fly::api::{
    MachineLaunch, auth_header, machine_config,
};
use systemprompt_web_admin_farm::services::runner::fly::egress::EgressPolicy;
use systemprompt_web_admin_farm::services::runner::fly::pool::{private_api_base, run_env};
use systemprompt_web_admin_farm::services::runner::fly::protocol::{
    Bundle, BundleFile, WorkerCredentials, WorkerResult, WorkerSpec, collect_bundle,
    rebase_request, worker_url, write_bundle,
};
use systemprompt_web_admin_farm::services::runner::fly::registry::RemoteRuns;
use systemprompt_web_admin_farm::services::runner::fly::remote::request_for_worker;
use systemprompt_web_admin_farm::services::runner::observer::unexecuted_outcome;
use systemprompt_web_admin_farm::types::{AgentRunStatus, ErrorClass};

const SERVER_ROOT: &str = "/app/storage/data/agent-farm/runs/run-1";

fn request() -> RunRequest {
    let root = Path::new(SERVER_ROOT);
    RunRequest {
        host: HostKind::ClaudeCode,
        prompt: format!("Work in {}/workspace please", SERVER_ROOT),
        system_prompt_append: None,
        persona: None,
        model: None,
        allowed_tools: Vec::new(),
        disallowed_tools: Vec::new(),
        permission_mode: PermissionMode::AcceptEdits,
        permission_prompts_none: false,
        workspace: root.join("workspace"),
        add_dirs: Vec::new(),
        mcp_servers: BTreeMap::from([(
            "dynapps-odoo".to_owned(),
            McpServerSpec::Http {
                url: "http://localhost:8080/api/v1/mcp/dynapps-odoo/mcp".to_owned(),
                headers: BTreeMap::new(),
            },
        )]),
        output_schema: None,
        max_turns: Some(5),
        max_budget_usd: Some(1.0),
        wall_clock: Duration::from_secs(60),
        resume_session_id: None,
        attempt: 1,
        attribution_scopes: Vec::new(),
        plugin_id: None,
        plugin_dirs: vec![root.join("plugin/kit")],
    }
}

const FARMS: &str = "farms: []\nrunner:\n  max_concurrent: 1\n";

#[test]
fn runner_mode_defaults_to_local() {
    let doc = parse_farms_doc(FARMS).expect("parses");
    let runner = doc.runner.expect("runner block");
    assert_eq!(runner.mode, RunnerMode::Local);
    assert_eq!(runner.fly, None);
}

#[test]
fn fly_machine_mode_parses_with_defaults() {
    let text = format!("{FARMS}  mode: fly_machine\n  fly:\n    pool_size: 3\n");
    let runner = parse_farms_doc(&text)
        .expect("parses")
        .runner
        .expect("runner");
    assert_eq!(runner.mode, RunnerMode::FlyMachine);
    let fly = runner.fly.expect("fly block");
    assert_eq!(fly.pool_size, 3);
    assert_eq!(fly.machines_api, "https://api.machines.dev");
    assert_eq!(fly.worker_bin, "/app/bin/systemprompt-farm-worker");
}

#[test]
fn unknown_fly_key_is_refused() {
    let text = format!("{FARMS}  fly:\n    pool: 3\n");
    assert!(parse_farms_doc(&text).is_err());
}

#[test]
fn env_override_switches_mode_and_ignores_garbage() {
    let runner = parse_farms_doc(FARMS)
        .expect("parses")
        .runner
        .expect("runner");
    let switched = with_mode_override(runner.clone(), Some("fly_machine"));
    assert_eq!(switched.mode, RunnerMode::FlyMachine);
    assert_eq!(
        with_mode_override(runner.clone(), Some("cloud")).mode,
        RunnerMode::Local
    );
    assert_eq!(with_mode_override(runner, None).mode, RunnerMode::Local);
}

#[test]
fn machine_config_launches_the_worker_without_services() {
    let launch = MachineLaunch {
        image: "registry.fly.io/app:deployment-1".to_owned(),
        env: run_env("run-1", "http://m1.vm.app.internal:8080", "tok"),
    };
    let config = machine_config(&FlyRunnerConfig::default(), &launch);
    assert_eq!(config["image"], "registry.fly.io/app:deployment-1");
    assert_eq!(
        config["init"]["exec"][0],
        "/app/bin/systemprompt-farm-worker"
    );
    assert_eq!(config["metadata"]["farm_runner"], "pool");
    assert_eq!(config["restart"]["policy"], "no");
    assert_eq!(config["env"]["RUN_ID"], "run-1");
    assert_eq!(config["env"]["FARM_WORKER_TOKEN"], "tok");
    assert!(config.get("services").is_none());
}

#[test]
fn auth_header_keeps_a_flyv1_scheme() {
    assert_eq!(auth_header("FlyV1 fm2_abc"), "FlyV1 fm2_abc");
    assert_eq!(auth_header("abc"), "Bearer abc");
}

#[test]
fn worker_reports_to_this_api_machine() {
    assert_eq!(
        private_api_base("sp-app", "e2865013", 8080),
        "http://e2865013.vm.sp-app.internal:8080"
    );
    assert_eq!(
        worker_url("http://h:8080/", "run-1", "spec"),
        "http://h:8080/api/public/agent-farm-worker/runs/run-1/spec"
    );
}

#[test]
fn mcp_endpoints_move_to_the_worker_base() {
    let moved = request_for_worker(
        &request(),
        "http://localhost:8080",
        "http://m1.vm.a.internal:8080",
    );
    match &moved.mcp_servers["dynapps-odoo"] {
        McpServerSpec::Http { url, .. } => {
            assert_eq!(
                url,
                "http://m1.vm.a.internal:8080/api/v1/mcp/dynapps-odoo/mcp"
            );
        },
        other @ McpServerSpec::Stdio { .. } => panic!("unexpected spec {other:?}"),
    }
}

#[test]
fn rebase_moves_every_path_and_the_prompt() {
    let rebased = rebase_request(&request(), Path::new(SERVER_ROOT), Path::new("/tmp/w"));
    assert_eq!(rebased.workspace, PathBuf::from("/tmp/w/workspace"));
    assert_eq!(
        rebased.plugin_dirs,
        vec![PathBuf::from("/tmp/w/plugin/kit")]
    );
    assert_eq!(rebased.prompt, "Work in /tmp/w/workspace please");
    let same = rebase_request(&request(), Path::new(SERVER_ROOT), Path::new(SERVER_ROOT));
    assert_eq!(same, request());
}

#[test]
fn bundle_round_trips_and_refuses_escapes() {
    let from = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(from.path().join("workspace/src")).expect("mkdir");
    std::fs::write(from.path().join("workspace/src/a.py"), b"print(1)\n").expect("write");
    std::fs::write(from.path().join("ignored.txt"), b"x").expect("write");
    let bundle = collect_bundle(from.path(), &["workspace", "missing"]).expect("collect");
    assert_eq!(bundle.files.len(), 1);
    assert_eq!(bundle.files[0].path, "workspace/src/a.py");

    let to = tempfile::tempdir().expect("tempdir");
    assert_eq!(write_bundle(to.path(), &bundle).expect("write"), 1);
    let copied = std::fs::read(to.path().join("workspace/src/a.py")).expect("read");
    assert_eq!(copied, b"print(1)\n");

    for path in ["../escape", "/etc/passwd", "workspace/../../x"] {
        let evil = Bundle {
            files: vec![BundleFile {
                path: path.to_owned(),
                content_b64: String::new(),
            }],
        };
        assert!(write_bundle(to.path(), &evil).is_err(), "{path} accepted");
    }
}

fn spec(run_id: &str) -> WorkerSpec {
    let credentials: WorkerCredentials = serde_json::from_value(serde_json::json!({
        "user_id": "user-1",
        "api_key_id": "key-1",
        "pat": "sp-secret",
        "gateway_session_id": "sess-1",
        "hook_token": null,
        "mcp_token": null,
    }))
    .expect("credentials");
    WorkerSpec {
        agent_run_id: run_id.to_owned(),
        farm_run_id: "farm-run-1".to_owned(),
        server_root: PathBuf::from(SERVER_ROOT),
        request: request(),
        credentials,
        gateway_base_url: "http://h:8080".to_owned(),
        govern_url: "http://h:8080/api/public/hooks/govern".to_owned(),
        track_url: "http://h:8080/api/public/hooks/track".to_owned(),
        seq_after: Some(2),
        heartbeat_seconds: 10,
        egress: EgressPolicy::default(),
    }
}

#[tokio::test]
async fn worker_token_opens_only_its_run_while_registered() {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://unused@localhost/unused")
        .expect("lazy pool");
    let repo = AgentRunRepository::from_pool(std::sync::Arc::new(pool));
    let sink = DbEventSink::new(repo, AgentRunId::new("fly-reg-run"));
    let runs = std::sync::Arc::new(RemoteRuns::default());
    let mut registration = runs.register(
        spec("fly-reg-run"),
        PathBuf::from(SERVER_ROOT),
        sink,
        "tok-a",
    );
    assert!(runs.authorise("fly-reg-run", "tok-a").is_some());
    assert!(runs.authorise("fly-reg-run", "tok-b").is_none());
    assert!(runs.authorise("other-run", "tok-a").is_none());
    let other = RemoteRuns::default();
    assert!(
        other.authorise("fly-reg-run", "tok-a").is_none(),
        "a registry is owned, not process-wide"
    );

    let run = runs.authorise("fly-reg-run", "tok-a").expect("admitted");
    assert!(!run.cancel_requested());
    run.request_cancel();
    assert!(run.cancel_requested());
    let result = WorkerResult {
        outcome: unexecuted_outcome(AgentRunStatus::Failed, ErrorClass::HostCrashed, "x"),
        diff: None,
    };
    assert!(run.deliver(result.clone()));
    assert!(!run.deliver(result.clone()));
    assert_eq!((&mut registration.result).await.expect("delivered"), result);

    drop(registration);
    assert!(runs.authorise("fly-reg-run", "tok-a").is_none());
}
