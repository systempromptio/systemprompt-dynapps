//! Load-time checks over a parsed `rules.yaml`: every entity carries a `why`,
//! names a well-formed id or glob and a valid owner, declares at least one
//! band, and never both allows and denies the same subject.

use super::rules_yaml_types::{
    BandSpec, EntityDecl, EntityRef, GLOB_ONLY_KINDS, OWNER_BUNDLE_PREFIX, RulesDoc,
};

impl EntityDecl {
    fn validate(&self) -> Result<(), String> {
        let entity: EntityRef = self.entity.parse()?;
        if self.why.trim().is_empty() {
            return Err(format!("{entity}: `why` is required"));
        }
        check_glob_shape(&entity)?;
        if self.allow.is_empty() && self.deny.is_empty() {
            return Err(format!("{entity}: declares no allow and no deny"));
        }
        self.check_owner(&entity)?;
        if let Some((band, subject)) = self.first_conflict() {
            return Err(format!(
                "{entity}: {band} '{subject}' is both allowed and denied"
            ));
        }
        self.check_band_subjects(&entity)
    }

    fn check_owner(&self, entity: &EntityRef) -> Result<(), String> {
        let Some(owner) = self.owner.as_deref() else {
            return Ok(());
        };
        match self.owner_bundle() {
            Some(name) if !name.trim().is_empty() => {},
            _ => {
                return Err(format!(
                    "{entity}: owner '{owner}' must be written as {OWNER_BUNDLE_PREFIX}<name>"
                ));
            },
        }
        if entity.is_glob() {
            return Err(format!("{entity}: a glob cannot name an owner"));
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

    fn check_band_subjects(&self, entity: &EntityRef) -> Result<(), String> {
        let bands = self.allow.bands().into_iter().chain(self.deny.bands());
        for (band, spec) in bands {
            if let Some(problem) = subject_problem(spec) {
                return Err(format!("{entity}: {band} {problem}"));
            }
        }
        Ok(())
    }
}
fn check_glob_shape(entity: &EntityRef) -> Result<(), String> {
    let kind = entity.kind.as_str();
    match (entity.is_glob(), GLOB_ONLY_KINDS.contains(&entity.kind)) {
        (true, false) => Err(format!(
            "{entity}: only gateway_route and hook take a glob; write the id"
        )),
        (false, true) => Err(format!(
            "{entity}: {kind} ids are generated, never written — use {kind}/*"
        )),
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
    pub fn validate(&self) -> Result<(), String> {
        self.entities.iter().try_for_each(EntityDecl::validate)
    }
}
