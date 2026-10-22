//! Fixture runs of scripts/lint-authoritative-reads.sh over copies of the real
//! identity routes and admin router: the tree as committed passes, and moving a
//! revoking route group onto the read pool fails.

use crate::support::GateFixture;

const FILES: &[&str] = &[
    "extensions/web/admin/identity/src/routes/api.rs",
    "extensions/web/src/router/admin/api.rs",
    "extensions/web/src/router/api.rs",
    "extensions/web/src/router/sign_out.rs",
    "extensions/web/admin/identity/src/sign_out.rs",
    "extensions/web/admin/identity/src/handlers/user_sessions.rs",
    "extensions/web/admin/identity/src/handlers/roles.rs",
    "extensions/web/admin/identity/src/handlers/dev_login.rs",
    "extensions/web/admin/identity/src/sso/roles.rs",
    "extensions/web/jobs/src/access_expiry.rs",
    "extensions/cli/dev-login/src/main.rs",
];

fn fixture() -> GateFixture {
    let fx = GateFixture::new("lint-authoritative-reads.sh");
    for rel in FILES {
        fx.copy_from_repo(rel);
    }
    fx
}

#[test]
fn the_committed_wiring_passes() {
    let out = fixture().run(&[]);
    assert!(out.passed(), "{}", out.text());
}

#[test]
fn a_revoking_group_on_the_read_pool_and_a_caller_without_the_write_pool_fail() {
    let fx = fixture();
    let router = "extensions/web/src/router/admin/api.rs";
    let text = std::fs::read_to_string(fx.root().join(router)).expect("router copy");
    let moved = text
        .replacen("        .merge(identity_routes::session_write_routes())\n", "", 1)
        .replacen(
            "        .merge(identity_routes::user_read_routes())\n",
            "        .merge(identity_routes::user_read_routes())\n        .merge(identity_routes::session_write_routes())\n",
            1,
        );
    assert_ne!(
        moved, text,
        "the fixture must move session_write_routes onto the read router"
    );
    fx.write(router, &moved);
    let out = fx.run(&[]);
    assert!(!out.passed(), "{}", out.text());
    assert!(
        out.text().contains("session_write_routes") && out.text().contains("read pool"),
        "{}",
        out.text()
    );

    let fx = fixture();
    let job = "extensions/web/jobs/src/access_expiry.rs";
    let text = std::fs::read_to_string(fx.root().join(job)).expect("job copy");
    let read_pool = text.replace("db.write_pool()", "db.pool()");
    assert_ne!(read_pool, text);
    fx.write(job, &read_pool);
    let out = fx.run(&[]);
    assert!(
        !out.passed() && out.text().contains("access_expiry.rs"),
        "{}",
        out.text()
    );
}
