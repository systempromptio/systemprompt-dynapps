//! Mobile sign-in, path 1.
//!
//! The `DynWork` phone app signs in to this instance the way the desktop
//! does, through the system browser and the instance's own login, and leaves
//! with a revocable device credential.
//!
//! 1. The app opens `GET /authorize?redirect_uri=dynwork://auth/callback&
//!    code_challenge&state&device_name` in the system browser. A signed-out
//!    browser goes through `/admin/login` (Entra on the Dynapps brand) and
//!    back; a signed-in one sees a consent page in the instance's branding.
//! 2. Allow answers `303` to `dynwork://auth/callback?code&state` with a
//!    single-use code that lives ten minutes and is bound to the challenge.
//! 3. `POST /token` trades the code, its PKCE S256 verifier and the phone's
//!    peer id for a `dwm1_` mobile credential and a one-hour JWT. The
//!    credential is a phone device row of the relay, so it shows up in, and is
//!    revoked from, the same device list as paired phones, and it opens the
//!    device socket through [`MobileCredentialVerifier`].
//! 4. `POST /refresh` mints a fresh JWT from the credential, `GET /session`
//!    tells a JWT's account and device, `POST /revoke` signs the phone out.

pub mod authorize;
pub mod branding;
pub mod copy;
pub mod jwt;
pub mod page;
pub mod params;
pub mod session;
pub mod token;
pub mod verifier;

use axum::Router;
use axum::routing::{get, post};

pub use branding::Branding;
pub use verifier::MobileCredentialVerifier;

use crate::state::RelayState;

pub const MOBILE_BASE_PATH: &str = "/api/dynwork/mobile/v1";
pub const DECISION_PATH: &str = "/api/dynwork/mobile/v1/authorize/decision";
pub const REDIRECT_URI: &str = "dynwork://auth/callback";
pub const LOGIN_PATH: &str = "/admin/login";
pub const CODE_TTL: chrono::Duration = chrono::Duration::minutes(10);
pub const OPEN_GRANTS_PER_USER: u32 = 10;

pub fn mobile_router(state: RelayState) -> Router {
    Router::new()
        .route("/discovery", get(token::discovery))
        .route("/authorize", get(authorize::show))
        .route("/authorize/decision", post(authorize::decide))
        .route("/token", post(token::exchange))
        .route("/refresh", post(token::refresh))
        .route("/session", get(token::current))
        .route("/revoke", post(token::revoke))
        .with_state(state)
}
