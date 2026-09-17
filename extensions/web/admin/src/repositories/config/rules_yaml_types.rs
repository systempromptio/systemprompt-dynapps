//! Wire types for `services/access-control/rules.yaml`, the one declarative
//! source of entitlement on this instance.
//!
//! The file is entity-centric: one block per entity, each band under `allow`
//! or `deny` listing the subjects it names. The band keys are the database
//! `rule_type` values verbatim, which is what lets the export module render a
//! database back into this exact shape. The same types serialise, so a
//! round-trip (parse → project → export → parse) is the identity.
//!
//! `why` is mandatory per entity. It lands on every rule row of that entity
//! as its `justification`, so the ledger can finally say why a rule exists
//! instead of leaving the column blank.

use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use systemprompt_security::authz::{Access, EntityKind};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RulesDoc {
    #[serde(default)]
    pub entities: Vec<EntityDecl>,
}

/// One entity block.
///
/// `entity` is `<kind>/<id>`, or `<kind>/*` for the glob-only kinds. `owner`
/// is `bundle:<name>` when the entity arrives in a remote services bundle
/// rather than this tree: access is still declared here — a kit never widens
/// its own audience — but the id may be absent until that bundle is active,
/// and is reported as *awaiting its bundle*.
///
/// `valid_until` is an RFC 3339 instant after which every rule of the entity
/// stops applying. It lands on each rule row's validity record; the hourly
/// expiry sweep deletes the rows once it passes, and the loader treats a
/// declaration already past it as not declared.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntityDecl {
    pub entity: String,
    #[serde(default)]
    pub default: EntityDefault,
    pub why: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "BandMap::is_empty")]
    pub allow: BandMap,
    #[serde(default, skip_serializing_if = "BandMap::is_empty")]
    pub deny: BandMap,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntityDefault {
    Open,
    #[default]
    Closed,
}

impl EntityDefault {
    #[must_use]
    pub const fn included(self) -> bool {
        matches!(self, Self::Open)
    }

    #[must_use]
    pub const fn from_included(included: bool) -> Self {
        if included { Self::Open } else { Self::Closed }
    }
}

/// The four subject bands this instance writes rules at. `user` is absent on
/// purpose: per-person overrides are dashboard-only and never declared.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BandMap {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<BandSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<BandSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<BandSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub salesforce: Option<BandSpec>,
}

impl BandMap {
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.role.is_none()
            && self.group.is_none()
            && self.project.is_none()
            && self.salesforce.is_none()
    }

    // Why: ladder order, narrowest first — the order the resolver consults.
    #[must_use]
    pub fn bands(&self) -> Vec<(&'static str, &BandSpec)> {
        [
            ("project", &self.project),
            ("group", &self.group),
            ("salesforce", &self.salesforce),
            ("role", &self.role),
        ]
        .into_iter()
        .filter_map(|(name, spec)| spec.as_ref().map(|s| (name, s)))
        .collect()
    }

    pub fn set(&mut self, rule_type: &str, spec: BandSpec) {
        match rule_type {
            "role" => self.role = Some(spec),
            "group" => self.group = Some(spec),
            "project" => self.project = Some(spec),
            "salesforce" => self.salesforce = Some(spec),
            _ => {},
        }
    }
}

/// A band is a plain list of subjects, or a list with its own `why` when the
/// reason differs from the entity's.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BandSpec {
    List(Vec<String>),
    Detailed { values: Vec<String>, why: String },
}

impl BandSpec {
    #[must_use]
    pub fn values(&self) -> &[String] {
        match self {
            Self::List(v) | Self::Detailed { values: v, .. } => v,
        }
    }

    #[must_use]
    pub fn why(&self) -> Option<&str> {
        match self {
            Self::List(_) => None,
            Self::Detailed { why, .. } => Some(why),
        }
    }
}

/// A parsed `<kind>/<id>` reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityRef {
    pub kind: EntityKind,
    pub id: String,
}

impl EntityRef {
    #[must_use]
    pub fn is_glob(&self) -> bool {
        self.id.contains('*')
    }
}

impl FromStr for EntityRef {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (kind, id) = s
            .split_once('/')
            .ok_or_else(|| format!("entity '{s}' must be written as <kind>/<id>"))?;
        if id.trim().is_empty() {
            return Err(format!("entity '{s}' names no id"));
        }
        let kind = EntityKind::from_str(kind).map_err(|e| e.to_string())?;
        Ok(Self {
            kind,
            id: id.to_owned(),
        })
    }
}

impl fmt::Display for EntityRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.kind.as_str(), self.id)
    }
}

// Why: the rest of an `owner:` value is the source name from the profile's
// `services.sources[]`, so the prefix is the only fixed part.
pub const OWNER_BUNDLE_PREFIX: &str = "bundle:";

impl EntityDecl {
    #[must_use]
    pub fn owner_bundle(&self) -> Option<&str> {
        self.owner.as_deref()?.strip_prefix(OWNER_BUNDLE_PREFIX)
    }
}

pub const BAND_NAMES: [&str; 4] = ["role", "group", "project", "salesforce"];

// Why: the kinds whose ids are generated rather than authored. A literal id of
// one of these names a row that cannot exist, so only the glob is accepted.
pub const GLOB_ONLY_KINDS: [EntityKind; 2] = [EntityKind::GatewayRoute, EntityKind::Hook];

impl RulesDoc {
    // Why: structural checks that need no catalog — references parse, `why`
    // is present, globs only where ids are generated, no subject both allowed
    // and denied on the same band, owners well-formed.
    pub fn validate(&self) -> Result<(), String> {
        for decl in &self.entities {
            let entity: EntityRef = decl.entity.parse()?;
            if decl.why.trim().is_empty() {
                return Err(format!("{entity}: `why` is required"));
            }
            let glob_only = GLOB_ONLY_KINDS.contains(&entity.kind);
            if entity.is_glob() && !glob_only {
                return Err(format!(
                    "{entity}: only gateway_route and hook take a glob; write the id"
                ));
            }
            if !entity.is_glob() && glob_only {
                return Err(format!(
                    "{entity}: {} ids are generated, never written — use {}/*",
                    entity.kind.as_str(),
                    entity.kind.as_str()
                ));
            }
            if decl.allow.is_empty() && decl.deny.is_empty() {
                return Err(format!("{entity}: declares no allow and no deny"));
            }
            if let Some(owner) = decl.owner.as_deref() {
                if decl.owner_bundle().is_none_or(|n| n.trim().is_empty()) {
                    return Err(format!(
                        "{entity}: owner '{owner}' must be written as {OWNER_BUNDLE_PREFIX}<name>"
                    ));
                }
                if entity.is_glob() {
                    return Err(format!("{entity}: a glob cannot name an owner"));
                }
            }
            for (band, allowed) in decl.allow.bands() {
                let denied = decl.deny.bands().into_iter().find(|(b, _)| *b == band);
                if let Some((_, denied)) = denied
                    && let Some(v) = allowed
                        .values()
                        .iter()
                        .find(|v| denied.values().contains(v))
                {
                    return Err(format!("{entity}: {band} '{v}' is both allowed and denied"));
                }
            }
            for (band, spec) in decl.allow.bands().into_iter().chain(decl.deny.bands()) {
                if spec.values().is_empty() {
                    return Err(format!("{entity}: {band} names no subjects"));
                }
                if spec.values().iter().any(|v| v.trim().is_empty()) {
                    return Err(format!("{entity}: {band} contains a blank subject"));
                }
            }
        }
        Ok(())
    }
}

#[must_use]
pub const fn access_of(band_is_allow: bool) -> Access {
    if band_is_allow {
        Access::Allow
    } else {
        Access::Deny
    }
}
