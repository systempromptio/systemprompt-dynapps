//! Every word the mobile sign-in pages show.
//!
//! English, like the desktop sign-in. `%device%`, `%instance%` and
//! `%account%` are filled in, escaped, by the page renderer.

pub const CONSENT_TITLE: &str = "Sign in to DynWork on your phone";
pub const CONSENT_HEADING: &str = "Allow DynWork on %device%?";
pub const CONSENT_BODY: &str = "DynWork on %device% wants to sign in to %instance% as %account%. \
     It will be able to reach your DynWork desktops through %instance% until you remove it \
     from your devices.";
pub const CONSENT_WARNING: &str =
    "Only allow this if you started signing in from the DynWork app just now.";
pub const ALLOW: &str = "Allow";
pub const DENY: &str = "Deny";

pub const ERROR_TITLE: &str = "DynWork sign-in";
pub const INVALID_REQUEST: &str =
    "This sign-in link is not valid. Start again from the DynWork app.";
pub const EXPIRED_REQUEST: &str =
    "This sign-in request has expired or was already answered. Start again from the DynWork app.";
pub const TOO_MANY_REQUESTS: &str = "There are too many sign-in requests open. Wait ten minutes and start again from the DynWork app.";
pub const SIGNED_OUT: &str =
    "You are no longer signed in to this instance. Start again from the DynWork app.";
pub const UNAVAILABLE: &str = "The sign-in could not be completed. Try again in a moment.";
