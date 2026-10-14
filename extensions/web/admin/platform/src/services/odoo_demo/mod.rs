//! The console's "Run Odoo demo" seeder for the demo-* Odoo records.
//!
//! It restores the records the three Odoo farm agents work on, over the farm
//! service account's own `odoo_api_key` machine credential — the same bot
//! identity the agents act as, so the seed never needs a broader Odoo login.
//!
//! `plan` holds the records as data; `seed` upserts them.

pub mod plan;
mod rpc;
mod seed;

use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};

pub use seed::SeedSummary;

use crate::services::machine_credentials::odoo_credential_for;
use crate::services::service_accounts::ServiceAccounts;

pub async fn reseed_odoo_demo(
    pool: &PgPool,
    accounts: &ServiceAccounts,
    service_account: &UserId,
) -> AdminResult<SeedSummary> {
    let credential = odoo_credential_for(pool, accounts, service_account)
        .await?
        .ok_or_else(|| {
            AdminError::Unprocessable(format!(
                "service account {service_account} holds no odoo_api_key credential"
            ))
        })?;
    let odoo = rpc::OdooSession::sign_in(&credential).await?;
    seed::reseed(&odoo).await
}
