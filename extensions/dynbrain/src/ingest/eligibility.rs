//! Which articles are indexed. Denied by default.
//!
//! An article's own retrieval mode decides first: `always` indexes it and
//! `never` keeps it out. With `default`, the first active rule that matches,
//! in rule order, decides: `include` indexes and `exclude` does not. A rule
//! matches every article (`all`), the articles of a space, or an article and
//! everything below it. When no rule matches, the article is not indexed:
//! nobody ruled on it, and a corpus a model answers from must fail closed.

use crate::ids::{ArticleId, SpaceId};
use crate::model::RagMode;

/// What a rule applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleScope {
    All,
    Space(SpaceId),
    Article(ArticleId),
}

/// One eligibility rule, already in evaluation order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RagRule {
    pub scope: RuleScope,
    pub include: bool,
}

/// Where an article sits: its space and its lineage, the article first and
/// its ancestors after it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placement {
    pub space: SpaceId,
    pub lineage: Vec<ArticleId>,
}

impl RagRule {
    pub fn matches(&self, placement: &Placement) -> bool {
        match self.scope {
            RuleScope::All => true,
            RuleScope::Space(space) => placement.space == space,
            RuleScope::Article(article) => placement.lineage.contains(&article),
        }
    }
}

pub fn eligible(mode: RagMode, rules: &[RagRule], placement: &Placement) -> bool {
    match mode {
        RagMode::Always => true,
        RagMode::Never => false,
        RagMode::Default => rules
            .iter()
            .find(|rule| rule.matches(placement))
            .is_some_and(|rule| rule.include),
    }
}
