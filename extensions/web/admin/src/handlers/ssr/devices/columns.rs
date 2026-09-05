//! The sortable column set of each tab, and what each column is for.
//!
//! A column is declared once, here, and the same declaration renders the
//! header, builds its sort link and whitelists its key. That is the point:
//! a sort key the reader cannot see in a header cannot be one they clicked,
//! so the list a query parameter is matched against is the list the page
//! actually drew.

use super::view::ColumnSpec;

pub(super) const BRIDGE_COLUMNS: &[ColumnSpec] = &[
    ColumnSpec {
        key: "host",
        label: "Host",
        class: "sp-col-host",
        hint: "The machine name the bridge reported at start-up",
    },
    ColumnSpec {
        key: "version",
        label: "Version",
        class: "sp-col-version",
        hint: "Bridge build; sort to find the machines still behind a rollout",
    },
    ColumnSpec {
        key: "started",
        label: "Started",
        class: "sp-table__cell--date",
        hint: "When this session first connected",
    },
    ColumnSpec {
        key: "heartbeat",
        label: "Last heartbeat",
        class: "sp-table__cell--date",
        hint: "A bridge silent for over seven days is reported stale",
    },
    ColumnSpec {
        key: "forwarded",
        label: "Forwarded",
        class: "sp-table__cell--num",
        hint: "Requests this session has relayed to the gateway",
    },
    ColumnSpec {
        key: "tokens",
        label: "Tokens",
        class: "sp-table__cell--num",
        hint: "Input plus output tokens across the session's lifetime",
    },
];

pub(super) const PAT_COLUMNS: &[ColumnSpec] = &[
    ColumnSpec {
        key: "name",
        label: "Token",
        class: "sp-col-token",
        hint: "The name given at issue, and the prefix that identifies it",
    },
    ColumnSpec {
        key: "created",
        label: "Issued",
        class: "sp-table__cell--date",
        hint: "When the secret was minted; it was shown exactly once",
    },
    ColumnSpec {
        key: "used",
        label: "Last used",
        class: "sp-table__cell--date",
        hint: "A token never used is a token nobody will miss",
    },
    ColumnSpec {
        key: "expires",
        label: "Expires",
        class: "sp-table__cell--date",
        hint: "Tokens issued without an expiry never lapse on their own",
    },
];

pub(super) const CERT_COLUMNS: &[ColumnSpec] = &[
    ColumnSpec {
        key: "label",
        label: "Device",
        class: "sp-col-host",
        hint: "The label the machine enrolled under",
    },
    ColumnSpec {
        key: "enrolled",
        label: "Enrolled",
        class: "sp-table__cell--date",
        hint: "When this certificate was issued to the machine",
    },
];

pub(super) const LINK_COLUMNS: &[ColumnSpec] = &[
    ColumnSpec {
        key: "created",
        label: "Issued",
        class: "sp-table__cell--date",
        hint: "When the connect code was handed out",
    },
    ColumnSpec {
        key: "expires",
        label: "Expires",
        class: "sp-table__cell--date",
        hint: "Connect codes live ten minutes and are single use",
    },
];
