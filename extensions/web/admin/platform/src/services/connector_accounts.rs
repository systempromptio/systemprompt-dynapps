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
    fn permits(&self, provider: &Provider) -> bool {
        self.active
            && provider
                .session_scopes()
                .is_none_or(|scopes| scopes_granted(&self.roles, scopes))
    }
}

fn not_entitled_message(provider: &Provider) -> &'static str {
    if provider.is_session_attested() {
        "This connector is not open to your account"
    } else {
        "An active account is required to connect providers"
    }
}

pub(crate) async fn require_entitlement(
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
    if !entitlement.permits(&provider) {
        return Err(AdminError::Forbidden(
            not_entitled_message(&provider).into(),
        ));
    }
    Ok(())
}

fn actions_for(
    provider: &Provider,
    row: Option<&repo::ProviderConnection>,
    status: &str,
    configured: bool,
    entitled: bool,
) -> Vec<String> {
    let mut actions = Vec::new();
    if configured && entitled && provider.is_session_attested() {
        actions.push("test".into());
    } else if configured && entitled && provider.requires_auth() {
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
    provider: &Provider,
    row: Option<&repo::ProviderConnection>,
    entitlement: &Entitlement,
) -> Connection {
    let configured = provider.configured();
    let entitled = entitlement.permits(provider);
    let session_attested = provider.is_session_attested();
    let attested_by_this_request = session_attested && entitled;
    let status = if configured && !provider.requires_auth() {
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
        display_name: provider.display_name(),
        requires_auth: provider.requires_auth(),
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
        actions: actions_for(provider, row, status, configured, entitled),
    }
}

struct AccountConnectionSource;

#[async_trait::async_trait]
impl ConnectionSnapshotSource for AccountConnectionSource {
    async fn snapshot(&self, pool: &PgPool, user: &UserId) -> AdminResult<ConnectionSnapshot> {
        build_snapshot(pool, user).await
    }

    fn display_name(&self, provider: &str) -> String {
        Provider::try_from(provider.to_owned())
            .map_or_else(|_| provider.to_owned(), |p| p.display_name())
    }
}

inventory::submit! { ConnectionSourceRegistration(&AccountConnectionSource) }

async fn build_snapshot(pool: &PgPool, user: &UserId) -> AdminResult<ConnectionSnapshot> {
    let rows = repo::list_accounts(pool, user).await?;
    let entitlement = load_entitlement(pool, user).await?;
    let services = systemprompt::loader::ServicesBootstrap::get().map_err(AdminError::internal)?;
    let mut ids = services
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
        connections.push(describe(&provider, row, &entitlement));
    }
    Ok(ConnectionSnapshot {
        version: 1,
        user_id: user.clone(),
        revision: rows.iter().map(|r| r.revision).max().unwrap_or(0),
        connections,
        verification: None,
    })
}
