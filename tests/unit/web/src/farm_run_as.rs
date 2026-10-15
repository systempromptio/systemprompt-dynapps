//! `run_as` in agent-farms.yaml: optional, typed as a mode, the legacy plain
//! email still read as `service_account`, and an empty or misplaced account
//! refused rather than read as "run as nobody".

use systemprompt_web_admin_farm::config::{RunAs, RunAsMode, parse_farms_doc};

const FARM: &str = "farms:
  - id: triage
    name: Triage
    marketplace: dynapps
    source:
      github: dynapps/agent-farm
";

fn farm_with(run_as: &str) -> String {
    format!("{FARM}    run_as:{run_as}\n")
}

#[test]
fn run_as_defaults_to_the_launcher() {
    let doc = parse_farms_doc(FARM).expect("parses");
    doc.validate().expect("valid");
    assert_eq!(doc.farms[0].run_as, None);
}

#[test]
fn a_plain_email_is_the_legacy_service_account_form() {
    let doc = parse_farms_doc(&farm_with(" svc-triage")).expect("parses");
    doc.validate().expect("valid");
    let run_as = doc.farms[0].run_as.as_ref().expect("run_as");
    assert_eq!(run_as, &RunAs::service_account("svc-triage"));
    assert_eq!(run_as.account(), Some("svc-triage"));
}

#[test]
fn each_mode_parses_from_a_block() {
    for (mode, expected) in [("user", RunAsMode::User), ("client", RunAsMode::Client)] {
        let doc = parse_farms_doc(&farm_with(&format!("\n      mode: {mode}"))).expect("parses");
        doc.validate().expect("valid");
        let run_as = doc.farms[0].run_as.as_ref().expect("run_as");
        assert_eq!(run_as, &RunAs::of(expected));
        assert_eq!(run_as.account(), None, "{mode} names no account to resolve");
    }
    let text = farm_with("\n      mode: service_account\n      service_account: svc@dynapps.es");
    let doc = parse_farms_doc(&text).expect("parses");
    doc.validate().expect("valid");
    assert_eq!(
        doc.farms[0].run_as.as_ref().and_then(RunAs::account),
        Some("svc@dynapps.es")
    );
}

#[test]
fn mode_strings_round_trip() {
    for mode in RunAsMode::ALL {
        assert_eq!(RunAsMode::parse(mode.as_str()), Some(mode));
    }
    assert_eq!(RunAsMode::parse("robot"), None);
}

#[test]
fn empty_run_as_is_refused() {
    let doc = parse_farms_doc(&farm_with(" \"  \"")).expect("parses");
    let err = doc.validate().expect_err("empty run_as must fail");
    assert!(err.to_string().contains("run_as is empty"), "{err}");
}

#[test]
fn service_account_mode_without_an_account_is_refused() {
    let doc = parse_farms_doc(&farm_with("\n      mode: service_account")).expect("parses");
    let err = doc.validate().expect_err("no account");
    assert!(err.to_string().contains("run_as is empty"), "{err}");
}

#[test]
fn an_account_outside_service_account_mode_is_refused() {
    let text = farm_with("\n      mode: user\n      service_account: svc@dynapps.es");
    let doc = parse_farms_doc(&text).expect("parses");
    let err = doc.validate().expect_err("misplaced account");
    assert!(
        err.to_string()
            .contains("only read in mode service_account"),
        "{err}"
    );
}

#[test]
fn an_unknown_mode_or_key_is_a_load_error() {
    assert!(parse_farms_doc(&farm_with("\n      mode: robot")).is_err());
    assert!(parse_farms_doc(&farm_with("\n      mode: user\n      as: me")).is_err());
}
