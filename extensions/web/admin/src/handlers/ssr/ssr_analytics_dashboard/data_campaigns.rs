//! Evaluation state per managed resource for the Skills tab, read through
//! core's `CampaignRepository` — the same repository the Campaigns page lists
//! from — so the two screens can never disagree about whether a skill is under
//! evaluation. Grouped here by the resource a campaign optimises, because a
//! skill row is keyed by the resource its verified installs resolved to.

use std::collections::HashMap;

use systemprompt::evaluation::campaigns::repository::CampaignRepository;
use systemprompt::evaluation::models::CampaignStatus;

use crate::routes::evaluation_state::EvaluationState;

#[derive(Debug, Clone)]
pub(super) struct SkillCampaignSummary {
    pub campaigns: i64,
    pub active: i64,
    pub latest_status: &'static str,
}

/// Every campaign on the instance, folded by resource id. A repository error
/// degrades to "no evaluation state" with a warning, like every other tab
/// aggregate: the Skills table must still render its adoption figures.
pub(super) async fn load_skill_campaigns(
    state: &EvaluationState,
) -> HashMap<String, SkillCampaignSummary> {
    let records = match all_campaigns(&state.campaigns, &state.owner).await {
        Ok(records) => records,
        Err(e) => {
            tracing::warn!(error = %e, "campaign list for the Skills tab failed");
            return HashMap::new();
        },
    };
    let mut by_resource: HashMap<String, (SkillCampaignSummary, chrono::DateTime<chrono::Utc>)> =
        HashMap::new();
    for record in records {
        let key = record.policy.resource_id.as_str().to_owned();
        let is_active = record.status == CampaignStatus::Active;
        let entry = by_resource.entry(key).or_insert_with(|| {
            (
                SkillCampaignSummary {
                    campaigns: 0,
                    active: 0,
                    latest_status: record.status.as_str(),
                },
                record.created_at,
            )
        });
        entry.0.campaigns += 1;
        entry.0.active += i64::from(is_active);
        if record.created_at >= entry.1 {
            entry.0.latest_status = record.status.as_str();
            entry.1 = record.created_at;
        }
    }
    by_resource
        .into_iter()
        .map(|(key, (summary, _))| (key, summary))
        .collect()
}

// Why: the repository pages by id and hands back one row past the page size
// to signal more; the tab wants the whole set, so walk it to the end.
async fn all_campaigns(
    repository: &CampaignRepository,
    owner: &systemprompt::identifiers::UserId,
) -> systemprompt::evaluation::Result<
    Vec<systemprompt::evaluation::campaigns::repository::CampaignRecord>,
> {
    let mut all = Vec::new();
    let mut after = None;
    loop {
        let page = repository.list(owner, after.as_ref()).await?;
        let more = page.len() > CampaignRepository::PAGE_SIZE;
        after = page.last().map(|record| record.id.clone());
        all.extend(page);
        if !more || after.is_none() {
            return Ok(all);
        }
    }
}
