//! Gateway [`SafetyScanner`] implementation for the systemprompt template.
//!
//! [`SecretsScanner`] flags credentials from the installation's configured
//! catalog leaving the gateway in a model reply. It uses the same compiled
//! scanner as the governance chain and registers through
//! `register_safety_scanner!` under the name `secrets`; the gateway runs it
//! for any policy whose `safety.scanners` lists it and blocks the reply when
//! `safety.block_response_categories` includes `secret`.
//!
//! **Egress only.** The `secret_scan` governance policy already scans request
//! content, runs first (`enforce_governance` precedes
//! `enforce_request_safety`), and is first-deny-wins, so scanning the request
//! here too produced a second verdict on identical bytes under different
//! config — two owners for one question, and a request that could be denied by
//! whichever plane the operator had not thought to configure. Responses have
//! no such overlap: the governance chain is request-only, so this is the sole
//! thing standing between a model that echoes a credential and the client.
//!
//! The catalog is compiled once, when core's scanner registry runs this
//! crate's factory, from the same `secret_scan` block the chain reads, in the
//! active services root (a pinned bundle's `governance/` when one owns it,
//! else the baked tree), never through a second governance engine: an
//! inventory-built scanner cannot reach the injected one, and a second
//! engine's rate limiter would double every budget.

use systemprompt::gateway::{
    Finding, SafetyScanner, ScannerSettings, Severity, register_safety_scanner,
};
use systemprompt::wire::canonical::{CanonicalRequest, CanonicalResponse};

use systemprompt::governance::policy::{GovernanceConfig, GovernedInput, SecretScanner};

#[derive(Debug, Clone, Default)]
pub struct SecretsScanner {
    scanner: Option<SecretScanner>,
}

impl SecretsScanner {
    #[must_use]
    pub const fn new() -> Self {
        Self { scanner: None }
    }

    #[must_use]
    pub const fn with_scanner(scanner: SecretScanner) -> Self {
        Self {
            scanner: Some(scanner),
        }
    }

    #[must_use]
    pub fn from_services_root() -> Self {
        Self {
            scanner: configured_scanner(),
        }
    }
}

fn configured_scanner() -> Option<SecretScanner> {
    let root = systemprompt::loader::services_root::ServicesRootBootstrap::require()
        .inspect_err(|error| tracing::error!(%error, "response secret scanner: no services root"))
        .ok()?;
    let path = root.path.join("governance/config.yaml");
    let text = std::fs::read_to_string(&path)
        .inspect_err(|error| tracing::error!(%error, "response secret scanner: config unreadable"))
        .ok()?;
    let config = GovernanceConfig::parse(&text)
        .inspect_err(
            |error| tracing::error!(%error, "response secret scanner configuration rejected"),
        )
        .ok()?;
    let policy = config
        .policies
        .into_iter()
        .find(|policy| policy.id == "secret_scan" && policy.enabled)?;
    SecretScanner::from_policy_yaml(&policy.params)
        .inspect_err(|error| tracing::error!(%error, "response secret scanner patterns rejected"))
        .ok()
}

#[async_trait::async_trait]
impl SafetyScanner for SecretsScanner {
    fn name(&self) -> &'static str {
        "secrets"
    }

    async fn scan_request(
        &self,
        _req: &CanonicalRequest,
    ) -> Result<Vec<Finding>, systemprompt::gateway::ScanError> {
        Ok(Vec::new())
    }

    async fn scan_response_final(
        &self,
        response: &CanonicalResponse,
    ) -> Result<Vec<Finding>, systemprompt::gateway::ScanError> {
        let mut findings = Vec::new();
        for unit in response.content_units() {
            findings.extend(self.scan(&unit));
        }
        Ok(findings)
    }
}

impl SecretsScanner {
    fn scan(&self, text: &str) -> Vec<Finding> {
        let input = GovernedInput::prompt_text(text.to_owned());
        self.scanner
            .as_ref()
            .and_then(|scanner| scanner.detect(&input))
            .map_or_else(Vec::new, |hit| {
                let observation = hit.observation;
                vec![Finding {
                    phase: "response",
                    severity: if observation {
                        Severity::Low
                    } else {
                        Severity::High
                    },
                    category: if observation {
                        "secret_observation"
                    } else {
                        "secret"
                    }
                    .to_owned(),
                    excerpt: Some(format!("{}: {}", hit.pattern.id, hit.redacted)),
                    scanner: "secrets",
                    spans: Vec::new(),
                    replacement: None,
                }]
            })
    }
}

register_safety_scanner!(
    |_: &ScannerSettings| SecretsScanner::from_services_root(),
    name = "secrets"
);

/// PII scanner for the categories core's heuristic does not carry: formatted
/// US SSNs (SSA never-issued ranges excluded) and E.164 phone numbers.
///
/// Both directions, unlike [`SecretsScanner`]: the governance `secret_scan`
/// stage owns request-side *credentials*, but no ingress plane scans for PII,
/// so there is no second-verdict overlap to avoid. Detection is deliberately
/// conservative — only the unambiguous shapes — because a false positive here
/// blocks a customer's request, and "looks a bit like a phone number" is most
/// ten-digit figures in an analytics conversation.
#[derive(Debug, Clone, Copy, Default)]
pub struct PiiScanner;

impl PiiScanner {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl SafetyScanner for PiiScanner {
    fn name(&self) -> &'static str {
        "pii_extended"
    }

    async fn scan_request(
        &self,
        req: &CanonicalRequest,
    ) -> Result<Vec<Finding>, systemprompt::gateway::ScanError> {
        Ok(req
            .safety_parts(false)
            .into_iter()
            .flat_map(|(_, text)| pii_findings(&text, "request"))
            .collect())
    }

    async fn scan_request_history(
        &self,
        req: &CanonicalRequest,
    ) -> Result<Vec<Finding>, systemprompt::gateway::ScanError> {
        Ok(req
            .safety_parts(true)
            .into_iter()
            .flat_map(|(_, text)| pii_findings(&text, "request_history"))
            .collect())
    }

    async fn scan_response_final(
        &self,
        response: &CanonicalResponse,
    ) -> Result<Vec<Finding>, systemprompt::gateway::ScanError> {
        Ok(response
            .content_units()
            .into_iter()
            .flat_map(|unit| pii_findings(&unit, "response"))
            .collect())
    }
}

fn pii_findings(text: &str, phase: &'static str) -> Vec<Finding> {
    let mut findings = Vec::new();
    if let Some(excerpt) = find_ssn(text) {
        findings.push(Finding {
            phase,
            severity: Severity::High,
            category: "pii_ssn".to_owned(),
            excerpt: Some(excerpt),
            scanner: "pii_extended",
            spans: Vec::new(),
            replacement: None,
        });
    }
    if let Some(excerpt) = find_e164_phone(text) {
        findings.push(Finding {
            phase,
            severity: Severity::Medium,
            category: "pii_phone".to_owned(),
            excerpt: Some(excerpt),
            scanner: "pii_extended",
            spans: Vec::new(),
            replacement: None,
        });
    }
    findings
}

fn find_ssn(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    for i in 0..bytes.len().saturating_sub(10) {
        let w = &bytes[i..i + 11];
        let shape_ok = w[..3].iter().all(u8::is_ascii_digit)
            && w[3] == b'-'
            && w[4..6].iter().all(u8::is_ascii_digit)
            && w[6] == b'-'
            && w[7..].iter().all(u8::is_ascii_digit);
        if !shape_ok || digit_adjacent(bytes, i, i + 11) {
            continue;
        }
        let (area, group, serial) = (&text[i..i + 3], &text[i + 4..i + 6], &text[i + 7..i + 11]);
        if !ssa_never_issued(area, group, serial) {
            return Some(format!("***-**-{serial}"));
        }
    }
    None
}

fn ssa_never_issued(area: &str, group: &str, serial: &str) -> bool {
    area == "000" || area == "666" || area.starts_with('9') || group == "00" || serial == "0000"
}

const E164_DIGITS: std::ops::RangeInclusive<usize> = 8..=15;

fn find_e164_phone(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    for (i, _) in bytes.iter().enumerate().filter(|&(_, &b)| b == b'+') {
        let mut digits = 0usize;
        let mut end = i + 1;
        for (j, &b) in bytes.iter().enumerate().skip(i + 1) {
            match b {
                b'0'..=b'9' => {
                    digits += 1;
                    end = j + 1;
                },
                b' ' | b'-' | b'(' | b')' | b'.' => {},
                _ => break,
            }
            if digits > *E164_DIGITS.end() {
                break;
            }
        }
        if E164_DIGITS.contains(&digits) && !digit_adjacent(bytes, i, end) {
            let tail: String = text[i..end]
                .chars()
                .filter(char::is_ascii_digit)
                .skip(digits.saturating_sub(3))
                .collect();
            return Some(format!("+…{tail}"));
        }
    }
    None
}

fn digit_adjacent(bytes: &[u8], start: usize, end: usize) -> bool {
    let before = start.checked_sub(1).map(|i| bytes[i]);
    let after = bytes.get(end).copied();
    before.is_some_and(|b| b.is_ascii_digit()) || after.is_some_and(|b| b.is_ascii_digit())
}

register_safety_scanner!(
    |_: &ScannerSettings| PiiScanner::new(),
    name = "pii_extended"
);
