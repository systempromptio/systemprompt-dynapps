//! The active tab's body for the Skills page. Only the active tab's figures
//! are computed; every other field of `ActiveTabBody` stays at its default.

use chrono::NaiveDate;

use super::activity::{chart, top_skills};
use super::context::SkillsRead;
use super::rows::{group_rows, skill_row_view};
use super::summary::{kpis, unused};
use super::views::{
    MarketplaceGroupView, SkillFactView, SkillsKpiView, TopSkillView, UnusedSkillView,
};
use super::{SkillsQuery, SkillsTab};
use crate::handlers::ssr::types::SvgLineChartView;
use crate::repositories::analysis::skills::MarketplaceAdoptionRow;

pub(super) struct TabInputs<'a> {
    pub(super) query: &'a SkillsQuery,
    pub(super) tab: SkillsTab,
    pub(super) read: &'a SkillsRead<'a>,
    pub(super) scoped_adoption: &'a [&'a MarketplaceAdoptionRow],
    pub(super) marketplace: Option<&'a str>,
    pub(super) today: NaiveDate,
}

#[derive(Default)]
pub(super) struct ActiveTabBody {
    pub(super) groups: Vec<MarketplaceGroupView>,
    pub(super) unused: Vec<UnusedSkillView>,
    pub(super) kpis: Vec<SkillsKpiView>,
    pub(super) chart: Option<SvgLineChartView>,
    pub(super) top_skills: Vec<TopSkillView>,
}

pub(super) fn active_tab_body(input: &TabInputs<'_>) -> ActiveTabBody {
    let SkillsRead {
        rows,
        audience,
        index,
        ..
    } = *input.read;
    match input.tab {
        SkillsTab::Overview => ActiveTabBody {
            kpis: kpis(
                rows,
                input.scoped_adoption,
                audience,
                input.marketplace,
                input.today,
            ),
            ..ActiveTabBody::default()
        },
        SkillsTab::Activity => ActiveTabBody {
            chart: Some(chart(rows, input.today, input.query.days())),
            top_skills: top_skills(input.query, rows, index),
            ..ActiveTabBody::default()
        },
        SkillsTab::Skills => {
            let views: Vec<SkillFactView> = rows
                .iter()
                .map(|r| skill_row_view(r, audience, index, input.today))
                .collect();
            ActiveTabBody {
                groups: group_rows(views, index),
                unused: unused(rows, index, audience, input.marketplace),
                ..ActiveTabBody::default()
            }
        },
    }
}
