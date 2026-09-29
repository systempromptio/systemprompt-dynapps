//! Server-rendered governance and gateway pages.
//!
//! Each module owns one page: it builds a typed template context and renders a
//! `.hbs` template from `storage/files/admin/templates/` at request time.

mod approvals;
mod gateway;
mod gateway_policies;
pub(crate) mod governance;
mod governance_quotas;
pub(crate) mod secrets_audit;
mod ssr_chain;
mod ssr_governance_audit_detail;
mod ssr_search_resolve;

pub(crate) use approvals::approvals_page;
pub(crate) use gateway::{gateway_page, gateway_route_page};
pub(crate) use gateway_policies::{
    delete_gateway_policy, gateway_policies_page, save_gateway_policy,
};
pub(crate) use governance::governance_page;
pub(crate) use governance_quotas::governance_quotas_page;
pub(crate) use secrets_audit::secrets_audit_page;
pub(crate) use ssr_chain::chain_envelope;
pub(crate) use ssr_governance_audit_detail::governance_audit_detail_page;
pub(crate) use ssr_search_resolve::search_resolve;
