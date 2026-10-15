//! `clients:` (with the `service_account` client mode acts as), a farm's
//! `client:` and per-agent `run_as:` in agent-farms.yaml, the credential
//! user's resolution per mode, and the pure parts of a run's task-completion
//! report.

use systemprompt::identifiers::UserId;
use systemprompt_web_admin_farm::config::{RunAs, RunAsMode, parse_farms_doc};
use systemprompt_web_admin_farm::error::RunAsRefusal;
use systemprompt_web_admin_farm::services::runner::prepare::{
    RunAsClient, RunAsSetting, credential_user, effective_mode,
};
use systemprompt_web_admin_farm::services::runner::report::{collect_outputs, summary_of};
use systemprompt_web_admin_farm::types::ClientId;

const DOC: &str = "clients:
  - slug: dynapps
    name: Dynapps
  - slug: acme-logistics
    name: Acme Logistics
    odoo_partner_id: 12
    service_account: svc-acme@dynapps.es
farms:
  - id: triage
    name: Triage
    marketplace: dynapps
    client: acme-logistics
    agents:
      lead_triage:
        run_as: svc-triage@dynapps.es
    source:
      github: dynapps/agent-farm
";

#[test]
fn clients_and_agent_run_as_parse() {
    let doc = parse_farms_doc(DOC).expect("parses");
    doc.validate().expect("valid");
    assert_eq!(doc.clients.len(), 2);
    assert_eq!(doc.clients[1].odoo_partner_id, Some(12));
    assert_eq!(doc.clients[0].service_account, None);
    assert_eq!(
        doc.clients[1].service_account.as_deref(),
        Some("svc-acme@dynapps.es")
    );
    assert_eq!(doc.farms[0].client.as_deref(), Some("acme-logistics"));
    assert_eq!(
        doc.farms[0].agents["lead_triage"].run_as,
        Some(RunAs::service_account("svc-triage@dynapps.es"))
    );
}

#[test]
fn an_undeclared_client_is_refused() {
    let text = DOC.replace("client: acme-logistics", "client: globex");
    let doc = parse_farms_doc(&text).expect("parses");
    let err = doc.validate().expect_err("undeclared client");
    assert!(
        err.to_string().contains("not declared under clients"),
        "{err}"
    );
}

#[test]
fn a_duplicate_client_is_refused() {
    let text = DOC.replace("slug: acme-logistics", "slug: dynapps");
    let doc = parse_farms_doc(&text).expect("parses");
    assert!(doc.validate().is_err());
}

#[test]
fn an_empty_client_service_account_is_refused() {
    let text = DOC.replace("svc-acme@dynapps.es", "\"  \"");
    let doc = parse_farms_doc(&text).expect("parses");
    let err = doc.validate().expect_err("empty service_account");
    assert!(err.to_string().contains("empty service_account"), "{err}");
}

#[test]
fn an_agent_run_as_block_is_validated() {
    let text = DOC.replace(
        "run_as: svc-triage@dynapps.es",
        "run_as:\n          mode: service_account",
    );
    let doc = parse_farms_doc(&text).expect("parses");
    let err = doc.validate().expect_err("agent run_as without account");
    assert!(err.to_string().contains("agent \"lead_triage\""), "{err}");
}

#[test]
fn an_unknown_agent_key_is_a_load_error() {
    let text = DOC.replace("run_as: svc-triage", "runas: svc-triage");
    assert!(parse_farms_doc(&text).is_err());
}

const fn setting(mode: Option<RunAsMode>, account: Option<&UserId>) -> RunAsSetting<'_> {
    RunAsSetting { mode, account }
}

#[test]
fn credential_user_prefers_agent_then_farm_then_launcher() {
    let agent = UserId::new("agent-svc");
    let farm = UserId::new("farm-svc");
    let launcher = UserId::new("launcher");
    let none = RunAsSetting::default();
    let resolve = |a, f| credential_user(a, f, None, &launcher).expect("resolves");
    assert_eq!(
        resolve(setting(None, Some(&agent)), setting(None, Some(&farm))),
        agent
    );
    assert_eq!(resolve(none, setting(None, Some(&farm))), farm);
    assert_eq!(resolve(none, none), launcher);
}

#[test]
fn user_mode_is_the_launcher_even_with_a_farm_account() {
    let farm = UserId::new("farm-svc");
    let launcher = UserId::new("launcher");
    let agent = setting(Some(RunAsMode::User), None);
    let farm_as = setting(Some(RunAsMode::ServiceAccount), Some(&farm));
    assert_eq!(effective_mode(agent, farm_as), RunAsMode::User);
    assert_eq!(
        credential_user(agent, farm_as, None, &launcher).expect("resolves"),
        launcher
    );
}

#[test]
fn client_mode_is_the_clients_service_account() {
    let launcher = UserId::new("launcher");
    let acme = ClientId::new("acme-logistics");
    let svc = UserId::new("svc-acme");
    let farm_as = setting(Some(RunAsMode::Client), None);
    let client = RunAsClient {
        id: &acme,
        account: Some(&svc),
    };
    assert_eq!(
        credential_user(RunAsSetting::default(), farm_as, Some(client), &launcher)
            .expect("resolves"),
        svc
    );
}

#[test]
fn client_mode_refusals_are_typed() {
    let launcher = UserId::new("launcher");
    let acme = ClientId::new("acme-logistics");
    let farm_as = setting(Some(RunAsMode::Client), None);
    let none = RunAsSetting::default();
    assert_eq!(
        credential_user(none, farm_as, None, &launcher),
        Err(RunAsRefusal::NoClient)
    );
    let bare = RunAsClient {
        id: &acme,
        account: None,
    };
    assert_eq!(
        credential_user(none, farm_as, Some(bare), &launcher),
        Err(RunAsRefusal::ClientHasNoAccount {
            client: "acme-logistics".to_owned()
        })
    );
}

#[test]
fn service_account_mode_without_a_resolved_account_is_refused() {
    let launcher = UserId::new("launcher");
    let farm_as = setting(Some(RunAsMode::ServiceAccount), None);
    assert_eq!(
        credential_user(RunAsSetting::default(), farm_as, None, &launcher),
        Err(RunAsRefusal::NoServiceAccount)
    );
}

#[test]
fn a_stored_mode_string_is_read_back() {
    let svc = UserId::new("svc");
    let row = RunAsSetting::from_row(Some("client"), Some(&svc));
    assert_eq!(row.mode, Some(RunAsMode::Client));
    assert_eq!(RunAsSetting::from_row(Some("bogus"), None).mode, None);
}

#[test]
fn summary_is_the_first_paragraph() {
    assert_eq!(
        summary_of("\n\nTriaged 3 leads.\nAll good.\n\nDetails follow.").as_deref(),
        Some("Triaged 3 leads.\nAll good.")
    );
    assert_eq!(summary_of("   "), None);
    assert_eq!(summary_of(&"x".repeat(900)).map(|s| s.len()), Some(500));
}

#[test]
fn outputs_are_hashed_files_under_the_output_path() {
    let dir = tempfile::tempdir().expect("tempdir");
    let workspace = dir.path();
    std::fs::write(workspace.join("TRIAGE.md"), b"abc").expect("write");
    std::fs::create_dir_all(workspace.join("out/sub")).expect("mkdir");
    std::fs::write(workspace.join("out/b.txt"), b"").expect("write");
    std::fs::write(workspace.join("out/sub/a.txt"), b"x").expect("write");

    let single = collect_outputs(workspace, "TRIAGE.md").expect("collect");
    assert_eq!(single.len(), 1);
    assert_eq!(single[0].path, "TRIAGE.md");
    assert_eq!(single[0].bytes, 3);
    assert_eq!(
        single[0].sha256,
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );

    let tree = collect_outputs(workspace, "out").expect("collect");
    let paths: Vec<&str> = tree.iter().map(|f| f.path.as_str()).collect();
    assert_eq!(paths, ["out/b.txt", "out/sub/a.txt"]);

    assert!(
        collect_outputs(workspace, "missing.md")
            .expect("collect")
            .is_empty()
    );
    assert!(
        collect_outputs(workspace, "../etc")
            .expect("collect")
            .is_empty()
    );
}
