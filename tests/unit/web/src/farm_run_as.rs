//! `run_as` in agent-farms.yaml: optional, carried through parsing, and an
//! empty value is refused rather than read as "run as nobody".

use systemprompt_web_admin_farm::config::parse_farms_doc;

const FARM: &str = "farms:
  - id: triage
    name: Triage
    marketplace: dynapps
    source:
      github: dynapps/agent-farm
";

#[test]
fn run_as_defaults_to_the_launcher() {
    let doc = parse_farms_doc(FARM).expect("parses");
    doc.validate().expect("valid");
    assert_eq!(doc.farms[0].run_as, None);
}

#[test]
fn run_as_names_the_service_account() {
    let text = format!("{FARM}    run_as: svc-triage\n");
    let doc = parse_farms_doc(&text).expect("parses");
    doc.validate().expect("valid");
    assert_eq!(doc.farms[0].run_as.as_deref(), Some("svc-triage"));
}

#[test]
fn empty_run_as_is_refused() {
    let text = format!("{FARM}    run_as: \"  \"\n");
    let doc = parse_farms_doc(&text).expect("parses");
    let err = doc.validate().expect_err("empty run_as must fail");
    assert!(err.to_string().contains("run_as is empty"), "{err}");
}
