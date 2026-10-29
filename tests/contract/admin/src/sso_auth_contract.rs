//! "Sign in with Microsoft" (Entra ID over `OpenID Connect`) through core's
//! SSO driver — the branches reachable without a live tenant.
//!
//! SSO reports failure by *redirecting*, not by returning an error status: a
//! `500` on the callback would strand the browser on a dead end instead of
//! returning it to a usable login page. Every case here therefore asserts on
//! the `Location` header — `?sso=<reason>` — because the status is `303` on
//! success and failure alike and proves nothing on its own.
//!
//! [`crate::app::App::new`] discovers the Entra provider from
//! `services/web/config/entra-oidc.yaml` and the fixture's client secret, so
//! every start route hands the browser to Microsoft and a callback whose
//! state this server never issued reports the correlation lost. The token
//! exchange runs against a fake
//! `IdP` in `tests/integration/admin-core` (`sso_oidc_*`). Core's generic
//! `/api/v1/core/sso/{id}/*` routes are mounted by core, not this router, and
//! are not driven here.

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal};

const OIDC_START: &str = "/admin/auth/entra/start";
const OIDC_START_REDIRECT: &str = "/admin/auth/entra/start?redirect=/admin/profile";
const OIDC_CALLBACK: &str = "/admin/auth/entra/callback?code=abc&state=xyz";
const IDP_AUTHORIZE: &str = "https://login.microsoftonline.com/";
const CORRELATION_LOST: &str = "/admin/login?sso=correlation_lost";
const PRINCIPALS: [Principal; 3] = [Principal::Anonymous, Principal::NonAdmin, Principal::Admin];

// Starting sign-in leaves for the IdP; a callback carrying a state no start
// issued is refused rather than exchanged.
#[tokio::test(flavor = "multi_thread")]
async fn sso_routes_start_at_the_idp_and_refuse_a_foreign_state() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        eprintln!("no DATABASE_URL — skipping SSO suite");
        return;
    };
    let app = App::new(&db.pool, principal::provision(&db.pool).await);

    let mut failures = Vec::new();
    for principal in PRINCIPALS {
        for (call, expected) in [
            (Call::get(OIDC_START, principal), IDP_AUTHORIZE),
            (Call::get(OIDC_START_REDIRECT, principal), IDP_AUTHORIZE),
            (Call::get(OIDC_CALLBACK, principal), CORRELATION_LOST),
        ] {
            let path = call.path;
            let (status, location) = app.redirect_of(call).await;
            if !status.is_redirection() || !location.contains(expected) {
                failures.push(format!(
                    "  {path} as {principal:?} -> {} {location:?}, expected a redirect to {expected}",
                    status.as_u16()
                ));
            }
        }
    }

    db.cleanup().await;
    assert!(
        failures.is_empty(),
        "{} SSO case(s) failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
