//! The two enumerated narrowings of the conversations page: which judge
//! verdict a row must carry, and which dimension the breakdown groups by.

/// Narrowing by the judge's verdict on the session behind a conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoredFilter {
    Scored,
    Unscored,
    LowQuality,
}

impl ScoredFilter {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Scored => "scored",
            Self::Unscored => "unscored",
            Self::LowQuality => "low",
        }
    }

    #[must_use]
    pub fn parse_scored_filter(value: Option<&str>) -> Option<Self> {
        match value {
            Some("scored") => Some(Self::Scored),
            Some("unscored") => Some(Self::Unscored),
            Some("low") => Some(Self::LowQuality),
            _ => None,
        }
    }
}

/// The dimension the breakdown table groups the filtered set by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BreakdownBy {
    #[default]
    Category,
    Group,
    Project,
    User,
    Outcome,
}

impl BreakdownBy {
    pub const ALL: [Self; 5] = [
        Self::Category,
        Self::Group,
        Self::Project,
        Self::User,
        Self::Outcome,
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Category => "category",
            Self::Group => "group",
            Self::Project => "project",
            Self::User => "user",
            Self::Outcome => "outcome",
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Category => "By category",
            Self::Group => "By group",
            Self::Project => "By project",
            Self::User => "By person",
            Self::Outcome => "By outcome",
        }
    }

    #[must_use]
    pub fn from_breakdown_param(value: Option<&str>) -> Self {
        match value {
            Some("group") => Self::Group,
            Some("project") => Self::Project,
            Some("user") => Self::User,
            Some("outcome") => Self::Outcome,
            _ => Self::Category,
        }
    }
}
