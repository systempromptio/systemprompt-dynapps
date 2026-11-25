//! Permission levels, subjects, grants and the inheritance rule.
//!
//! The rule, as dynbrain applies it:
//!
//! - an article with grants of its own is reached by exactly those grants;
//! - an article without grants takes its parent's grants that inherit;
//! - a grant with `inherit = false` reaches its article only, never the
//!   descendants;
//! - an article that ends up with no grant is reached by nobody but the space's
//!   creator and administrators, who reach every article.
//!
//! Nothing is granted by default. [`closure`] resolves the rule into the rows
//! of `dynbrain_access`.

use std::collections::{BTreeMap, HashMap, HashSet};

use serde::{Deserialize, Serialize};
use systemprompt::identifiers::UserId;

use crate::ids::ArticleId;

pub const ADMIN_ROLE: &str = "admin";

/// What a subject may do with an article. Ordered: `Manage` covers `Write`,
/// which covers `Read`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Read,
    Write,
    Manage,
}

impl Level {
    pub const fn as_db(self) -> i16 {
        match self {
            Self::Read => 1,
            Self::Write => 2,
            Self::Manage => 3,
        }
    }

    pub const fn from_db(value: i16) -> Option<Self> {
        match value {
            1 => Some(Self::Read),
            2 => Some(Self::Write),
            3 => Some(Self::Manage),
            _ => None,
        }
    }
}

/// Who a grant names: Governance's own users, roles and groups.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SubjectKind {
    User,
    Role,
    Group,
}

impl SubjectKind {
    pub const fn as_db(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Role => "role",
            Self::Group => "group",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "user" => Some(Self::User),
            "role" => Some(Self::Role),
            "group" => Some(Self::Group),
            _ => None,
        }
    }
}

/// A user id, a role name or a group id, with its kind.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Subject {
    pub kind: SubjectKind,
    pub id: String,
}

impl Subject {
    pub fn user(user: &UserId) -> Self {
        Self {
            kind: SubjectKind::User,
            id: user.as_str().to_owned(),
        }
    }

    pub fn role(role: &str) -> Self {
        Self {
            kind: SubjectKind::Role,
            id: role.to_owned(),
        }
    }

    pub fn group(group: &str) -> Self {
        Self {
            kind: SubjectKind::Group,
            id: group.to_owned(),
        }
    }

    pub fn is_valid(&self) -> bool {
        (1..=255).contains(&self.id.chars().count())
    }
}

/// A grant written on an article.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grant {
    pub article: ArticleId,
    pub subject: Subject,
    pub level: Level,
    pub inherit: bool,
}

/// One row of the access closure: `subject` reaches `article` at `level`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessEntry {
    pub article: ArticleId,
    pub subject: Subject,
    pub level: Level,
}

/// An article and its parent: the shape of a space's tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeNode {
    pub id: ArticleId,
    pub parent: Option<ArticleId>,
}

#[derive(Clone)]
struct Reach {
    subject: Subject,
    level: Level,
    inherit: bool,
}

pub fn closure(nodes: &[TreeNode], grants: &[Grant], owner: Option<&UserId>) -> Vec<AccessEntry> {
    let own = grants_by_article(grants);
    let children = children_by_parent(nodes);
    let known: HashSet<ArticleId> = nodes.iter().map(|node| node.id).collect();
    let mut pending: Vec<(ArticleId, Vec<Reach>)> = nodes
        .iter()
        .filter(|node| node.parent.is_none_or(|parent| !known.contains(&parent)))
        .map(|node| (node.id, Vec::new()))
        .collect();
    let mut resolved: BTreeMap<ArticleId, BTreeMap<Subject, Level>> = BTreeMap::new();
    while let Some((article, inherited)) = pending.pop() {
        let effective = own.get(&article).map_or(inherited, Clone::clone);
        let levels = resolved.entry(article).or_default();
        for reach in &effective {
            raise(levels, reach.subject.clone(), reach.level);
        }
        if let Some(owner) = owner {
            raise(levels, Subject::user(owner), Level::Manage);
        }
        raise(levels, Subject::role(ADMIN_ROLE), Level::Manage);
        let passed: Vec<Reach> = effective
            .into_iter()
            .filter(|reach| reach.inherit)
            .collect();
        for child in children.get(&article).into_iter().flatten() {
            pending.push((*child, passed.clone()));
        }
    }
    resolved
        .into_iter()
        .flat_map(|(article, levels)| {
            levels.into_iter().map(move |(subject, level)| AccessEntry {
                article,
                subject,
                level,
            })
        })
        .collect()
}

pub fn subtree(nodes: &[TreeNode], root: ArticleId) -> Vec<ArticleId> {
    let children = children_by_parent(nodes);
    let mut found = vec![root];
    let mut seen = HashSet::from([root]);
    let mut next = 0;
    while let Some(article) = found.get(next).copied() {
        for child in children.get(&article).into_iter().flatten() {
            if seen.insert(*child) {
                found.push(*child);
            }
        }
        next += 1;
    }
    found
}

fn raise(levels: &mut BTreeMap<Subject, Level>, subject: Subject, level: Level) {
    let current = levels.entry(subject).or_insert(level);
    *current = (*current).max(level);
}

fn grants_by_article(grants: &[Grant]) -> HashMap<ArticleId, Vec<Reach>> {
    let mut by_article: HashMap<ArticleId, Vec<Reach>> = HashMap::new();
    for grant in grants {
        by_article.entry(grant.article).or_default().push(Reach {
            subject: grant.subject.clone(),
            level: grant.level,
            inherit: grant.inherit,
        });
    }
    by_article
}

fn children_by_parent(nodes: &[TreeNode]) -> HashMap<ArticleId, Vec<ArticleId>> {
    let mut children: HashMap<ArticleId, Vec<ArticleId>> = HashMap::new();
    for node in nodes {
        if let Some(parent) = node.parent {
            children.entry(parent).or_default().push(node.id);
        }
    }
    children
}
