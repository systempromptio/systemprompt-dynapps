//! Fixture runs of scripts/check-claude-code-version.sh: one agreed
//! `ARG CLAUDE_CODE_VERSION` default across every Dockerfile that installs
//! Claude Code, and no unpinned install.

use crate::support::GateFixture;

const SERVER: &str = "FROM debian\nARG CLAUDE_CODE_VERSION=2.1.296\nRUN curl -fsSL https://claude.ai/install.sh | bash -s \"${CLAUDE_CODE_VERSION}\"\n";
const CLIENT: &str = "FROM node\nARG CLAUDE_CODE_VERSION=2.1.296\nRUN npm install -g \"@anthropic-ai/claude-code@${CLAUDE_CODE_VERSION}\"\n";

fn fixture(server: &str, client: &str) -> GateFixture {
    let fx = GateFixture::new("check-claude-code-version.sh");
    fx.write("Dockerfile", server)
        .write("deploy/clean-client/Dockerfile", client)
        .write("deploy/other/Dockerfile.mock", "FROM scratch\n");
    fx
}

#[test]
fn agreeing_pins_pass() {
    let out = fixture(SERVER, CLIENT).run(&[]);
    assert!(out.passed(), "{}", out.text());
    assert!(out.stdout.contains("2.1.296"), "{}", out.text());
}

#[test]
fn latest_an_unpinned_installer_and_disagreeing_defaults_each_fail() {
    let latest = CLIENT.replace(
        "\"@anthropic-ai/claude-code@${CLAUDE_CODE_VERSION}\"",
        "@anthropic-ai/claude-code@latest",
    );
    let out = fixture(SERVER, &latest).run(&[]);
    assert!(
        !out.passed() && out.text().contains("@latest"),
        "{}",
        out.text()
    );

    let unpinned = SERVER.replace(" -s \"${CLAUDE_CODE_VERSION}\"", "");
    let out = fixture(&unpinned, CLIENT).run(&[]);
    assert!(
        !out.passed() && out.text().contains("install.sh"),
        "{}",
        out.text()
    );

    let drifted = CLIENT.replace("2.1.296", "2.1.300");
    let out = fixture(SERVER, &drifted).run(&[]);
    assert!(
        !out.passed() && out.text().contains("disagree"),
        "{}",
        out.text()
    );
}
