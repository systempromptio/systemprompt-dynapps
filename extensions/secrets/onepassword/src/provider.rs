//! [`OnePasswordProvider`]: core's `DynSecretsProvider` for
//! `secrets.external.kind: onepassword`.
//!
//! Each fetch reads every distinct item once and maps it with
//! [`map_fields`]. When the CLI cannot return an item whole, its references
//! are read one by one with `op read` before the fetch gives up.

use std::collections::BTreeMap;
use std::fmt;
use std::time::Duration;

use systemprompt::config::{
    DynSecretsProvider, EnvLookup, ExternalSecretsConfig, SecretsBootstrapError, SecretsDocument,
    SecretsFetch, SharedSecretsProvider,
};
use zeroize::Zeroizing;

use crate::backend::Backend;
use crate::cli::CliBackend;
use crate::config::{BackendKind, Mapping, OnePasswordConfig};
use crate::connect::ConnectBackend;
use crate::error::OnePasswordError;
use crate::item::{Field, Item};
use crate::mapping::map_fields;
use crate::reference::OpReference;

pub struct OnePasswordProvider {
    backend: Box<dyn Backend>,
    vault: String,
    mapping: Mapping,
    refresh: Option<Duration>,
}

impl fmt::Debug for OnePasswordProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OnePasswordProvider")
            .field("backend", &self.backend.label())
            .field("vault", &self.vault)
            .field("refresh", &self.refresh)
            .finish_non_exhaustive()
    }
}

impl OnePasswordProvider {
    pub fn from_config(
        config: &OnePasswordConfig,
        refresh_secs: Option<u64>,
        env: &dyn Fn(&str) -> Option<String>,
    ) -> Result<Self, OnePasswordError> {
        let token = env(&config.token_env)
            .filter(|t| !t.trim().is_empty())
            .map(Zeroizing::new)
            .ok_or_else(|| OnePasswordError::TokenMissing {
                env: config.token_env.clone(),
            })?;
        let backend: Box<dyn Backend> = match config.backend {
            BackendKind::Cli => Box::new(CliBackend::new(
                config.op_bin.clone(),
                token,
                CliBackend::passthrough_from(env),
                config.timeout(),
            )),
            BackendKind::Connect => {
                let url = config.connect_url.as_deref().ok_or_else(|| {
                    OnePasswordError::Config("backend: connect requires connect_url".to_owned())
                })?;
                Box::new(ConnectBackend::new(
                    url,
                    token,
                    config.timeout(),
                    config.retries,
                )?)
            },
        };
        Ok(Self::with_backend(
            backend,
            config,
            refresh_secs.map(Duration::from_secs),
        ))
    }

    #[must_use]
    pub fn with_backend(
        backend: Box<dyn Backend>,
        config: &OnePasswordConfig,
        refresh: Option<Duration>,
    ) -> Self {
        Self {
            backend,
            vault: config.vault.clone(),
            mapping: config.mapping.clone(),
            refresh,
        }
    }

    pub async fn fetch(&self) -> Result<SecretsDocument, OnePasswordError> {
        let items = match &self.mapping {
            Mapping::SingleItem { item } => self.fetch_group(&self.vault, item, &[]).await?,
            Mapping::Items(refs) => {
                let mut groups: BTreeMap<(&str, &str), Vec<&OpReference>> = BTreeMap::new();
                for reference in refs.values() {
                    groups
                        .entry((reference.vault.as_str(), reference.item.as_str()))
                        .or_default()
                        .push(reference);
                }
                let mut items = Vec::new();
                for ((vault, title), group) in groups {
                    items.extend(self.fetch_group(vault, title, &group).await?);
                }
                items
            },
        };
        map_fields(&items, &self.mapping)
    }

    async fn fetch_group(
        &self,
        vault: &str,
        title: &str,
        refs: &[&OpReference],
    ) -> Result<Vec<Item>, OnePasswordError> {
        match self.backend.fetch_items(vault, &[title.to_owned()]).await {
            Ok(mut items) => {
                for item in &mut items {
                    vault.clone_into(&mut item.vault);
                }
                Ok(items)
            },
            Err(error) if !refs.is_empty() && error.allows_reference_fallback() => {
                self.read_each(vault, title, refs, error).await
            },
            Err(error) => Err(error),
        }
    }

    async fn read_each(
        &self,
        vault: &str,
        title: &str,
        refs: &[&OpReference],
        original: OnePasswordError,
    ) -> Result<Vec<Item>, OnePasswordError> {
        let mut fields = Vec::with_capacity(refs.len());
        for reference in refs {
            match self.backend.read_reference(reference).await {
                Ok(value) => fields.push(Field::concealed(&reference.field, value)),
                Err(OnePasswordError::Unsupported) => return Err(original),
                Err(error) => return Err(error),
            }
        }
        tracing::warn!(vault, item = title, error = %original, "op item get failed; read each reference instead");
        Ok(vec![Item {
            vault: vault.to_owned(),
            id: title.to_owned(),
            title: title.to_owned(),
            fields,
        }])
    }
}

impl DynSecretsProvider for OnePasswordProvider {
    fn fetch_boxed(&self) -> SecretsFetch<'_> {
        Box::pin(async move { self.fetch().await.map_err(SecretsBootstrapError::from) })
    }

    fn describe(&self) -> String {
        format!("onepassword {} vault {}", self.backend.label(), self.vault)
    }

    fn refresh_hint(&self) -> Option<Duration> {
        self.refresh
    }
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "core's SecretsSourceFactory fn-pointer type hands EnvLookup by value"
)]
pub fn build(
    cfg: &ExternalSecretsConfig,
    env: EnvLookup,
) -> Result<SharedSecretsProvider, SecretsBootstrapError> {
    let config = OnePasswordConfig::from_external(cfg)?;
    let provider = OnePasswordProvider::from_config(&config, cfg.refresh_secs, &*env)?;
    Ok(Box::new(provider))
}
