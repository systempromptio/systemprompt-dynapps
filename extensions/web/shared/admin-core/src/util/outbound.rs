//! Outbound-URL admission for destinations this installation reaches on its
//! own private network.
//!
//! Core's `validate_outbound_url` refuses plain `http` to anything but a
//! loopback host and refuses every private address range. The farm API that a
//! runner machine reports to and an in-cluster OTLP collector both sit on
//! Fly's private network, which has no TLS and is addressed by `.internal`
//! names, so [`validate_mesh_url`] trusts exactly those names for `http` and
//! applies core's rule unchanged to everything else.

use systemprompt::models::net::{OutboundUrlError, validate_outbound_url_with_trust};
use url::Url;

// Why: Fly's 6PN private network resolves only `*.internal` names and carries
// no TLS, so an `http` destination there is the platform's shape, not a
// downgrade.
const FLY_PRIVATE_SUFFIX: &str = ".internal";

pub fn validate_mesh_url(url: &str) -> Result<Url, OutboundUrlError> {
    let host = Url::parse(url)?
        .host_str()
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    let trusted: Vec<String> = if host.ends_with(FLY_PRIVATE_SUFFIX) {
        vec![host]
    } else {
        Vec::new()
    };
    validate_outbound_url_with_trust(url, &trusted)
}
