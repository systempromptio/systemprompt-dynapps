//! Salesforce org binding at consent, and ECA pre-authorization using the
//! user's administratively mapped identity for one org.

use super::config::validate_my_domain;
use super::{Grant, Provider};
use crate::handlers::salesforce_auth::SalesforceConfig;
use crate::repositories::connectors::salesforce_identity;
use crate::services::salesforce_jwt_bearer;
use chrono::Utc;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
// JSON: Salesforce's OAuth userinfo is an open claim set; only
// `organization_id` and `urls.custom_domain` are read, at consent time.
use serde_json::Value;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

pub(super) async fn mint(
    pool: &PgPool,
    user: &UserId,
    provider: &Provider,
    generation: i64,
) -> AdminResult<Grant> {
    let org = provider.salesforce_org()?;
    let username = salesforce_identity::find_username(pool, user, provider.slug())
        .await?
        .ok_or_else(|| {
            AdminError::NotFound(
                "Salesforce identity is not provisioned; connect in the browser".into(),
            )
        })?;
    let cfg = SalesforceConfig {
        enabled: true,
        my_domain: org.domain()?,
        consumer_key: org.client_id()?,
    };
    let private_key = org.private_key()?;
    let token = salesforce_jwt_bearer::get_token(&cfg, &username, &private_key)
        .await
        .map_err(|_redacted_error| {
            AdminError::Upstream(
                "Salesforce pre-authorization failed; check External Client App settings".into(),
            )
        })?;
    Ok(Grant {
        configuration_binding: String::new(),
        authorization_issuer: String::new(),
        token_auth_method: String::new(),
        user: user.to_string(),
        provider: provider.clone(),
        client: cfg.consumer_key,
        client_secret: String::new(),
        verifier: String::new(),
        access_token: token.access_token,
        refresh_token: None,
        expires_at: Utc::now().timestamp() + 60,
        token_endpoint: String::new(),
        generation,
        session: None,
        auth_method: "jwt_bearer".into(),
        account_id: String::new(),
        account_name: username,
        resource_id: String::new(),
        resource_name: cfg.my_domain,
        authorization_scheme: "Bearer".into(),
    })
}

pub(super) fn bind_org(grant: &mut Grant, info: &Value) -> AdminResult<()> {
    let org = grant.provider.salesforce_org()?;
    let organization_id = info
        .get("organization_id")
        .and_then(Value::as_str)
        .ok_or_else(|| AdminError::Upstream("Salesforce returned no organization ID".into()))?;
    let signed_into_wrong_org = org
        .org_id
        .as_deref()
        .is_some_and(|expected| expected != organization_id);
    if signed_into_wrong_org {
        return Err(AdminError::Unauthorized(
            "Signed in to a different Salesforce org than this connector expects".into(),
        ));
    }
    grant.resource_id = organization_id.into();
    grant.resource_name = info
        .pointer("/urls/custom_domain")
        .and_then(Value::as_str)
        .and_then(|url| validate_my_domain(url).ok())
        .map_or_else(|| org.domain(), Ok)?;
    Ok(())
}
