//! Where an instance's Microsoft 365 connector signs people in: the Entra
//! authority, the tenant path segment and the tenants it admits.
//!
//! All three are read from the secrets store on every use, so a change applies
//! without a restart and never lands in `services/`. The tenant segment is
//! `organizations` for the shared multi-tenant app or one tenant id for a
//! customer's own app. Personal Microsoft accounts are never admitted:
//! `common`, `consumers` and the consumer tenant's own id ([`PERSONAL_TENANT`],
//! shaped like any tenant id) are refused as the tenant, in the allow-list and
//! as the `tid` of an ID token, all through [`admits_personal_accounts`]. In
//! shared mode the allow-list is mandatory; in dedicated mode it defaults to
//! that one tenant. Every value is validated before it reaches a URL, so a
//! secret cannot add a path, a query or a host.

use systemprompt::models::net::{LoopbackPolicy, validate_outbound_url};
use systemprompt::traits::SecretStore;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use url::Url;

pub const TENANT_SECRET: &str = "microsoft_365_tenant";
pub const ALLOWED_TENANTS_SECRET: &str = "microsoft_365_allowed_tenants";
pub const AUTHORITY_SECRET: &str = "microsoft_365_authority";
pub const PUBLIC_AUTHORITY: &str = "https://login.microsoftonline.com";
pub const SHARED_TENANT: &str = "organizations";

// Why: Entra signs personal Microsoft accounts into this tenant; shaped like
// any tenant id, as a tenant segment it behaves like `consumers`.
pub const PERSONAL_TENANT: &str = "9188040d-6c67-4c5b-b112-36a304b66dad";

const PERSONAL_SEGMENTS: [&str; 3] = ["common", "consumers", PERSONAL_TENANT];

/// The Entra authority, tenant segment and admitted tenants of one instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntraTenancy {
    authority: Url,
    tenant: String,
    allowed: Vec<String>,
}

impl EntraTenancy {
    pub fn load(secrets: &dyn SecretStore) -> AdminResult<Self> {
        let authority = optional(secrets, AUTHORITY_SECRET).map_or_else(
            || Url::parse(PUBLIC_AUTHORITY).map_err(AdminError::internal),
            |value| authority(&value),
        )?;
        let tenant = optional(secrets, TENANT_SECRET).map_or_else(
            || Ok(SHARED_TENANT.to_owned()),
            |value| tenant_segment(&value),
        )?;
        let allowed = match optional(secrets, ALLOWED_TENANTS_SECRET) {
            Some(list) => allow_list(&list)?,
            None if tenant == SHARED_TENANT => Vec::new(),
            None => vec![tenant.clone()],
        };
        let dedicated_outside_list = tenant != SHARED_TENANT && !allowed.contains(&tenant);
        if allowed.is_empty() || dedicated_outside_list {
            return Err(AdminError::Unavailable(format!(
                "Connector configuration missing: {ALLOWED_TENANTS_SECRET} must list the \
                 admitted tenant ids, including {TENANT_SECRET} when it names one"
            )));
        }
        Ok(Self {
            authority,
            tenant,
            allowed,
        })
    }

    pub fn tenant(&self) -> &str {
        &self.tenant
    }

    pub fn authorize_url(&self) -> Url {
        self.endpoint("authorize")
    }

    pub fn token_url(&self) -> Url {
        self.endpoint("token")
    }

    pub fn admits(&self, tenant: &str) -> bool {
        self.allowed.iter().any(|allowed| allowed == tenant)
    }

    fn endpoint(&self, leaf: &str) -> Url {
        let mut url = self.authority.clone();
        url.set_path(&format!("/{}/oauth2/v2.0/{leaf}", self.tenant));
        url
    }
}

pub fn tenant_id(value: &str) -> Option<String> {
    guid(value).filter(|id| !admits_personal_accounts(id))
}

pub fn admits_personal_accounts(value: &str) -> bool {
    let value = value.trim();
    PERSONAL_SEGMENTS
        .iter()
        .any(|segment| value.eq_ignore_ascii_case(segment))
}

pub(super) fn guid(value: &str) -> Option<String> {
    let id = value.trim().to_ascii_lowercase();
    let shaped = id.split('-').map(str::len).eq([8, 4, 4, 4, 12])
        && id
            .bytes()
            .all(|byte| byte == b'-' || byte.is_ascii_hexdigit());
    shaped.then_some(id)
}

pub fn is_tenant_segment(segment: &str) -> bool {
    segment == SHARED_TENANT || tenant_id(segment).as_deref() == Some(segment)
}

fn optional(secrets: &dyn SecretStore, name: &str) -> Option<String> {
    secrets
        .secret(name)
        .map(|secret| secret.expose().trim().to_owned())
        .filter(|value| !value.is_empty() && !value.starts_with("REPLACE_WITH"))
}

fn tenant_segment(value: &str) -> AdminResult<String> {
    if value.eq_ignore_ascii_case(SHARED_TENANT) {
        return Ok(SHARED_TENANT.to_owned());
    }
    if admits_personal_accounts(value) {
        return Err(personal_accounts_refused(TENANT_SECRET));
    }
    tenant_id(value).ok_or_else(|| {
        AdminError::Unavailable(format!(
            "{TENANT_SECRET} must be `{SHARED_TENANT}` or a tenant id"
        ))
    })
}

fn allow_list(value: &str) -> AdminResult<Vec<String>> {
    value
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|entry| !entry.is_empty())
        .map(|entry| {
            if admits_personal_accounts(entry) {
                return Err(personal_accounts_refused(ALLOWED_TENANTS_SECRET));
            }
            tenant_id(entry).ok_or_else(|| {
                AdminError::Unavailable(format!(
                    "{ALLOWED_TENANTS_SECRET} holds a value that is not a tenant id"
                ))
            })
        })
        .collect()
}

fn personal_accounts_refused(secret: &str) -> AdminError {
    AdminError::Unavailable(format!(
        "{secret} names a tenant that admits personal Microsoft accounts, which are not \
         supported; use an organization's tenant id"
    ))
}

fn authority(value: &str) -> AdminResult<Url> {
    let url = validate_outbound_url(value, LoopbackPolicy::Allow)?;
    let origin_only = url.scheme() == "https"
        && url.username().is_empty()
        && url.password().is_none()
        && url.path() == "/"
        && url.query().is_none()
        && url.fragment().is_none();
    if !origin_only {
        return Err(AdminError::Unavailable(format!(
            "{AUTHORITY_SECRET} must be an https origin with no path"
        )));
    }
    Ok(url)
}
