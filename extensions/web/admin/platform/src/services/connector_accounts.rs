//! The connection snapshot source: browser accounts and every enrolled bridge,
//! described against the caller's entitlement.

use super::connector_oauth::Provider;
use crate::repositories::connectors::connector_accounts as repo;
use chrono::Utc;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt::models::auth::Permission;
use systemprompt_web_admin_core::connections::source::{
    ConnectionSnapshotSource, ConnectionSourceRegistration,
};
use systemprompt_web_admin_core::connections::{Connection, ConnectionSnapshot};
use systemprompt_web_admin_core::error::{AdminError, AdminResult};

pub(crate) use super::connector_readiness::probe_session_connection;

pub(super) struct Entitlement {
    pub(super) active: bool,
    pub(super) roles: Vec<String>,
}

pub(super) fn scopes_granted(roles: &[String], scopes: &[Permission]) -> bool {
    scopes.iter().all(|scope| match scope {
        Permission::Admin => systemprompt_web_admin_core::types::roles_grant_manage(roles),
        Permission::User | Permission::Anonymous => true,
        _ => false,
    })
}

pub(super) async fn load_entitlement(pool: &PgPool, user: &UserId) -> AdminResult<Entitlement> {
    let identity =
        systemprompt_web_admin_core::repositories::principal::identity::find_identity_envelope(
            pool, user,
        )
        .await?
        .ok_or_else(|| AdminError::Unauthorized("Account unavailable".into()))?;
    Ok(Entitlement {
        active: identity.status == "active",
        roles: identity.roles,
    })
}

impl Entitlement {
    fn permits(
        &self,
        services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
        provider: &Provider,
    ) -> bool {
        self.active
            && provider
                .session_scopes(&services.config())
                .is_none_or(|scopes| scopes_granted(&self.roles, scopes))
    }
}

fn not_entitled_message(
    services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    provider: &Provider,
) -> &'static str {
    if provider.is_session_attested(&services.config()) {
        "This connector is not open to your account"
    } else {
        "An active account is required to connect providers"
    }
}

pub(crate) async fn require_entitlement(
    services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    pool: &PgPool,
    user: &UserId,
    provider: Provider,
) -> AdminResult<()> {
    let entitlement = load_entitlement(pool, user).await?;
    if !entitlement.active {
        return Err(AdminError::Forbidden(
            "An active account is required to connect providers".into(),
        ));
    }
    if !entitlement.permits(services, &provider) {
        return Err(AdminError::Forbidden(
            not_entitled_message(services, &provider).into(),
        ));
    }
    Ok(())
}

struct ActionInputs<'a> {
    row: Option<&'a repo::ProviderConnection>,
    status: &'a str,
    configured: bool,
    entitled: bool,
    session_attested: bool,
    requires_auth: bool,
}

fn actions_for(inputs: &ActionInputs<'_>) -> Vec<String> {
    let ActionInputs {
        row,
        status,
        configured,
        entitled,
        session_attested,
        requires_auth,
    } = *inputs;
    let mut actions = Vec::new();
    if configured && entitled && session_attested {
        actions.push("test".into());
    } else if configured && entitled && requires_auth {
        actions.push(
            if status == "connected" {
                "reconnect"
            } else {
                "connect"
            }
            .into(),
        );
        if row.is_some_and(|r| r.auth_method.is_some()) {
            actions.push("test".into());
        }
    }
    if row.is_some_and(|r| r.auth_method.is_some()) {
        actions.push("disconnect".into());
    }
    actions
}

fn describe(
    services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    provider: &Provider,
    row: Option<&repo::ProviderConnection>,
    entitlement: &Entitlement,
) -> Connection {
    let configured = provider.configured(&services.config());
    let entitled = entitlement.permits(services, provider);
    let session_attested = provider.is_session_attested(&services.config());
    let attested_by_this_request = session_attested && entitled;
    let status = if configured && !provider.requires_auth(&services.config()) {
        "no_auth_required"
    } else if configured && session_attested {
        if entitled {
            "connected"
        } else {
            "not_connected"
        }
    } else if configured {
        row.map_or("not_connected", |r| r.status.as_str())
    } else {
        "not_configured"
    };
    Connection {
        provider: provider.slug().into(),
        display_name: provider.display_name(&services.config()),
        requires_auth: provider.requires_auth(&services.config()),
        session_attested,
        configured,
        entitled,
        status: status.into(),
        auth_method: if session_attested {
            Some("session".into())
        } else {
            row.and_then(|r| r.auth_method.clone())
        },
        account_id: row.and_then(|r| r.account_id.clone()),
        account_name: row.and_then(|r| r.account_name.clone()),
        resource_id: row.and_then(|r| r.resource_id.clone()),
        resource_name: row.and_then(|r| r.resource_name.clone()),
        error_code: row.and_then(|r| r.error_code.clone()),
        verified_at: if attested_by_this_request {
            Some(Utc::now().to_rfc3339())
        } else {
            row.and_then(|r| r.verified_at.map(|t| t.to_rfc3339()))
        },
        actions: actions_for(&ActionInputs {
            row,
            status,
            configured,
            entitled,
            session_attested,
            requires_auth: provider.requires_auth(&services.config()),
        }),
    }
}

struct AccountConnectionSource;

#[async_trait::async_trait]
impl ConnectionSnapshotSource for AccountConnectionSource {
    async fn snapshot(
        &self,
        pool: &PgPool,
        services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
        user: &UserId,
    ) -> AdminResult<ConnectionSnapshot> {
        build_snapshot(pool, services, user).await
    }

    fn display_name(
        &self,
        services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
        provider: &str,
    ) -> String {
        Provider::try_from(provider.to_owned()).map_or_else(
            |_| provider.to_owned(),
            |p| p.display_name(&services.config()),
        )
    }
}

inventory::submit! { ConnectionSourceRegistration(&AccountConnectionSource) }

async fn build_snapshot(
    pool: &PgPool,
    services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    user: &UserId,
) -> AdminResult<ConnectionSnapshot> {
    let rows = repo::list_accounts(pool, user).await?;
    let entitlement = load_entitlement(pool, user).await?;
    let mut ids = services
        .config()
        .mcp_servers
        .iter()
        .filter(|(_, s)| s.enabled)
        .map(|(id, _)| id.clone())
        .collect::<std::collections::BTreeSet<_>>();
    ids.extend(
        rows.iter()
            .filter(|r| r.auth_method.is_some())
            .map(|r| r.provider.clone()),
    );
    let mut connections = Vec::new();
    for id in ids {
        let provider = Provider::try_from(id).map_err(AdminError::BadRequest)?;
        let row = rows.iter().find(|r| r.provider == provider.slug());
        connections.push(describe(services, &provider, row, &entitlement));
    }
    Ok(ConnectionSnapshot {
        version: 1,
        user_id: user.clone(),
        revision: rows.iter().map(|r| r.revision).max().unwrap_or(0),
        connections,
        verification: None,
    })
}
