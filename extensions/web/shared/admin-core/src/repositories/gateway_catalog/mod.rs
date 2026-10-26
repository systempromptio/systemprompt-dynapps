//! Gateway route configuration backed by `services/ai/gateway.yaml`, and the
//! route and provider views every console page reads from the resolved AI
//! catalog.
//!
//! The gateway config is not a Postgres table: it lives in the services
//! tree's `gateway` block, which is why it sits here. These functions read,
//! mutate, and re-serialize that block. Reads never write, and a write touches
//! the edited field only: the file is an operator's, comments and all.

mod catalog;
mod config;
mod labels;
mod matching;
mod routes;
mod yaml_io;

pub use catalog::{
    client_facing_routes, dispatchable_route_ids, dispatchable_routes, registered_routes,
    registered_routes_from_services, resolved_catalog, retain_client_facing, route_reaches,
};
pub use config::{get_gateway_config, update_gateway_settings};
pub use labels::{
    ProviderLabel, RouteLabel, RouteLabels, derive_provider_label, derive_route_label,
    get_route_labels,
};
pub use matching::{
    find_matching_route, find_matching_route_index, find_route_index_by_id, glob_match,
    slugify_pattern, synthesize_route_id,
};
pub use routes::{
    create_route, delete_route, normalise_metadata, reorder_routes, replace_routes, update_route,
    validate_route,
};
pub use yaml_io::{read_gateway_file, route_from_yaml, route_to_yaml, route_view};
