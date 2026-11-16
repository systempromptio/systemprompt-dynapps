//! Graph query strings, built from typed pairs and percent-encoded here so
//! no tool ever splices user text into a URL.

/// Ordered query pairs; setting a key again replaces its value.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GraphQuery {
    pairs: Vec<(String, String)>,
}

impl GraphQuery {
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with(mut self, key: &str, value: impl Into<String>) -> Self {
        self.set(key, value);
        self
    }

    pub fn set(&mut self, key: &str, value: impl Into<String>) {
        let value = value.into();
        if let Some(slot) = self.pairs.iter_mut().find(|(k, _)| k == key) {
            slot.1 = value;
        } else {
            self.pairs.push((key.to_owned(), value));
        }
    }

    pub fn remove(&mut self, key: &str) {
        self.pairs.retain(|(k, _)| k != key);
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.pairs
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }

    pub fn encode(&self) -> String {
        self.pairs
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&")
    }
}
