//! Fixture runs of scripts/check-gate-parity.sh: every gate core runs has a
//! wrapper or a decided line in scripts/core-gate-map.txt, and no line is
//! stale.

use crate::support::GateFixture;

fn core_with_gates(gates: &str) -> tempfile::TempDir {
    let core = tempfile::tempdir().expect("core dir");
    std::fs::create_dir_all(core.path().join("scripts")).expect("core scripts");
    std::fs::write(core.path().join("scripts/check-gates.txt"), gates).expect("core gate list");
    core
}

fn fixture(map: &str) -> GateFixture {
    let fx = GateFixture::new("check-gate-parity.sh");
    fx.write("bridge/CORE_REF", "v0.72.0\n")
        .write("scripts/core-gate-map.txt", map)
        .write(
            "scripts/lint-owned-tasks.sh",
            "#!/usr/bin/env bash\nexec bash scripts/core-gate.sh lint-owned-tasks.sh extensions\n",
        )
        .write("scripts/check-sqlx.sh", "#!/usr/bin/env bash\n")
        .write("justfile", "lint-no-untyped-admin:\n    true\n");
    fx
}

const CORE_GATES: &str =
    "lint-owned-tasks\nlint-sqlx\nlint-no-untyped-admin\nlint-bridge-i18n\nlint-async-trait\n";
const MAP: &str = "# decided\nlint-sqlx  install:check-sqlx.sh\nlint-no-untyped-admin  install:justfile:lint-no-untyped-admin\nlint-bridge-i18n  n/a: no bridge GUI here\nlint-async-trait  wrapper\n";

#[test]
fn a_fully_decided_map_passes() {
    let fx = fixture(MAP);
    fx.write(
        "scripts/lint-async-trait.sh",
        "#!/usr/bin/env bash\nexec bash scripts/core-gate.sh lint-async-trait.sh extensions\n",
    );
    let core = core_with_gates(CORE_GATES);
    let out = fx.run(&[("SYSTEMPROMPT_CORE_DIR", core.path().to_str().expect("utf8"))]);
    assert!(out.passed(), "{}", out.text());
    assert!(out.stdout.contains("5 core gates"), "{}", out.text());
}

#[test]
fn an_undecided_core_gate_a_stale_line_and_a_missing_wrapper_each_fail() {
    let core = core_with_gates(CORE_GATES);
    let core_dir = core.path().to_str().expect("utf8");

    let out = fixture(MAP).run(&[("SYSTEMPROMPT_CORE_DIR", core_dir)]);
    assert!(
        !out.passed() && out.text().contains("lint-async-trait is 'wrapper'"),
        "{}",
        out.text()
    );

    let fx = fixture(&MAP.replace("lint-async-trait  wrapper\n", ""));
    let out = fx.run(&[("SYSTEMPROMPT_CORE_DIR", core_dir)]);
    assert!(
        !out.passed()
            && out
                .text()
                .contains("neither scripts/lint-async-trait.sh nor a line"),
        "{}",
        out.text()
    );

    let fx = fixture(&format!("{MAP}lint-retired  n/a: gone\n"));
    fx.write(
        "scripts/lint-async-trait.sh",
        "#!/usr/bin/env bash\nexec bash scripts/core-gate.sh lint-async-trait.sh extensions\n",
    );
    let out = fx.run(&[("SYSTEMPROMPT_CORE_DIR", core_dir)]);
    assert!(
        !out.passed() && out.text().contains("core no longer runs 'lint-retired'"),
        "{}",
        out.text()
    );
}
