//! Permission levels, subjects, grants and the inheritance rule.
//!
//! A grant names a subject and a level, on a space or on one of its articles.
//! The access of an article is resolved top-down:
//!
//! - it starts from what its parent (its space, for a top-level article)
//!   resolves to, unless the article has `inherit_access` off, in which case it
//!   starts empty;
//! - its own grants are added on top, and an own grant for a subject the
//!   article inherited replaces the inherited level, lower or higher;
//! - an own `none` grant removes the subject, here and below, until a
//!   descendant grants it again;
//! - the space's creator and the `admin` role manage every article, whatever
//!   the grants say.
//!
//! Nothing is granted by default. [`closure`] resolves the rule into the rows
//! of `dynbrain_access` (ordered by article, then subject), where `none` never
//! appears. [`inherited`] is what an article would take from above, the
//! grants alone, without the owner and the administrators. [`subtree`] is an
//! article and everything below it, root first.

use std::collections::{BTreeMap, HashMap, HashSet};

use serde::{Deserialize, Serialize};
use systemprompt::identifiers::UserId;

use crate::ids::{ArticleId, SpaceId};

pub const ADMIN_ROLE: &str = "admin";

/// What a subject may do with an article. Ordered: `Manage` covers `Write`,
/// which covers `Read`; `NoAccess` (`none`) only takes inherited access away.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    #[serde(rename = "none")]
    NoAccess,
    Read,
    Write,
    Manage,
}

impl Level {
    pub const fn as_db(self) -> i16 {
        match self {
            Self::NoAccess => 0,
            Self::Read => 1,
            Self::Write => 2,
            Self::Manage => 3,
        }
    }

    pub const fn from_db(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::NoAccess),
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

/// A grant written on a space (`article: None`) or on one of its articles.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grant {
    pub space: SpaceId,
    pub article: Option<ArticleId>,
    pub subject: Subject,
    pub level: Level,
}

/// One row of the access closure: `subject` reaches `article` at `level`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessEntry {
    pub article: ArticleId,
    pub subject: Subject,
    pub level: Level,
}

/// An article, its parent and whether it inherits: the shape of a space's
/// tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeNode {
    pub id: ArticleId,
    pub parent: Option<ArticleId>,
    pub inherit_access: bool,
}

type Levels = BTreeMap<Subject, Level>;

pub fn closure(nodes: &[TreeNode], grants: &[Grant], owner: Option<&UserId>) -> Vec<AccessEntry> {
    resolve(nodes, grants)
        .into_iter()
        .flat_map(|(article, mut levels)| {
            if let Some(owner) = owner {
                levels.insert(Subject::user(owner), Level::Manage);
            }
            levels.insert(Subject::role(ADMIN_ROLE), Level::Manage);
            levels.into_iter().map(move |(subject, level)| AccessEntry {
                article,
                subject,
                level,
            })
        })
        .collect()
}

pub fn inherited(
    nodes: &[TreeNode],
    grants: &[Grant],
    article: ArticleId,
) -> Vec<(Subject, Level)> {
    let parent = nodes
        .iter()
        .find(|node| node.id == article)
        .and_then(|node| node.parent);
    let source = parent.map_or_else(
        || space_levels(grants),
        |parent| resolve(nodes, grants).remove(&parent).unwrap_or_default(),
    );
    source.into_iter().collect()
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

fn resolve(nodes: &[TreeNode], grants: &[Grant]) -> BTreeMap<ArticleId, Levels> {
    let own = article_grants(grants);
    let children = children_by_parent(nodes);
    let known: HashSet<ArticleId> = nodes.iter().map(|node| node.id).collect();
    let from_space = space_levels(grants);
    let mut pending: Vec<(TreeNode, Levels)> = nodes
        .iter()
        .filter(|node| node.parent.is_none_or(|parent| !known.contains(&parent)))
        .map(|node| (*node, from_space.clone()))
        .collect();
    let by_id: HashMap<ArticleId, TreeNode> = nodes.iter().map(|node| (node.id, *node)).collect();
    let mut resolved = BTreeMap::new();
    while let Some((node, above)) = pending.pop() {
        let mut levels = if node.inherit_access {
            above
        } else {
            Levels::new()
        };
        for grant in own.get(&node.id).into_iter().flatten() {
            apply(&mut levels, grant);
        }
        for child in children.get(&node.id).into_iter().flatten() {
            if let Some(child) = by_id.get(child) {
                pending.push((*child, levels.clone()));
            }
        }
        resolved.insert(node.id, levels);
    }
    resolved
}

fn space_levels(grants: &[Grant]) -> Levels {
    let mut levels = Levels::new();
    for grant in grants.iter().filter(|grant| grant.article.is_none()) {
        apply(&mut levels, grant);
    }
    levels
}

fn apply(levels: &mut Levels, grant: &Grant) {
    if grant.level == Level::NoAccess {
        levels.remove(&grant.subject);
    } else {
        levels.insert(grant.subject.clone(), grant.level);
    }
}

fn article_grants(grants: &[Grant]) -> HashMap<ArticleId, Vec<&Grant>> {
    let mut by_article: HashMap<ArticleId, Vec<&Grant>> = HashMap::new();
    for grant in grants {
        if let Some(article) = grant.article {
            by_article.entry(article).or_default().push(grant);
        }
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
