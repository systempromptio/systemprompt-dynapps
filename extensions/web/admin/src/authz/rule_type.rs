//! Compile-time-checked extension slugs for the tenant subject dimensions.
//!
//! Core's `RuleType` keeps its field private, so an extension dimension cannot
//! be a `const`. `ExtensionSlug` carries the slug instead: its constructor is a
//! `const fn` that asserts core's well-formedness rules, and binding it in a
//! `const` item turns a malformed slug into a compile error. The one runtime
//! conversion into `RuleType` lives here.

use systemprompt_security::authz::RuleType;

#[derive(Debug, Clone, Copy)]
pub(crate) struct ExtensionSlug(&'static str);

impl ExtensionSlug {
    pub(crate) const fn new(slug: &'static str) -> Self {
        assert!(
            is_well_formed(slug),
            "extension slug must be non-empty [a-z0-9_], not start or end with `_`, and not be `user` or `role`"
        );
        Self(slug)
    }
}

const fn is_well_formed(slug: &str) -> bool {
    let bytes = slug.as_bytes();
    if bytes.is_empty() || bytes[0] == b'_' || bytes[bytes.len() - 1] == b'_' {
        return false;
    }
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_') {
            return false;
        }
        i += 1;
    }
    !bytes_eq(bytes, b"user") && !bytes_eq(bytes, b"role")
}

const fn bytes_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

pub(crate) fn extension_rule_type(slug: ExtensionSlug) -> RuleType {
    RuleType::extension(slug.0)
        .unwrap_or_else(|e| unreachable!("`{}` is a well-formed slug: {e}", slug.0))
}
