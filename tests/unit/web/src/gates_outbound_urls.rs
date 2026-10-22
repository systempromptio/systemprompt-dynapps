//! `scripts/check-outbound-urls.sh` over a temporary tree: a request site
//! without the guard fails, a guarded one passes, and a `.get(` on an
//! ordinary map in a file that never names reqwest is not a request.

use std::path::Path;
use std::process::Command;

use crate::support::repo_root;

fn run(root: &Path) -> (bool, String) {
    let out = Command::new("bash")
        .arg(repo_root().join("scripts/check-outbound-urls.sh"))
        .arg(root)
        .output()
        .expect("run the gate");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn tree(files: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    for (name, body) in files {
        std::fs::write(dir.path().join(name), body).expect("write sample");
    }
    dir
}

#[test]
fn an_unguarded_request_site_fails_and_is_named() {
    let dir = tree(&[(
        "send.rs",
        "use reqwest::Client;\nasync fn f(http: &Client, u: &str) {\n    let _ = http.get(u).send().await;\n}\n",
    )]);
    let (ok, stderr) = run(dir.path());
    assert!(!ok, "an unguarded send must fail the gate");
    assert!(stderr.contains("send.rs"), "{stderr}");
}

#[test]
fn a_guarded_request_site_passes() {
    let dir = tree(&[(
        "send.rs",
        "use reqwest::Client;\nuse systemprompt::models::net::validate_outbound_url;\nasync fn f(http: &Client, u: &str) {\n    let _ = http\n        .post(validate_outbound_url(u)?)\n        .send()\n        .await;\n}\n",
    )]);
    let (ok, stderr) = run(dir.path());
    assert!(ok, "{stderr}");
}

#[test]
fn a_map_lookup_in_a_file_without_reqwest_is_not_a_request() {
    let dir = tree(&[(
        "lookup.rs",
        "fn g(client: &std::collections::HashMap<u8, u8>) -> Option<&u8> {\n    client.get(&1)\n}\n",
    )]);
    let (ok, stderr) = run(dir.path());
    assert!(ok, "{stderr}");
}
