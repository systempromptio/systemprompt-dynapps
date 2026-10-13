//! The secrets card on `/admin/configuration` and its **Reload now** action
//! (`POST /admin/platform/secrets/reload`), under the three principals.
//!
//! The fixture profile loads secrets from a file, so a reload re-reads the
//! same file: it must succeed with no keys changed and land back on the
//! configuration page with an `ok` flash. Only an admin reaches the handler;
//! the page must name the source and the key names and never a value.

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal};

const RELOAD: &str = "/admin/platform/secrets/reload";
const CONFIGURATION: &str = "/admin/configuration";
// A value from the fixture secrets file; the card may never render it.
const PEPPER_VALUE: &str = "contract-suite-pepper-not-a-real-secret";

fn reload(principal: Principal) -> Call<'static> {
    Call {
        method: "post",
        path: RELOAD,
        principal,
        content_type: Some("application/x-www-form-urlencoded"),
        body: Some(""),
    }
}

fn origin() -> String {
    let profile = systemprompt::config::ProfileBootstrap::get().expect("fixture profile");
    url::Url::parse(&profile.server.api_external_url)
        .expect("fixture origin")
        .origin()
        .ascii_serialization()
}

#[tokio::test(flavor = "multi_thread")]
async fn secrets_reload_is_admin_only_and_redirects_with_a_flash() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let app = App::new(&db.pool, principal::provision(&db.pool).await);
    let origin = origin();
    let headers = [("origin", origin.as_str())];

    let (status, location) = app
        .redirect_with_headers(reload(Principal::Admin), &headers)
        .await;
    assert_eq!(status.as_u16(), 303, "admin reload -> {location:?}");
    assert!(
        location.starts_with(CONFIGURATION),
        "admin lands on the page: {location}"
    );
    assert!(
        location.contains("ok"),
        "a same-file reload succeeds: {location}"
    );

    let (status, location) = app
        .redirect_with_headers(reload(Principal::NonAdmin), &headers)
        .await;
    assert_eq!(status.as_u16(), 303, "non-admin -> {location:?}");
    assert!(
        !location.starts_with(CONFIGURATION),
        "non-admin never reaches the handler"
    );

    let (status, location) = app
        .redirect_with_headers(reload(Principal::Anonymous), &headers)
        .await;
    assert_eq!(status.as_u16(), 307, "anonymous -> {location:?}");
    assert!(location.contains("/admin/login"), "{location}");

    // Same-origin is required of the admin too.
    let (status, _) = app.call(reload(Principal::Admin)).await;
    assert!(
        !status.is_success() && !status.is_server_error(),
        "a cross-origin reload is refused without a 5xx: {status}"
    );
    db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn configuration_page_renders_the_secrets_card_without_values() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let app = App::new(&db.pool, principal::provision(&db.pool).await);
    let (status, body) = app.call(Call::get(CONFIGURATION, Principal::Admin)).await;
    assert_eq!(status.as_u16(), 200);
    assert!(
        body.contains("data-secrets-source=\"file\""),
        "the card names the source"
    );
    assert!(body.contains(RELOAD), "the card offers Reload now");
    assert!(
        body.contains("oauth_at_rest_pepper"),
        "key names are listed"
    );
    assert!(
        !body.contains(PEPPER_VALUE),
        "a secret value is never rendered"
    );
    db.cleanup().await;
}
