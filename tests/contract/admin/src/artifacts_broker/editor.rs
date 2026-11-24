//! The Drive's editor page as a client opens it: `/drive/{id}` with the web's
//! whole chrome, and with `?embed=1` without what takes a person away from the
//! document, so a client that shows the page as a view of its own draws no
//! second app chrome around it.

use reqwest::header::LOCATION;
use systemprompt::identifiers::DocumentId;

use super::stack::Stack;

const CHROME: [&str; 6] = [
    "sp-ws__home",
    "sp-ws__title",
    "sp-ws__format",
    "data-notification-bell",
    "sp-ws__rail",
    "data-slot=\"ws-outline\"",
];

const DOCUMENT: [&str; 6] = [
    "data-ws-root",
    "data-command=\"comments\"",
    "data-command=\"details\"",
    "data-command=\"share\"",
    "data-slot=\"ws-editor\"",
    "/js/workspace/shell.js",
];

fn html_tag(body: &str) -> &str {
    let start = body.find("<html").expect("the page opens an html element");
    let end = body[start..].find('>').expect("the html tag closes");
    &body[start..start + end]
}

async fn load(stack: &Stack, path: &str, cookie: &str) -> String {
    let response = stack.navigate(path, Some(cookie)).await;
    assert_eq!(response.status(), 200, "the owner's editor page: {path}");
    response.text().await.expect("read the editor page")
}

pub(super) async fn the_editor_has_an_embed_mode(stack: &Stack, owner: &str, page: &DocumentId) {
    let path = format!("/drive/{}", page.as_str());

    let full = load(stack, &path, owner).await;
    assert!(
        !html_tag(&full).contains("data-embed"),
        "the plain page is not marked as embedded: {}",
        html_tag(&full)
    );
    for needed in CHROME.into_iter().chain(DOCUMENT) {
        assert!(full.contains(needed), "the plain page has `{needed}`");
    }
    assert!(
        !full.contains("target=\"_blank\" rel=\"noopener\">Back to Drive"),
        "the plain page's links leave in place"
    );

    let embedded = load(stack, &format!("{path}?embed=1"), owner).await;
    assert!(
        html_tag(&embedded).contains("data-embed"),
        "the embedded page marks the html element: {}",
        html_tag(&embedded)
    );
    for gone in CHROME {
        assert!(
            !embedded.contains(gone),
            "the embedded page leaves out `{gone}`"
        );
    }
    for kept in DOCUMENT {
        assert!(embedded.contains(kept), "the embedded page keeps `{kept}`");
    }
    assert!(
        embedded.contains("target=\"_blank\" rel=\"noopener\">Back to Drive"),
        "the one link out of the document opens apart"
    );

    for other in ["0", "true", ""] {
        let plain = load(stack, &format!("{path}?embed={other}"), owner).await;
        assert!(
            !html_tag(&plain).contains("data-embed"),
            "`?embed={other}` is not the switch: {}",
            html_tag(&plain)
        );
    }
}

pub(super) async fn the_embedded_editor_signs_in_first(stack: &Stack, page: &DocumentId) {
    let anonymous = stack
        .navigate(&format!("/drive/{}?embed=1", page.as_str()), None)
        .await;
    assert!(
        anonymous.status().is_redirection(),
        "no session goes to sign in: {}",
        anonymous.status()
    );
    let back = anonymous
        .headers()
        .get(LOCATION)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    assert_eq!(
        back,
        format!(
            "/admin/login?redirect=%2Fdrive%2F{}%3Fembed%3D1",
            page.as_str()
        ),
        "the web login comes back to the embedded editor"
    );
}
