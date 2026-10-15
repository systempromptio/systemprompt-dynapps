//! Fake identity providers for the SSO suites.
//!
//! An Entra-shaped OIDC provider on loopback HTTP ([`oidc_idp`]), keyed by
//! [`keys::TestIdpKeys`].

pub mod keys;
pub mod oidc_idp;

pub use keys::TestIdpKeys;
pub use oidc_idp::{FakeOidcIdp, IdTokenSpec, pkce_challenge};
