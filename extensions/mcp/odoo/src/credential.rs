//! Resolving the acting caller's Odoo credential.
//!
//! Two kinds of caller reach this server and each brings its own Odoo login:
//! a service account (the farm's) holds a sealed `odoo_api_key` machine
//! credential attached in the identity console, and a person holds the Odoo
//! account they linked under /admin/connectors. The platform crate opens either
//! in-process; this module turns it into a signed-in [`Credentials`]. There is
//! no fallback account and no shared key — a caller holding no credential is
//! refused with the remedy for their kind, because the alternative is
//! executing their request as somebody else.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_web_admin_platform::machine_credentials::{
    OdooCredentialLookup, OdooMachineCredential, caller, resolve_odoo_credential,
};
use systemprompt_web_admin_platform::service_accounts::ServiceAccounts;

use crate::client::{Credentials, OdooClient, OdooConnection};
use crate::error::OdooError;

pub const PERSON_UNLINKED_MESSAGE: &str = "Link your Odoo account in the Dynapps console \
     (/admin/connectors → Odoo account), then try again.";

pub const SERVICE_ACCOUNT_MISSING_MESSAGE: &str = "The service account has no Odoo credential. \
     Attach an `odoo_api_key` machine credential to it at /admin/identity/service-accounts.";

pub const PURPOSE_REFUSED_MESSAGE: &str = "A service account's Odoo key is only spent for a \
     declared purpose (such as `farm:<run_id>`); this call carried none.";

/// Where per-call credentials come from: the shared service-account registry.
#[derive(Debug, Clone)]
pub struct CredentialSource {
    pub accounts: ServiceAccounts,
}

#[derive(Debug)]
pub struct Lookup<'a> {
    pub pool: &'a PgPool,
    pub source: &'a CredentialSource,
    pub user_id: &'a UserId,
    pub caller_purpose: Option<&'a str>,
}

async fn lookup(l: &Lookup<'_>) -> Result<OdooCredentialLookup, OdooError> {
    resolve_odoo_credential(l.pool, &l.source.accounts, l.user_id)
        .await
        .map_err(|e| OdooError::Internal(e.to_string()))
}

pub fn stored_credential(
    lookup: OdooCredentialLookup,
    caller_purpose: Option<&str>,
) -> Result<OdooMachineCredential, OdooError> {
    match lookup {
        OdooCredentialLookup::ServiceAccount(stored) => {
            if caller::allowed(caller_purpose) {
                Ok(stored)
            } else {
                Err(OdooError::AccessDenied(PURPOSE_REFUSED_MESSAGE.to_owned()))
            }
        },
        OdooCredentialLookup::Person(stored) => Ok(stored),
        OdooCredentialLookup::ServiceAccountMissing => Err(OdooError::NotLinked(
            SERVICE_ACCOUNT_MISSING_MESSAGE.to_owned(),
        )),
        OdooCredentialLookup::PersonUnlinked => {
            Err(OdooError::NotLinked(PERSON_UNLINKED_MESSAGE.to_owned()))
        },
    }
}

pub async fn resolve_credentials(
    l: &Lookup<'_>,
    client: &OdooClient,
) -> Result<Credentials, OdooError> {
    let stored = stored_credential(lookup(l).await?, l.caller_purpose)?;
    let conn = OdooConnection::new(&stored.url, &stored.db)?;
    client.sign_in(conn, stored.login, stored.api_key).await
}
