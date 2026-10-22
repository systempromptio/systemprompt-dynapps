//! Load-time checks over a parsed `rules.yaml`: every entity carries a `why`,
//! names a well-formed id or glob and a valid owner, declares at least one
//! band, and never both allows and denies the same subject.

use super::rules_yaml_types::{
    BandSpec, EntityDecl, EntityRef, EntityRefParseError, GLOB_ONLY_KINDS, OWNER_BUNDLE_PREFIX,
    RulesDoc,
};

#[derive(Debug, thiserror::Error)]
pub enum RulesValidationError {
    #[error(transparent)]
    Entity(#[from] EntityRefParseError),
    #[error("{entity}: `why` is required")]
    MissingWhy { entity: String },
    #[error("{entity}: only gateway_route and hook take a glob; write the id")]
    GlobNotAllowed { entity: String },
    #[error("{entity}: {kind} ids are generated, never written — use {kind}/*")]
    GlobRequired { entity: String, kind: String },
    #[error("{entity}: declares no allow and no deny")]
    NoBands { entity: String },
    #[error("{entity}: owner '{owner}' must be written as {OWNER_BUNDLE_PREFIX}<name>")]
    OwnerNotBundle { entity: String, owner: String },
    #[error("{entity}: a glob cannot name an owner")]
    GlobWithOwner { entity: String },
    #[error("{entity}: {band} '{subject}' is both allowed and denied")]
    Conflict {
        entity: String,
        band: &'static str,
        subject: String,
    },
    #[error("{entity}: {band} {problem}")]
    Subjects {
        entity: String,
        band: &'static str,
        problem: &'static str,
    },
}

impl EntityDecl {
    fn validate(&self) -> Result<(), RulesValidationError> {
        let entity: EntityRef = self.entity.parse()?;
        if self.why.trim().is_empty() {
            return Err(RulesValidationError::MissingWhy {
                entity: entity.to_string(),
            });
        }
        check_glob_shape(&entity)?;
        if self.allow.is_empty() && self.deny.is_empty() {
            return Err(RulesValidationError::NoBands {
                entity: entity.to_string(),
            });
        }
        self.check_owner(&entity)?;
        if let Some((band, subject)) = self.first_conflict() {
            return Err(RulesValidationError::Conflict {
                entity: entity.to_string(),
                band,
                subject: subject.to_owned(),
            });
        }
        self.check_band_subjects(&entity)
    }

    fn check_owner(&self, entity: &EntityRef) -> Result<(), RulesValidationError> {
        let Some(owner) = self.owner.as_deref() else {
            return Ok(());
        };
        match self.owner_bundle() {
            Some(name) if !name.trim().is_empty() => {},
            _ => {
                return Err(RulesValidationError::OwnerNotBundle {
                    entity: entity.to_string(),
                    owner: owner.to_owned(),
                });
            },
        }
        if entity.is_glob() {
            return Err(RulesValidationError::GlobWithOwner {
                entity: entity.to_string(),
            });
        }
        Ok(())
    }

    fn first_conflict(&self) -> Option<(&'static str, &str)> {
        let denied = self.deny.bands();
        self.allow.bands().into_iter().find_map(|(band, allowed)| {
            let (_, denied) = denied.iter().find(|(b, _)| *b == band)?;
            allowed
                .values()
                .iter()
                .find(|v| denied.values().contains(v))
                .map(|v| (band, v.as_str()))
        })
    }

    fn check_band_subjects(&self, entity: &EntityRef) -> Result<(), RulesValidationError> {
        let bands = self.allow.bands().into_iter().chain(self.deny.bands());
        for (band, spec) in bands {
            if let Some(problem) = subject_problem(spec) {
                return Err(RulesValidationError::Subjects {
                    entity: entity.to_string(),
                    band,
                    problem,
                });
            }
        }
        Ok(())
    }
}

fn check_glob_shape(entity: &EntityRef) -> Result<(), RulesValidationError> {
    match (entity.is_glob(), GLOB_ONLY_KINDS.contains(&entity.kind)) {
        (true, false) => Err(RulesValidationError::GlobNotAllowed {
            entity: entity.to_string(),
        }),
        (false, true) => Err(RulesValidationError::GlobRequired {
            entity: entity.to_string(),
            kind: entity.kind.as_str().to_owned(),
        }),
        (true, true) | (false, false) => Ok(()),
    }
}

fn subject_problem(spec: &BandSpec) -> Option<&'static str> {
    let values = spec.values();
    if values.is_empty() {
        Some("names no subjects")
    } else if values.iter().any(|v| v.trim().is_empty()) {
        Some("contains a blank subject")
    } else {
        None
    }
}

impl RulesDoc {
    pub fn validate(&self) -> Result<(), RulesValidationError> {
        self.entities.iter().try_for_each(EntityDecl::validate)
    }
}
