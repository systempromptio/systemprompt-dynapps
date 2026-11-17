//! What a draft must be to be saved, and what a template must be to be
//! published: the contract's "Don't ask" controls and the restrictions on a
//! mandatory template. Each refusal names the one rule that failed.

use super::{FolderNeed, Permissions, TemplateDraft};

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Refusal {
    #[error("a template names at least one anchor plugin")]
    NoPlugins,
    #[error("a template's name, schedule, anchor plugins and connectors are not blank")]
    Blank,
    #[error("connectors name Governance servers one by one, never \"all\"")]
    AllConnectors,
    #[error("a template that runs without asking names the connectors it may use")]
    BypassWithoutConnectors,
    #[error("a template that runs without asking needs a folder chosen at activation")]
    BypassWithoutFolder,
    #[error("a mandatory template never runs without asking")]
    BypassMandatory,
    #[error("a mandatory template cannot require a folder: nobody is asked for one")]
    MandatoryFolder,
    #[error("the time limit is above the maximum the template locks")]
    TimeLimitOverMax,
}

pub fn check_draft(draft: &TemplateDraft) -> Result<(), Refusal> {
    if draft.plugins.is_empty() {
        return Err(Refusal::NoPlugins);
    }
    if draft.plugins.iter().any(|plugin| plugin.trim().is_empty()) {
        return Err(Refusal::Blank);
    }
    Ok(())
}

pub fn check_publishable(draft: &TemplateDraft) -> Result<(), Refusal> {
    check_draft(draft)?;
    let body = &draft.body;
    let options = &body.definition.options;
    if body.name.trim().is_empty()
        || body.definition.schedule.cron.trim().is_empty()
        || options.connectors.iter().any(|name| name.trim().is_empty())
    {
        return Err(Refusal::Blank);
    }
    if options.connectors.iter().any(|name| name == "all") {
        return Err(Refusal::AllConnectors);
    }
    if options.permissions == Permissions::Bypass {
        check_bypass(draft)?;
    }
    if draft.mandatory && body.personal.folder == FolderNeed::Required {
        return Err(Refusal::MandatoryFolder);
    }
    let over_max = options
        .time_limit
        .zip(body.locks.time_limit_max)
        .is_some_and(|(limit, max)| limit > max);
    if over_max {
        return Err(Refusal::TimeLimitOverMax);
    }
    Ok(())
}

fn check_bypass(draft: &TemplateDraft) -> Result<(), Refusal> {
    if draft.mandatory {
        return Err(Refusal::BypassMandatory);
    }
    if draft.body.definition.options.connectors.is_empty() {
        return Err(Refusal::BypassWithoutConnectors);
    }
    if draft.body.personal.folder != FolderNeed::Required {
        return Err(Refusal::BypassWithoutFolder);
    }
    Ok(())
}
