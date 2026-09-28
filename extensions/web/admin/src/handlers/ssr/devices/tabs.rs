//! The four `/admin/devices` tabs: their query slug, labels, sortable
//! columns and default sort. A requested sort key is honoured only if it names
//! one of the tab's own columns — the SQL binds it either way, but a header the
//! reader cannot see cannot be the one they clicked, so anything else falls
//! back to the tab's default.

use super::columns::{BRIDGE_COLUMNS, CERT_COLUMNS, LINK_COLUMNS, PAT_COLUMNS};
use crate::handlers::ssr::types::table::SortColumn;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DeviceTab {
    Bridges,
    Pats,
    Certs,
    Links,
}

impl DeviceTab {
    pub(super) fn parse(raw: Option<&str>) -> Self {
        match raw {
            Some("pats") => Self::Pats,
            Some("certs") => Self::Certs,
            Some("links") => Self::Links,
            _ => Self::Bridges,
        }
    }

    pub(super) const fn slug(self) -> &'static str {
        match self {
            Self::Bridges => "bridges",
            Self::Pats => "pats",
            Self::Certs => "certs",
            Self::Links => "links",
        }
    }

    pub(super) const fn items_label(self) -> &'static str {
        match self {
            Self::Bridges => "Machines",
            Self::Pats => "Tokens",
            Self::Certs => "Certificates",
            Self::Links => "Codes",
        }
    }

    pub(super) const fn empty_message(self) -> &'static str {
        match self {
            Self::Bridges => {
                "No bridge has called home under this filter. Every desktop \
                 client writes a session on its first heartbeat."
            },
            Self::Pats => {
                "No access token matches this filter. Tokens are minted from \
                 the bridge setup page and from the profile page."
            },
            Self::Certs => {
                "No device certificate matches this filter. A certificate is \
                 enrolled when a machine completes the device-link flow."
            },
            Self::Links => {
                "Nothing is waiting to be linked. A connect code appears here \
                 for the ten minutes between issue and enrolment."
            },
        }
    }

    pub(super) const fn columns(self) -> &'static [SortColumn] {
        match self {
            Self::Bridges => BRIDGE_COLUMNS,
            Self::Pats => PAT_COLUMNS,
            Self::Certs => CERT_COLUMNS,
            Self::Links => LINK_COLUMNS,
        }
    }

    const fn default_sort(self) -> &'static str {
        match self {
            Self::Bridges => "heartbeat",
            Self::Pats | Self::Links => "created",
            Self::Certs => "enrolled",
        }
    }
}

pub(super) fn resolve_sort(tab: DeviceTab, raw: Option<&str>) -> &'static str {
    tab.columns()
        .iter()
        .find(|c| Some(c.key) == raw)
        .map_or_else(|| tab.default_sort(), |c| c.key)
}
