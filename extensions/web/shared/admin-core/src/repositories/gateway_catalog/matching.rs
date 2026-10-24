//! Route-id synthesis and glob matching.
//!
//! Declared routes carry their own ids; the one synthesized id left is the
//! catalog route's (`slug + fnv1a6`), which core mints. A second
//! implementation here would only ever agree with core's by accident, so the
//! functions below delegate and exist only to keep the `String`-returning
//! signature this crate's callers expect. [`glob_match`] implements the same
//! `*` semantics a route's `models` globs use at request time.

use crate::types::GatewayRouteView;

#[must_use]
pub fn slugify_pattern(pattern: &str) -> String {
    systemprompt::manifest::services::slugify_pattern(pattern)
}

#[must_use]
pub fn synthesize_route_id(glob: &str, provider: &str) -> String {
    systemprompt::manifest::services::synthesize_route_id(glob, provider)
        .as_str()
        .to_owned()
}

#[must_use]
pub fn find_matching_route_index(routes: &[GatewayRouteView], model: &str) -> Option<usize> {
    routes
        .iter()
        .position(|r| route_matches(r, model))
}

#[must_use]
pub fn find_matching_route<'a>(
    routes: &'a [GatewayRouteView],
    model: &str,
) -> Option<&'a GatewayRouteView> {
    routes.iter().find(|r| route_matches(r, model))
}

fn route_matches(route: &GatewayRouteView, model: &str) -> bool {
    route
        .alias
        .as_ref()
        .is_some_and(|alias| alias.requested == model)
        || route.models.iter().any(|glob| glob_match(glob, model))
}

#[must_use]
pub fn find_route_index_by_id(routes: &[GatewayRouteView], id: &str) -> Option<usize> {
    routes.iter().position(|r| r.id == id)
}

pub fn glob_match(pattern: &str, value: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if !pattern.contains('*') {
        return pattern == value;
    }
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 2 {
        let (prefix, suffix) = (parts[0], parts[1]);
        return value.starts_with(prefix)
            && value.ends_with(suffix)
            && value.len() >= prefix.len() + suffix.len();
    }
    let mut cursor = 0usize;
    for (i, segment) in parts.iter().enumerate() {
        if segment.is_empty() {
            continue;
        }
        let Some(found) = value[cursor..].find(segment) else {
            return false;
        };
        if i == 0 && found != 0 {
            return false;
        }
        cursor += found + segment.len();
    }
    if let Some(last) = parts.last()
        && !last.is_empty()
        && !value.ends_with(last)
    {
        return false;
    }
    true
}
