//! Generates one test per release schema on the ladder.
//!
//! A single test that walks every rung grows with each release and crossed
//! nextest's timeout once the ladder reached 19 rungs. One test per rung keeps
//! each test's cost flat, lets nextest schedule them in parallel, and names the
//! failing release in the test id.

use std::fmt::Write as _;
use std::path::Path;

const FIXTURE_DIR: &str = "../../fixtures/schema";
const FIXTURE_PREFIX: &str = "release-baseline-";

fn main() {
    println!("cargo:rerun-if-changed={FIXTURE_DIR}");
    let mut versions: Vec<String> = std::fs::read_dir(FIXTURE_DIR)
        .unwrap_or_else(|e| panic!("read {FIXTURE_DIR}: {e}"))
        .filter_map(|entry| {
            let name = entry.ok()?.file_name().into_string().ok()?;
            Some(
                name.strip_prefix(FIXTURE_PREFIX)?
                    .strip_suffix(".sql")?
                    .to_owned(),
            )
        })
        .collect();
    versions.sort();

    let mut out = String::new();
    for version in &versions {
        let ident = version.replace('.', "_");
        writeln!(
            out,
            "#[tokio::test(flavor = \"multi_thread\", worker_threads = 2)]\n\
             async fn release_{ident}_upgrades_to_a_fresh_install() {{\n    \
             check_release(\"{version}\").await;\n}}\n"
        )
        .expect("write to a String");
    }
    let dest = Path::new(&std::env::var("OUT_DIR").expect("OUT_DIR")).join("rung_tests.rs");
    std::fs::write(&dest, out).unwrap_or_else(|e| panic!("write {}: {e}", dest.display()));
}
