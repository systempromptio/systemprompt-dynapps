//! `clients:`, a farm's `client:` and per-agent `run_as:` in
//! agent-farms.yaml, the credential user's precedence, and the pure parts of
//! a run's task-completion report.

use systemprompt::identifiers::UserId;
use systemprompt_web_admin_farm::config::parse_farms_doc;
use systemprompt_web_admin_farm::services::runner::prepare::credential_user;
use systemprompt_web_admin_farm::services::runner::report::{collect_outputs, summary_of};

const DOC: &str = "clients:
  - slug: dynapps
    name: Dynapps
  - slug: acme-logistics
    name: Acme Logistics
    odoo_partner_id: 12
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
    assert_eq!(doc.farms[0].client.as_deref(), Some("acme-logistics"));
    assert_eq!(
        doc.farms[0].agents["lead_triage"].run_as.as_deref(),
        Some("svc-triage@dynapps.es")
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
fn an_unknown_agent_key_is_a_load_error() {
    let text = DOC.replace("run_as: svc-triage", "runas: svc-triage");
    assert!(parse_farms_doc(&text).is_err());
}

#[test]
fn credential_user_prefers_agent_then_farm_then_launcher() {
    let agent = UserId::new("agent-svc");
    let farm = UserId::new("farm-svc");
    let launcher = UserId::new("launcher");
    assert_eq!(credential_user(Some(&agent), Some(&farm), &launcher), agent);
    assert_eq!(credential_user(None, Some(&farm), &launcher), farm);
    assert_eq!(credential_user(None, None, &launcher), launcher);
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
