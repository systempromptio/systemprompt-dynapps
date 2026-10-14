//! Shared harness for every test crate in this repository.
//!
//! The rule the crate exists to enforce: a test that cannot run says so. A
//! missing database is a skip on a developer machine and a failure under `CI`,
//! decided in one place ([`skip`]) rather than five times over with four of
//! them getting it wrong.

pub mod fake_http;
pub mod onepassword_fake;
pub mod onepassword_fake_cli;
pub mod orphans;
pub mod paths;
pub mod skip;
pub mod sso;
pub mod tempdb;

pub use fake_http::{RecordedRequest, RequestLog};
pub use onepassword_fake::FakeConnect;
pub use onepassword_fake_cli::FakeOpCli;
pub use paths::{repo_path, repo_root};
pub use skip::{ci, skip_or_panic};
pub use sso::{
    FakeOidcIdp, IdTokenSpec, ParsedRedirect, SamlIdpFixture, SamlResponseSpec, Tamper,
    TestIdpKeys, parse_authn_request_redirect,
};
pub use tempdb::TempDb;

// Why: the sanctioned way for a DB-backed test to acquire its database. The
// `return` is unreachable under CI -- `TempDb::create` panics first -- so the
// early exit is a developer-machine convenience, not a hole in the tier.
#[macro_export]
macro_rules! db_or_skip {
    () => {{
        let Some(db) = $crate::TempDb::create().await else {
            return; // skip-ok: TempDb::create panics under CI
        };
        db
    }};
}

#[macro_export]
macro_rules! empty_db_or_skip {
    () => {{
        let Some(db) = $crate::TempDb::create_empty().await else {
            return; // skip-ok: TempDb::create_empty panics under CI
        };
        db
    }};
}
