//! The sortable column set of each tab, and what each column is for.
//!
//! A column is declared once, here, and the same declaration renders the
//! header, builds its sort link and whitelists its key. That is the point:
//! a sort key the reader cannot see in a header cannot be one they clicked,
//! so the list a query parameter is matched against is the list the page
//! actually drew. Every column is an aggregate over a person's rows — the
//! newest, the latest, the sum — because the row it sorts is a person.

use crate::handlers::ssr::types::table::SortColumn;

pub(super) const BRIDGE_COLUMNS: &[SortColumn] = &[
    SortColumn {
        key: "version",
        label: "Version",
        class: "sp-col-version",
        hint: "Newest bridge build across this person's machines; open the row for each one",
    },
    SortColumn {
        key: "started",
        label: "First seen",
        class: "sp-table__cell--date",
        hint: "When this person's first bridge connected",
    },
    SortColumn {
        key: "heartbeat",
        label: "Last heartbeat",
        class: "sp-table__cell--date",
        hint: "The most recent heartbeat from any of their machines; online within a minute, stale after a week",
    },
    SortColumn {
        key: "forwarded",
        label: "Forwarded",
        class: "sp-table__cell--num",
        hint: "Requests relayed to the gateway, summed over every machine and session",
    },
    SortColumn {
        key: "tokens",
        label: "Tokens",
        class: "sp-table__cell--num",
        hint: "Input plus output tokens, summed over every machine and session",
    },
];

pub(super) const PAT_COLUMNS: &[SortColumn] = &[
    SortColumn {
        key: "created",
        label: "Newest issued",
        class: "sp-table__cell--date",
        hint: "When this person's most recent token was minted",
    },
    SortColumn {
        key: "used",
        label: "Last used",
        class: "sp-table__cell--date",
        hint: "The last time any of their tokens was presented",
    },
    SortColumn {
        key: "expires",
        label: "Next expiry",
        class: "sp-table__cell--date",
        hint: "The soonest a live token lapses; Never if none of them will",
    },
];

pub(super) const CERT_COLUMNS: &[SortColumn] = &[SortColumn {
    key: "enrolled",
    label: "Latest enrolled",
    class: "sp-table__cell--date",
    hint: "When this person's most recent certificate was issued",
}];

pub(super) const LINK_COLUMNS: &[SortColumn] = &[
    SortColumn {
        key: "created",
        label: "Newest issued",
        class: "sp-table__cell--date",
        hint: "When this person's most recent connect code was handed out",
    },
    SortColumn {
        key: "expires",
        label: "Latest expiry",
        class: "sp-table__cell--date",
        hint: "Connect codes live ten minutes and are single use",
    },
];
