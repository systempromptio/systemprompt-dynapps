//! Identity half of the ADFS callback: turn a verified assertion into gated
//! claims (allow-listed domain, mapped AD group) and resolve them to a local
//! user with the roles the groups map to.
//!
//! A refused sign-in (`forbidden`, `no_group`) also revokes the cookie
//! sessions and bridge links the directory once vouched for; revocation
//! failures are logged and never turn the `?sso=` redirect into a 500.

use saml::{Identity, NameIdFormat};

use super::config::AdfsConfig;
use super::{AdfsDeps, LoginRedirect};
use crate::repositories::users::{federated, revocation};
use crate::repositories::{groups, projects};

/// The subset of a verified assertion this login consumes: the subject and
/// the attribute statement, with the `IdP`'s claim URIs left as-is.
#[derive(Debug, Clone, Default)]
pub struct AssertionClaims {
    pub name_id: Option<String>,
    pub name_id_is_email: bool,
    pub attributes: Vec<(String, Vec<String>)>,
}

impl AssertionClaims {
    #[must_use]
    pub fn from_identity(identity: &Identity) -> Self {
        Self {
            name_id: Some(identity.name_id.value.clone()),
            name_id_is_email: identity.name_id.format == NameIdFormat::EmailAddress,
            attributes: identity
                .attributes
                .iter()
                .map(|a| (a.name.clone(), a.values.clone()))
                .collect(),
        }
    }

    #[must_use]
    pub fn values(&self, attribute: &str) -> Vec<String> {
        self.attributes
            .iter()
            .filter(|(name, _)| name == attribute)
            .flat_map(|(_, values)| values.iter().cloned())
            .collect()
    }

    fn first(&self, attribute: &str) -> Option<String> {
        self.values(attribute)
            .into_iter()
            .map(|v| v.trim().to_owned())
            .find(|v| !v.is_empty())
    }

    // Why: AD compares addresses case-insensitively and `users.email` is
    // stored lower-cased, so the asserted email is folded before any lookup.
    #[must_use]
    pub fn login_email(&self, cfg: &AdfsConfig) -> Option<String> {
        self.first(&cfg.email_attribute)
            .or_else(|| {
                self.name_id
                    .as_deref()
                    .filter(|_| self.name_id_is_email)
                    .map(ToOwned::to_owned)
            })
            .map(|e| e.trim().to_lowercase())
            .filter(|e| e.contains('@'))
    }

    #[must_use]
    pub fn display_name(&self, cfg: &AdfsConfig) -> Option<String> {
        self.first(&cfg.name_attribute)
    }

    // Why: AD FS relying parties are routinely configured with claims but no
    // NameID, so the login email stands in as the stable external subject.
    #[must_use]
    pub fn external_sub(&self, email: &str) -> String {
        self.name_id
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map_or_else(|| email.to_owned(), ToOwned::to_owned)
    }
}

pub(super) struct GatedIdentity {
    pub external_sub: String,
    pub email: String,
    pub display_name: String,
    pub groups: Vec<String>,
    pub roles: Vec<String>,
}

// Why: AD is the corporate directory, so an address it asserts is verified by
// construction; the domain allow-list and group map gate, not `email_verified`.
pub(super) fn gate_claims(
    cfg: &AdfsConfig,
    claims: &AssertionClaims,
) -> Result<GatedIdentity, LoginRedirect> {
    let email = claims.login_email(cfg).ok_or(LoginRedirect("no_email"))?;
    if !cfg.email_allowed(&email) {
        tracing::warn!(email, "ADFS login rejected: domain not allow-listed");
        return Err(LoginRedirect("forbidden"));
    }
    let groups = claims.values(&cfg.groups_attribute);
    let directory_vouched_for_nobody = groups.is_empty();
    if directory_vouched_for_nobody && cfg.deny_without_group {
        tracing::warn!(email, "ADFS login rejected: assertion carried no AD group");
        return Err(LoginRedirect("no_group"));
    }
    let mut roles = cfg.roles_for_groups(&groups);
    if roles.is_empty() {
        roles.push("user".to_owned());
    }
    let display_name = claims.display_name(cfg).unwrap_or_else(|| email.clone());
    Ok(GatedIdentity {
        external_sub: claims.external_sub(&email),
        email,
        display_name,
        groups,
        roles,
    })
}

pub(super) async fn resolve_identity(
    deps: &AdfsDeps,
    idp_entity_id: &str,
    claims: &AssertionClaims,
) -> Result<federated::ResolvedFederatedUser, LoginRedirect> {
    let gated = match gate_claims(&deps.config, claims) {
        Ok(gated) => gated,
        Err(reason) => {
            revoke_on_denial(deps, &deps.config, claims, reason.0).await;
            return Err(reason);
        },
    };
    let resolved = find_or_provision_user(deps, idp_entity_id, &gated).await?;
    persist_directory_memberships(deps, &resolved, &gated.groups).await;
    revoke_if_demoted(deps, &resolved).await;
    Ok(resolved)
}

async fn find_or_provision_user(
    deps: &AdfsDeps,
    idp_entity_id: &str,
    gated: &GatedIdentity,
) -> Result<federated::ResolvedFederatedUser, LoginRedirect> {
    let federated_claims = federated::FederatedClaims {
        issuer: idp_entity_id,
        external_sub: &gated.external_sub,
        email: &gated.email,
        display_name: &gated.display_name,
        roles: &gated.roles,
    };
    federated::resolve_federated_user(
        &deps.write_pool,
        &federated_claims,
        deps.config.auto_provision,
    )
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "Failed to resolve federated ADFS user");
        LoginRedirect("error")
    })?
    .ok_or_else(|| {
        tracing::warn!(
            email = %gated.email,
            "ADFS login rejected: auto-provisioning disabled and no existing account"
        );
        LoginRedirect("not_provisioned")
    })
}

async fn persist_directory_memberships(
    deps: &AdfsDeps,
    resolved: &federated::ResolvedFederatedUser,
    groups: &[String],
) {
    let user_id = &resolved.user_id;
    if let Err(e) =
        groups::members::replace_directory_group_memberships(&deps.write_pool, user_id, groups)
            .await
    {
        tracing::warn!(error = %e, %user_id, "Failed to persist ADFS group memberships");
    }
    if let Err(e) =
        projects::members::replace_directory_project_memberships(&deps.write_pool, user_id, groups)
            .await
    {
        tracing::warn!(error = %e, %user_id, "Failed to persist ADFS project memberships");
    }
}

async fn revoke_if_demoted(deps: &AdfsDeps, resolved: &federated::ResolvedFederatedUser) {
    if !resolved.lost_admin {
        return;
    }
    match revocation::revoke_user_access(&deps.write_pool, &resolved.user_id).await {
        Ok(counts) => tracing::warn!(
            user_id = %resolved.user_id,
            sessions = counts.sessions,
            api_keys = counts.api_keys,
            "Revoked credentials after ADFS removed the admin role"
        ),
        Err(e) => tracing::error!(
            error = %e,
            user_id = %resolved.user_id,
            "Failed to revoke credentials after an ADFS admin demotion"
        ),
    }
}

async fn revoke_on_denial(
    deps: &AdfsDeps,
    cfg: &AdfsConfig,
    claims: &AssertionClaims,
    reason: &str,
) {
    if !matches!(reason, "no_group" | "forbidden") {
        return;
    }
    let Some(email) = claims.login_email(cfg) else {
        return;
    };
    match revocation::revoke_access_by_email(&deps.write_pool, &email).await {
        Ok(Some((user_id, counts))) if !counts.is_empty() => tracing::warn!(
            %user_id,
            reason,
            sessions = counts.sessions,
            api_keys = counts.api_keys,
            device_certs = counts.device_certs,
            exchange_codes = counts.exchange_codes,
            "Revoked credentials after ADFS refused the sign-in"
        ),
        Ok(_) => {},
        Err(e) => {
            tracing::error!(error = %e, reason, "Failed to revoke credentials after a refused ADFS sign-in");
        },
    }
}
