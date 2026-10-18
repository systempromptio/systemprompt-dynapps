//! The container entrypoint reads `server.role` from the profile so a gateway
//! node skips the admin bootstrap and the site render.

use std::process::Command;

use crate::support::repo_root;

fn role_of(profile: &str) -> String {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("profile.yaml");
    std::fs::write(&path, profile).expect("write profile");
    let entrypoint = repo_root().join("docker/entrypoint.sh");
    let script = format!(
        "eval \"$(sed -n '/^server_role() {{/,/^}}/p' '{}')\"; PROFILE_FILE='{}' server_role",
        entrypoint.display(),
        path.display()
    );
    let out = Command::new("bash")
        .arg("-c")
        .arg(script)
        .output()
        .expect("run bash");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_owned()
}

#[test]
fn the_entrypoint_reads_the_server_role() {
    assert_eq!(
        role_of("database:\n  url: x\nserver:\n  host: 0.0.0.0\n  role: gateway\n"),
        "gateway"
    );
    assert_eq!(role_of("server:\n  role: \"admin\" # split\n"), "admin");
    assert_eq!(
        role_of("server:\n  host: 0.0.0.0\nworkers:\n  role: gateway\n"),
        "",
        "a role outside the server block is not the node's role"
    );
}

#[test]
fn a_gateway_node_skips_the_bootstrap_and_no_node_renders_twice() {
    let text =
        std::fs::read_to_string(repo_root().join("docker/entrypoint.sh")).expect("read entrypoint");
    let gateway = text
        .find("if [ \"$ROLE\" = \"gateway\" ]; then")
        .expect("the entrypoint branches on the gateway role");
    let bootstrap = text
        .rfind("/app/bin/systemprompt admin bootstrap")
        .expect("the boot-time admin bootstrap");
    let start = text.find("infra services start").expect("the server start");
    assert!(gateway < bootstrap && bootstrap < start);
    assert!(
        !text.contains("infra jobs run publish_pipeline")
            && !text.contains("infra jobs run site_render"),
        "web.dist is rendered by the scheduler's bootstrap site_render, not again here"
    );
    let scheduler = std::fs::read_to_string(repo_root().join("services/scheduler/config.yaml"))
        .expect("read scheduler config");
    let bootstrap_jobs = &scheduler[scheduler.find("bootstrap_jobs:").expect("bootstrap list")..];
    assert!(bootstrap_jobs.contains("- site_render"));
    assert!(!bootstrap_jobs.contains("- publish_pipeline"));
}
