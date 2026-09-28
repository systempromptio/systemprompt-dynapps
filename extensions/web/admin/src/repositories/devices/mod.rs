//! Reads and revocations for the device fleet: bridge sessions, personal
//! access tokens, device certificates, and enrolment links still unclaimed.
//!
//! The four are one page because they are one question — "what is holding a
//! credential to this platform, and is it still alive?" — asked of four
//! tables. They are separate modules because nothing joins them: a bridge
//! session is a heartbeat, a token is a secret, a certificate is a key, and a
//! link is a promise. Only the person they belong to is common, so every row
//! type carries a `UserId` and the page renders it as the same link.
//!
//! Presence is one rule, `repositories::bridge::presence::bridge_presence` in
//! `systemprompt_web_admin_core`, applied to a heartbeat, so the count on the
//! KPI tile, the badge on the row and the user page can never disagree about
//! what online or stale means.

pub mod certs;
pub mod links;
pub mod pats;
pub mod sessions;
pub mod stats;
