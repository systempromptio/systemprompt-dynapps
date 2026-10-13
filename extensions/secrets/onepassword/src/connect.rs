//! The 1Password Connect backend: a self-hosted Connect server reached over
//! HTTPS with a bearer token.
//!
//! Redirects are never followed (the bearer token must not leave the
//! configured host). 429, 5xx, timeouts and refused connections are retried
//! on the [`RetryPolicy`]; 401/403/404 fail at once.

use std::fmt;
use std::time::Duration;

use async_trait::async_trait;
use reqwest::{Client, redirect};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use urlencoding::encode;
use zeroize::Zeroizing;

use crate::backend::Backend;
use crate::error::OnePasswordError;
use crate::item::Item;
use crate::retry::{RetryPolicy, StatusClass, classify_status};

const CONNECT_TIMEOUT_CAP: Duration = Duration::from_secs(5);

#[derive(Debug, Deserialize)]
struct Summary {
    id: String,
}

pub struct ConnectBackend {
    base_url: String,
    token: Zeroizing<String>,
    client: Client,
    policy: RetryPolicy,
    timeout: Duration,
}

impl fmt::Debug for ConnectBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConnectBackend")
            .field("base_url", &self.base_url)
            .field("token", &"<redacted>")
            .field("policy", &self.policy)
            .finish_non_exhaustive()
    }
}

impl ConnectBackend {
    pub fn new(
        base_url: &str,
        token: Zeroizing<String>,
        timeout: Duration,
        retries: u32,
    ) -> Result<Self, OnePasswordError> {
        let client = Client::builder()
            .timeout(timeout)
            .connect_timeout(timeout.min(CONNECT_TIMEOUT_CAP))
            .redirect(redirect::Policy::none())
            .build()
            .map_err(OnePasswordError::Http)?;
        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_owned(),
            token,
            client,
            policy: RetryPolicy::new(retries),
            timeout,
        })
    }

    async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, OnePasswordError> {
        let url = format!("{}{path}", self.base_url);
        let delays = self.policy.delays();
        let mut attempt = 0;
        loop {
            let sent = self
                .client
                .get(&url)
                .bearer_auth(self.token.as_str())
                .send()
                .await;
            let failure = match sent {
                Ok(response) => {
                    let code = response.status().as_u16();
                    let status = OnePasswordError::Status {
                        code,
                        path: path.to_owned(),
                    };
                    match classify_status(code) {
                        StatusClass::Success => {
                            return response.json::<T>().await.map_err(|source| {
                                OnePasswordError::ResponseBody {
                                    path: path.to_owned(),
                                    source,
                                }
                            });
                        },
                        StatusClass::Retryable => status,
                        StatusClass::Fatal => return Err(status),
                    }
                },
                Err(e) if e.is_timeout() => OnePasswordError::Timeout(self.timeout.as_secs()),
                Err(e) if e.is_connect() => OnePasswordError::Http(e),
                Err(e) => return Err(OnePasswordError::Http(e)),
            };
            let Some(delay) = delays.get(attempt) else {
                return Err(failure);
            };
            tracing::debug!(attempt, path, error = %failure, "1Password Connect request retrying");
            tokio::time::sleep(*delay).await;
            attempt += 1;
        }
    }

    async fn vault_id(&self, vault: &str) -> Result<String, OnePasswordError> {
        let filter = format!("name eq \"{vault}\"");
        let found: Vec<Summary> = self
            .get_json(&format!("/v1/vaults?filter={}", encode(&filter)))
            .await?;
        match found.as_slice() {
            [one] => Ok(one.id.clone()),
            [] => self
                .get_json::<Summary>(&format!("/v1/vaults/{}", encode(vault)))
                .await
                .map(|s| s.id)
                .map_err(|e| not_found_as(e, || OnePasswordError::VaultNotFound(vault.to_owned()))),
            _ => Err(OnePasswordError::AmbiguousItem {
                what: "vault",
                name: vault.to_owned(),
            }),
        }
    }

    async fn item_id(
        &self,
        vault_id: &str,
        vault: &str,
        title: &str,
    ) -> Result<String, OnePasswordError> {
        let filter = format!("title eq \"{title}\"");
        let found: Vec<Summary> = self
            .get_json(&format!(
                "/v1/vaults/{}/items?filter={}",
                encode(vault_id),
                encode(&filter)
            ))
            .await?;
        match found.as_slice() {
            [one] => Ok(one.id.clone()),
            [] => Ok(title.to_owned()),
            _ => Err(OnePasswordError::AmbiguousItem {
                what: "item",
                name: format!("{vault}/{title}"),
            }),
        }
    }
}

fn not_found_as(
    error: OnePasswordError,
    replace: impl FnOnce() -> OnePasswordError,
) -> OnePasswordError {
    match error {
        OnePasswordError::Status { code: 404, .. } => replace(),
        other => other,
    }
}

#[async_trait]
impl Backend for ConnectBackend {
    fn label(&self) -> &'static str {
        "connect"
    }

    async fn fetch_items(
        &self,
        vault: &str,
        titles: &[String],
    ) -> Result<Vec<Item>, OnePasswordError> {
        let vault_id = self.vault_id(vault).await?;
        let mut items = Vec::with_capacity(titles.len());
        for title in titles {
            let item_id = self.item_id(&vault_id, vault, title).await?;
            let item: Item = self
                .get_json(&format!(
                    "/v1/vaults/{}/items/{}",
                    encode(&vault_id),
                    encode(&item_id)
                ))
                .await
                .map_err(|e| {
                    not_found_as(e, || OnePasswordError::ItemNotFound {
                        vault: vault.to_owned(),
                        item: title.clone(),
                    })
                })?;
            items.push(item);
        }
        Ok(items)
    }
}
