//! The declaration: the `gateway.routes:` sequence of
//! `services/ai/gateway.yaml`, in file order.
//!
//! Read from the baked services tree — the file the editor writes and core
//! boots from — or, when a pinned services bundle owns `ai/`, from the
//! composed root the dispatcher reads (and then never written); hashed on the
//! parsed routes so a comment or whitespace edit is not a drift. Every field
//! core's `GatewayRoute` carries is part of the hash.

use serde_yaml::Value;
use sha2::{Digest, Sha256};
use systemprompt_web_admin_core::extension_boot::ExtensionBoot;

use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::repositories::gateway_catalog::{route_from_yaml, route_to_yaml};
use systemprompt_web_admin_core::types::GatewayRouteView;

pub const GATEWAY_FILE: &str = "ai/gateway.yaml";

#[derive(Debug, Clone)]
pub struct DeclaredRoute {
    pub route: GatewayRouteView,
}

#[derive(Debug, Clone, Default)]
pub struct DeclaredRoutes {
    pub routes: Vec<DeclaredRoute>,
}

#[must_use]
pub fn route_fingerprint(route: &GatewayRouteView) -> String {
    route_to_yaml(route)
        .ok()
        .and_then(|value| serde_yaml::to_string(&value).ok())
        .unwrap_or_else(|| format!("unrenderable route {}", route.id))
}

impl DeclaredRoutes {
    #[must_use]
    pub fn declared_hash(&self) -> String {
        let mut hasher = Sha256::new();
        for (i, d) in self.routes.iter().enumerate() {
            hasher.update(format!("route\t{i}\t{}\n", route_fingerprint(&d.route)).as_bytes());
        }
        format!("{:x}", hasher.finalize())
    }

    #[must_use]
    pub fn find(&self, id: &str) -> Option<&DeclaredRoute> {
        self.routes.iter().find(|d| d.route.id == id)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeclaredRoutesError {
    #[error("gateway.yaml does not parse: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("gateway.routes[{index}] is not a route core loads")]
    IncompleteRoute { index: usize },
    #[error("gateway.routes[{index}] repeats id `{id}`")]
    DuplicateRoute { index: usize, id: String },
}

pub fn parse_declared_routes(yaml: &str) -> Result<DeclaredRoutes, DeclaredRoutesError> {
    let doc: Value = serde_yaml::from_str(yaml)?;
    let Some(seq) = doc
        .get("gateway")
        .and_then(|g| g.get("routes"))
        .and_then(Value::as_sequence)
    else {
        return Ok(DeclaredRoutes::default());
    };
    let mut routes = Vec::with_capacity(seq.len());
    for (i, entry) in seq.iter().enumerate() {
        let Some(route) = route_from_yaml(entry) else {
            return Err(DeclaredRoutesError::IncompleteRoute { index: i });
        };
        if routes
            .iter()
            .any(|d: &DeclaredRoute| d.route.id == route.id)
        {
            return Err(DeclaredRoutesError::DuplicateRoute {
                index: i,
                id: route.id,
            });
        }
        routes.push(DeclaredRoute { route });
    }
    Ok(DeclaredRoutes { routes })
}

// Why: not core's composed services root — that is a content-addressed overlay
// the next refresh replaces, and this is the one file the console writes back.
// When a pinned bundle owns `ai/` the composed copy is what the dispatcher
// reads, so the plane reads that one and the console does not write at all.
#[must_use]
pub fn gateway_file_path(boot: &ExtensionBoot) -> std::path::PathBuf {
    systemprompt_web_admin_core::request::get_gateway_file_path(boot)
}

#[must_use]
pub fn bundle_owner(boot: &ExtensionBoot) -> Option<String> {
    systemprompt_web_admin_core::repositories::sync::sources::dir_owner(boot.profile(), "ai")
}

pub fn declared_routes_now(boot: &ExtensionBoot) -> AdminResult<DeclaredRoutes> {
    let file = gateway_file_path(boot);
    let yaml = std::fs::read_to_string(&file)
        .map_err(|e| AdminError::invalid("gateway.yaml could not be read", e))?;
    parse_declared_routes(&yaml)
        .map_err(|e| AdminError::invalid("gateway.yaml could not be parsed", e))
}
