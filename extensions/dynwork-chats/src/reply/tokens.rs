//! Token counts as `OpenCode` writes them (`input`, `output`, `reasoning`,
//! `cache.read`, `cache.write`), summed over the steps of a thread.

// JSON: the counts are read from and written to `OpenCode` documents.
use serde_json::{Value, json};

use crate::reply::model::Usage;

/// A token count as `OpenCode` writes it, summed over steps.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Tokens {
    pub input: u64,
    pub output: u64,
    pub reasoning: u64,
    pub cache_read: u64,
    pub cache_write: u64,
}

impl Tokens {
    pub const fn of(usage: &Usage) -> Self {
        Self {
            input: usage.input,
            output: usage.output,
            reasoning: usage.reasoning,
            cache_read: usage.cache_read,
            cache_write: usage.cache_write,
        }
    }

    pub const fn plus(self, other: Self) -> Self {
        Self {
            input: self.input + other.input,
            output: self.output + other.output,
            reasoning: self.reasoning + other.reasoning,
            cache_read: self.cache_read + other.cache_read,
            cache_write: self.cache_write + other.cache_write,
        }
    }

    pub fn to_json(self) -> Value {
        json!({
            "input": self.input,
            "output": self.output,
            "reasoning": self.reasoning,
            "cache": { "read": self.cache_read, "write": self.cache_write },
        })
    }

    pub fn from_json(value: &Value) -> Self {
        let read = |path: &[&str]| {
            path.iter()
                .try_fold(value, |node, key| node.get(key))
                .and_then(Value::as_u64)
                .unwrap_or(0)
        };
        Self {
            input: read(&["input"]),
            output: read(&["output"]),
            reasoning: read(&["reasoning"]),
            cache_read: read(&["cache", "read"]),
            cache_write: read(&["cache", "write"]),
        }
    }
}
