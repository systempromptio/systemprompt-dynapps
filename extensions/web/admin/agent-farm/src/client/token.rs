//! Where the client's farm-api token comes from.

use std::fmt;
use std::sync::Arc;

use systemprompt::traits::SecretStore;

#[derive(Clone)]
pub(super) enum Token {
    Store {
        store: Arc<dyn SecretStore>,
        key: String,
    },
    Fixed(String),
}

impl Token {
    pub(super) fn current(&self) -> Option<String> {
        let token = match self {
            Self::Store { store, key } => store.secret(key)?.expose().trim().to_owned(),
            Self::Fixed(token) => token.clone(),
        };
        Some(token).filter(|t| !t.is_empty())
    }
}

impl fmt::Debug for Token {
    // Why: a derived Debug would print a fixed token.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Store { key, .. } => f.debug_struct("Store").field("key", key).finish(),
            Self::Fixed(_) => f.write_str("Fixed"),
        }
    }
}
