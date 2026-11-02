//! `DynWork` relay, a systemprompt extension.
//!
//! The `DynWork` desktop and the `DynWork` phone app each hold one WebSocket to
//! this relay. Everything they say to each other is end-to-end encrypted with
//! Noise (`@dynwork/remote`), so the relay routes opaque envelopes by peer id
//! and never holds a key that opens them. It authenticates sockets with the
//! instance's own credentials: the `DynWork` JWT of a desktop, the device
//! credential of a phone. It pairs phones with one-time tokens a desktop
//! mints, lists and revokes devices, and holds a bounded ring of encrypted
//! envelopes per desktop so a peer that reconnects can pick up what it
//! missed.
//!
//! Routing is in process: one replica only (see the crate README).

pub mod audit;
pub mod auth;
pub mod clock;
pub mod config;
pub mod credential;
pub mod envelope;
pub mod error;
pub mod extension;
pub mod http;
pub mod hub;
pub mod ids;
pub mod mobile_auth;
pub mod router;
pub mod schema;
pub mod state;
pub mod store;
pub mod sweeper;
pub mod ws;

pub use config::RelayLimits;
pub use extension::RelayExtension;
pub use router::{
    BASE_PATH, EXTENSION_BASE_PATH, MOBILE_BASE_PATH, extension_router, relay_router,
};
pub use state::{RelayState, RelayStateBuilder};

systemprompt::extension::register_extension!(RelayExtension);
