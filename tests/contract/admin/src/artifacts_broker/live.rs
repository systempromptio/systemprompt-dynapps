//! The live page as a client opens it: the viewer shell at
//! `/drive/{id}/live`, the page in its sandboxed frame at
//! `/drive/{id}/live/frame` with the runtime inlined and a policy that keeps
//! it opaque and offline, and the manifest a client reads to decide whether
//! to open the shell, with the web session or the person's `ui` session.

use reqwest::header::{CONTENT_SECURITY_POLICY, LOCATION, X_FRAME_OPTIONS};
use systemprompt::identifiers::DocumentId;

use super::stack::{As, Stack};

pub(super) struct Who<'a> {
    pub(super) owner: &'a str,
    pub(super) owner_ui: &'a str,
    pub(super) agent: &'a str,
}

// Before the page is shared with them, a person who cannot see it gets no
// frame, no shell and no manifest, by cookie or by `ui` session.
pub(super) async fn a_stranger_gets_nothing(
    stack: &Stack,
    cookie: &str,
    ui: &str,
    page: &DocumentId,
) {
    for path in [
        format!("/drive/{}/live/frame", page.as_str()),
        format!("/drive/{}/live", page.as_str()),
    ] {
        let hidden = stack.navigate(&path, Some(cookie)).await;
        assert_eq!(
            hidden.status(),
            404,
            "{path} for a person who cannot see it"
        );
    }
    let manifest = stack
        .send(
            reqwest::Method::GET,
            &format!("/api/v1/artifacts/{}/manifest", page.as_str()),
            As::Bearer(ui),
            None,
        )
        .await;
    assert_eq!(
        manifest.status, 404,
        "a person who cannot see the page learns nothing: {}",
        manifest.body
    );
}

fn header(response: &reqwest::Response, name: &reqwest::header::HeaderName) -> String {
    response
        .headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned()
}

pub(super) async fn the_frame_is_opaque_and_offline(
    stack: &Stack,
    who: &Who<'_>,
    page: &DocumentId,
) {
    let path = format!("/drive/{}/live/frame", page.as_str());
    let framed = stack.navigate(&path, Some(who.owner)).await;
    assert_eq!(framed.status(), 200, "the owner's frame loads");
    let csp = header(&framed, &CONTENT_SECURITY_POLICY);
    let frame_options = header(&framed, &X_FRAME_OPTIONS);
    let body = framed.text().await.expect("read the frame");
    for needed in [
        "sandbox allow-scripts;",
        "default-src 'none'",
        "connect-src 'none'",
        "form-action 'none'",
        "frame-ancestors 'self'",
    ] {
        assert!(
            csp.contains(needed),
            "the frame's policy has `{needed}`: {csp}"
        );
    }
    assert!(
        !csp.contains("allow-same-origin"),
        "the frame never shares the site's origin: {csp}"
    );
    assert_eq!(
        frame_options, "SAMEORIGIN",
        "only this site frames the page"
    );
    let nonce = csp
        .split("'nonce-")
        .nth(1)
        .and_then(|rest| rest.split('\'').next())
        .expect("with the instance's scripts off, only the runtime runs, by its nonce");
    let runtime = body
        .find(&format!("<script nonce=\"{nonce}\">"))
        .expect("the runtime is inlined with the policy's nonce");
    assert!(
        body.contains("governance-live/1"),
        "the runtime speaks the shell's protocol"
    );
    assert!(
        body.contains(&stack.origin),
        "the runtime knows the shell's origin"
    );
    assert!(
        !body.contains("/*HOST*/null"),
        "the host marker is filled in"
    );
    let page_text = body
        .find("published from the page")
        .expect("the frame serves the page's own HTML");
    assert!(runtime < page_text, "the runtime runs before the page");

    let bad_revision = stack
        .navigate(&format!("{path}?revision=nope"), Some(who.owner))
        .await;
    assert_eq!(
        bad_revision.status(),
        404,
        "a revision that is not one is not found"
    );
}

pub(super) async fn the_shell_signs_in_first(stack: &Stack, who: &Who<'_>, page: &DocumentId) {
    let path = format!("/drive/{}/live?embed=1", page.as_str());
    let anonymous = stack.navigate(&path, None).await;
    assert!(
        anonymous.status().is_redirection(),
        "no session goes to sign in: {}",
        anonymous.status()
    );
    let expected = format!(
        "/admin/login?redirect=%2Fdrive%2F{}%2Flive%3Fembed%3D1",
        page.as_str()
    );
    assert_eq!(
        header(&anonymous, &LOCATION),
        expected,
        "the web login comes back to the embedded shell"
    );

    let embedded = stack.navigate(&path, Some(who.owner)).await;
    assert_eq!(embedded.status(), 200, "the owner's shell loads");
    let body = embedded.text().await.expect("read the shell");
    assert!(
        body.contains("data-live-root"),
        "the shell mounts on its root"
    );
    assert!(body.contains("sp-live-page--embed"), "embed mode is marked");
    assert!(
        !body.contains("sp-live-page__bar"),
        "embed mode leaves the bar out"
    );
    assert!(
        body.contains("/js/workspace/live/page.js"),
        "the shell's script loads"
    );
    let full = stack
        .navigate(&format!("/drive/{}/live", page.as_str()), Some(who.owner))
        .await;
    assert!(
        full.text()
            .await
            .expect("read the shell")
            .contains("sp-live-page__bar"),
        "the full shell has its bar"
    );
}

pub(super) async fn a_client_reads_the_manifest(
    stack: &Stack,
    who: &Who<'_>,
    page: &DocumentId,
    undeclared: &DocumentId,
) {
    let path = |document: &DocumentId| format!("/api/v1/artifacts/{}/manifest", document.as_str());
    let by_cookie = stack
        .send(
            reqwest::Method::GET,
            &path(page),
            As::Cookie(who.owner),
            None,
        )
        .await;
    assert_eq!(by_cookie.status, 200, "{}", by_cookie.body);
    let manifest = by_cookie.json();
    assert_eq!(manifest["declared"], true);
    assert_eq!(manifest["contract"], "0.2.99");
    assert!(manifest["capabilities"]["mcp"]["servers"].is_array());
    let live_url = manifest["live_url"].as_str().expect("a live URL");
    assert!(
        live_url.starts_with("http")
            && live_url.ends_with(&format!("/drive/{}/live", page.as_str())),
        "the live URL is the shell's absolute URL: {live_url}"
    );

    let by_ui = stack
        .send(
            reqwest::Method::GET,
            &path(page),
            As::Bearer(who.owner_ui),
            None,
        )
        .await;
    assert_eq!(
        by_ui.status, 200,
        "the person's ui session reads it: {}",
        by_ui.body
    );
    assert_eq!(by_ui.json(), manifest);

    let plain = stack
        .send(
            reqwest::Method::GET,
            &path(undeclared),
            As::Bearer(who.owner_ui),
            None,
        )
        .await;
    assert_eq!(plain.status, 200, "{}", plain.body);
    assert_eq!(
        plain.json(),
        serde_json::json!({ "declared": false, "capabilities": null, "contract": null, "live_url": null, "servers": [] })
    );

    let agent = stack
        .send(
            reqwest::Method::GET,
            &path(page),
            As::Bearer(who.agent),
            None,
        )
        .await;
    assert_eq!(
        agent.status, 401,
        "an agent's bearer is not the person's ui session"
    );
    assert_eq!(agent.key(), "reauth");
    let anonymous = stack
        .send(reqwest::Method::GET, &path(page), As::Nobody, None)
        .await;
    assert_eq!(anonymous.status, 401);
}
