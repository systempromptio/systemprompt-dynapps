//! Persistence for the bridge control plane: API keys, device certificates,
//! exchange codes, and the presence rule applied to a bridge heartbeat.

pub mod api_keys;
pub mod bridge_users;
pub mod device_certs;
pub mod device_keys;
pub mod error;
pub mod exchange_codes;
pub mod presence;

pub use api_keys::{
    API_KEY_PREFIX, ApiKeyRow, ApiKeyUser, EnrollDeviceParams, EnrolledDevice, IssuedApiKey,
    enroll_device, find_api_key_user, issue_api_key, list_api_keys_for_user, revoke_api_key,
};
pub use bridge_users::{BridgeIdentityRow, find_bridge_user};
pub use device_certs::{DeviceCertRow, revoke_device_cert};
pub use device_keys::revoke_device_key;
pub use error::{BridgeRepoError, Result};
pub use exchange_codes::{
    EXCHANGE_CODE_TTL_SECONDS, IssuedExchangeCode, UiSessionGrant, issue_device_link_code,
    issue_exchange_code,
};
