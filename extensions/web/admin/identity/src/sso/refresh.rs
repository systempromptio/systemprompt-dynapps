//! A remote document an identity provider publishes and rotates on its own
//! schedule — SAML federation metadata, OIDC discovery — held in memory and
//! kept fresh in the background.
//!
//! A successful fetch is served for `ttl`; a failed one keeps the last good
//! document and retries after `retry`. With no good document yet, [`get`]
//! answers `None` and the caller decides between an inline fetch and a pinned
//! fallback.
//!
//! [`get`]: RefreshingDoc::get

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use super::error::SsoProviderError;

type FetchFuture<T> = Pin<Box<dyn Future<Output = Result<T, SsoProviderError>> + Send>>;
type Fetcher<T> = Box<dyn Fn() -> FetchFuture<T> + Send + Sync>;

struct Inner<T> {
    name: &'static str,
    fetch: Fetcher<T>,
    ttl: Duration,
    retry: Duration,
    current: RwLock<Option<Arc<T>>>,
}

pub struct RefreshingDoc<T> {
    inner: Arc<Inner<T>>,
}

impl<T> std::fmt::Debug for RefreshingDoc<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RefreshingDoc")
            .field("name", &self.inner.name)
            .field("ttl", &self.inner.ttl)
            .finish_non_exhaustive()
    }
}

impl<T: Clone + Send + Sync + 'static> RefreshingDoc<T> {
    pub fn new<F, Fut>(name: &'static str, fetch: F, ttl: Duration, retry: Duration) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<T, SsoProviderError>> + Send + 'static,
    {
        let fetch: Fetcher<T> = Box::new(move || Box::pin(fetch()));
        Self {
            inner: Arc::new(Inner {
                name,
                fetch,
                ttl,
                retry,
                current: RwLock::new(None),
            }),
        }
    }

    #[must_use]
    pub fn get(&self) -> Option<Arc<T>> {
        self.inner
            .current
            .read()
            .map_or(None, |guard| guard.as_ref().map(Arc::clone))
    }

    // Why: an Entra ID discovery or metadata outage must not end sign-in, so a
    // failed fetch keeps answering the last good document.
    pub async fn refresh(&self) -> Result<Arc<T>, SsoProviderError> {
        refresh_inner(&self.inner).await
    }

    // Why: the IdP rotates on its own schedule; fetching at boot and on an
    // interval keeps the first sign-in after either from waiting on it. A
    // no-op outside a Tokio runtime — the caller then fetches on demand.
    pub fn spawn_refresh(&self) {
        let Ok(runtime) = tokio::runtime::Handle::try_current() else {
            tracing::warn!(
                doc = self.inner.name,
                "No runtime at boot; the document is fetched at first use"
            );
            return;
        };
        let inner = Arc::clone(&self.inner);
        runtime.spawn(async move {
            loop {
                let wait = match refresh_inner(&inner).await {
                    Ok(_) => inner.ttl,
                    Err(_) => inner.retry,
                };
                tokio::time::sleep(wait).await;
            }
        });
    }
}

async fn refresh_inner<T: Send + Sync>(inner: &Inner<T>) -> Result<Arc<T>, SsoProviderError> {
    match (inner.fetch)().await {
        Ok(doc) => {
            let doc = Arc::new(doc);
            if let Ok(mut current) = inner.current.write() {
                *current = Some(Arc::clone(&doc));
            }
            tracing::info!(doc = inner.name, "Fetched identity provider document");
            Ok(doc)
        },
        Err(e) => {
            let last_good = inner
                .current
                .read()
                .map_or(None, |guard| guard.as_ref().map(Arc::clone));
            if let Some(doc) = last_good {
                tracing::warn!(error = %e, doc = inner.name, "Identity provider document refresh failed; keeping the last good one");
                Ok(doc)
            } else {
                tracing::error!(error = %e, doc = inner.name, "Identity provider document unavailable");
                Err(e)
            }
        },
    }
}
