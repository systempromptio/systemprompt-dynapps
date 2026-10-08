//! What a page could not load, said on the page.
//!
//! A server-rendered panel that fails to read falls back to its empty value —
//! the page still renders — but the fallback is never silent: every fallback
//! is recorded here, and the shell draws the list as an "unavailable" notice
//! above the content. A spend or governance page that shows a zero because
//! the database was down is wrong data presented as real; a page that says
//! "group usage could not be read" is a page an operator can trust.
//!
//! One [`Faults`] per request. Loaders take `&mut Faults` and hand every
//! `Result` to [`Faults::take`], which logs once and returns the default;
//! shared builders that render their own partial carry a `Vec<Fault>` of
//! their own instead of threading the page's collector through.

use std::fmt::Display;

use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Fault {
    pub what: &'static str,
    pub error: String,
}

#[derive(Debug, Default)]
pub struct Faults {
    list: Vec<Fault>,
}

impl Faults {
    #[must_use]
    pub const fn new() -> Self {
        Self { list: Vec::new() }
    }

    pub fn record<E: Display>(&mut self, what: &'static str, error: &E) {
        tracing::warn!(what, error = %error, "page panel unavailable");
        self.list.push(Fault {
            what,
            error: error.to_string(),
        });
    }

    pub fn take<T: Default, E: Display>(&mut self, what: &'static str, result: Result<T, E>) -> T {
        self.take_or(what, result, T::default())
    }

    pub fn take_or<T, E: Display>(
        &mut self,
        what: &'static str,
        result: Result<T, E>,
        fallback: T,
    ) -> T {
        match result {
            Ok(value) => value,
            Err(error) => {
                self.record(what, &error);
                fallback
            },
        }
    }

    pub fn take_with<T, E: Display>(
        &mut self,
        what: &'static str,
        result: Result<T, E>,
        fallback: impl FnOnce() -> T,
    ) -> T {
        match result {
            Ok(value) => value,
            Err(error) => {
                self.record(what, &error);
                fallback()
            },
        }
    }

    pub fn extend(&mut self, faults: impl IntoIterator<Item = Fault>) {
        self.list.extend(faults);
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    #[must_use]
    pub fn as_slice(&self) -> &[Fault] {
        &self.list
    }

    #[must_use]
    pub fn into_vec(self) -> Vec<Fault> {
        self.list
    }
}
