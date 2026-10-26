//! Fixture runs of scripts/check-core-ref.sh: the lockfile rules for both
//! modes of `bridge/CORE_REF`.

use crate::support::GateFixture;

const SHA: &str = "0123456789abcdef0123456789abcdef01234567";

fn manifest(patch_active: bool) -> String {
    let patch = if patch_active {
        "[patch.crates-io]\nsystemprompt = { path = \"../systemprompt-core/systemprompt\" }\n"
    } else {
        "# [patch.crates-io]\n"
    };
    format!(
        "[workspace.dependencies]\nsystemprompt = {{ version = \"0.72.0\", default-features = false }}\n\n{patch}"
    )
}

fn registry_lock() -> &'static str {
    "[[package]]\nname = \"systemprompt\"\nversion = \"0.72.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\n\n[[package]]\nname = \"serde\"\nversion = \"1.0.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\n"
}

fn fixture(patch_active: bool, core_ref: &str, lock: &str) -> GateFixture {
    let fx = GateFixture::new("check-core-ref.sh");
    fx.write("Cargo.toml", &manifest(patch_active))
        .write("bridge/CORE_REF", core_ref)
        .write("Cargo.lock", lock)
        .write("tests/Cargo.lock", lock);
    fx
}

#[test]
fn published_core_mode_passes_when_every_lockfile_resolves_from_the_registry() {
    let out = fixture(false, "v0.72.0\n", registry_lock()).run(&[]);
    assert!(out.passed(), "{}", out.text());
}

#[test]
fn published_core_mode_fails_on_a_path_source_in_a_tracked_lockfile() {
    let lock = registry_lock().replacen(
        "source = \"registry+https://github.com/rust-lang/crates.io-index\"",
        "source = \"path+file:///home/x/systemprompt-core/systemprompt\"",
        1,
    );
    let out = fixture(false, "v0.72.0\n", &lock).run(&[]);
    assert!(!out.passed(), "{}", out.text());
    assert!(
        out.text().contains("path+") && out.text().contains("Cargo.lock"),
        "{}",
        out.text()
    );
}

#[test]
fn patch_active_passes_when_the_pin_matches_the_sibling_and_nothing_is_unused() {
    let fx = fixture(
        true,
        &format!("{SHA}\n"),
        "[[package]]\nname = \"systemprompt\"\nversion = \"0.72.0\"\n",
    );
    let core = tempfile::tempdir().expect("core dir");
    std::fs::write(
        core.path().join("Cargo.toml"),
        "[workspace.package]\nversion = \"0.72.0\"\n",
    )
    .expect("core manifest");
    let out = fx.run(&[("SYSTEMPROMPT_CORE_DIR", core.path().to_str().expect("utf8"))]);
    assert!(out.passed(), "{}", out.text());
}

#[test]
fn patch_active_fails_on_patch_unused_or_a_pin_behind_the_sibling() {
    let unused = "[[package]]\nname = \"systemprompt\"\nversion = \"0.72.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\n\n[[patch.unused]]\nname = \"systemprompt\"\nversion = \"0.73.0\"\n";
    let out = fixture(true, &format!("{SHA}\n"), unused).run(&[]);
    assert!(
        !out.passed() && out.text().contains("[[patch.unused]]"),
        "{}",
        out.text()
    );

    let fx = fixture(
        true,
        &format!("{SHA}\n"),
        "[[package]]\nname = \"systemprompt\"\nversion = \"0.72.0\"\n",
    );
    let core = tempfile::tempdir().expect("core dir");
    std::fs::write(
        core.path().join("Cargo.toml"),
        "[workspace.package]\nversion = \"0.73.0\"\n",
    )
    .expect("core manifest");
    let out = fx.run(&[("SYSTEMPROMPT_CORE_DIR", core.path().to_str().expect("utf8"))]);
    assert!(
        !out.passed() && out.text().contains("0.73.0"),
        "{}",
        out.text()
    );
}
