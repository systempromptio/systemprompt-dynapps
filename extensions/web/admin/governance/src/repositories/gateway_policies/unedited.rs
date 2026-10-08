//! Safety settings the policy form does not edit, carried over on save.
//!
//! The form has no fields for `scanner_settings` (per-scanner fail mode,
//! timeout and opaque config) or `redact_categories`, so a spec parsed from it
//! alone would silently drop what the file or an earlier save declared. Saving
//! keeps the stored values, and keeps a scanner's settings only while the
//! scanner is still listed: core refuses settings for a scanner not in
//! `scanners`.

use systemprompt::gateway::SafetyConfig;

pub fn carry_unedited_safety(edited: &mut SafetyConfig, stored: &SafetyConfig) {
    edited.scanner_settings = stored
        .scanner_settings
        .iter()
        .filter(|(scanner, _)| edited.scanners.contains(scanner))
        .map(|(scanner, settings)| (scanner.clone(), settings.clone()))
        .collect();
    edited
        .redact_categories
        .clone_from(&stored.redact_categories);
}
