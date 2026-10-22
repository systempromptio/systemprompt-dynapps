//! Shared test setup.

// The bridge's reqwest is built with `rustls-no-provider`; the binary installs
// ring at startup, so a test that builds a client must do the same. An error
// means another test already installed it.
pub(crate) fn install_tls() {
    if rustls::crypto::ring::default_provider()
        .install_default()
        .is_err()
    {
        // already installed by an earlier test in this process
    }
}
