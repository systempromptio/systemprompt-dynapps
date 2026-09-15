//! An execution admission that accepts deterministic fixtures. Production
//! admission demands a reviewed native proof for each variant's exact
//! client/version/image and this harness ships none, so every experiment the
//! evaluation suites create would be refused at budget reservation — before
//! the fences, pins and owner scoping they actually test ever ran.

use std::sync::Arc;
use systemprompt::evaluation::capabilities::{ExecutionAdmission, paired_variants};
use systemprompt::evaluation::experiments::{ExecutionMode, ExperimentSpec};

#[derive(Debug)]
struct FixtureAdmission;

impl ExecutionAdmission for FixtureAdmission {
    fn admit(&self, spec: &ExperimentSpec) -> systemprompt::evaluation::Result<()> {
        spec.validate()?;
        paired_variants(spec)?;
        if spec.execution_mode != ExecutionMode::Fixture {
            return Err(systemprompt::evaluation::EvaluationError::InvalidSpec(
                "The integration harness admits deterministic fixtures only".to_owned(),
            ));
        }
        Ok(())
    }
}

pub(crate) fn fixture_admission() -> Arc<dyn ExecutionAdmission> {
    Arc::new(FixtureAdmission)
}
