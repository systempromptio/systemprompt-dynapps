//! The binary's argument grammar: no arguments serves, `ingest` runs once
//! with optional `--rebuild` and repeatable `--source`, and anything else is
//! refused with usage rather than silently serving.

use systemprompt_mcp_knowledge_bank::cli::{Command, parse};

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| (*s).to_owned()).collect()
}

#[test]
fn no_arguments_or_serve_means_serve() {
    assert_eq!(parse(args(&[])).expect("parses"), Command::Serve);
    assert_eq!(parse(args(&["serve"])).expect("parses"), Command::Serve);
}

#[test]
fn ingest_collects_rebuild_and_sources() {
    let Command::Ingest(options) = parse(args(&[
        "ingest",
        "--rebuild",
        "--source",
        "adrs",
        "--source",
        "jira",
    ]))
    .expect("parses") else {
        panic!("expected ingest");
    };
    assert!(options.rebuild);
    assert_eq!(options.sources, vec!["adrs", "jira"]);
}

#[test]
fn unknown_commands_and_flags_are_refused_with_usage() {
    let err = parse(args(&["frobnicate"])).expect_err("unknown command");
    assert!(err.contains("unknown command"));
    assert!(err.contains("ingest"));
    let err = parse(args(&["ingest", "--fast"])).expect_err("unknown flag");
    assert!(err.contains("--fast"));
    let err = parse(args(&["ingest", "--source"])).expect_err("missing id");
    assert!(err.contains("category id"));
}

#[test]
fn help_is_recognised() {
    assert_eq!(parse(args(&["--help"])).expect("parses"), Command::Help);
}
