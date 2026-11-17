//! The tenant selector every Farm page carries.

use serde::Serialize;
use crate::client::is_all_tenants;
use crate::models::TenantRef;

#[derive(Debug, Serialize)]
pub struct TenantOption {
    pub slug: String,
    pub label: String,
    pub selected: bool,
}

/// The tenant `<select>` every Farm page carries: "All tenants" (the default,
/// sent as an empty value) and one option per tenant.
#[derive(Debug, Serialize)]
pub struct TenantSelect {
    pub action: &'static str,
    pub tenant: String,
    pub all_selected: bool,
    pub tenants: Vec<TenantOption>,
    pub has_tenants: bool,
}

/// `selected` is the filter: a tenant slug, or the all-tenants value.
pub fn tenant_select(action: &'static str, tenants: &[TenantRef], selected: &str) -> TenantSelect {
    TenantSelect {
        action,
        tenant: selected.to_owned(),
        all_selected: is_all_tenants(selected),
        tenants: tenant_options(tenants, selected),
        has_tenants: !tenants.is_empty(),
    }
}

pub fn tenant_options(tenants: &[TenantRef], selected: &str) -> Vec<TenantOption> {
    tenants
        .iter()
        .map(|t| TenantOption {
            label: t
                .name
                .as_deref()
                .filter(|n| !n.is_empty() && *n != t.tenant)
                .map_or_else(|| t.tenant.clone(), |name| format!("{name} ({})", t.tenant)),
            selected: t.tenant == selected,
            slug: t.tenant.clone(),
        })
        .collect()
}
