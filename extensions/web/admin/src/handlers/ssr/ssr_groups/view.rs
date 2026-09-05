//! The group listing's header tiles.

use super::super::types::{GroupRowView, StatTileView};
use super::UNASSIGNED_GROUP;

// Why: Why: the Groups count excludes `unassigned`. It is a derived bucket
// rather than a group anyone created, and counting it would make an estate with
// no groups at all report one.
pub(super) fn group_stats(rows: &[GroupRowView]) -> Vec<StatTileView> {
    let real: Vec<&GroupRowView> = rows.iter().filter(|r| !r.is_unassigned).collect();
    let members: i64 = real.iter().map(|r| r.member_count).sum();
    let unassigned = rows
        .iter()
        .find(|r| r.id == UNASSIGNED_GROUP)
        .map_or(0, |r| r.member_count);

    vec![
        StatTileView {
            label: "Groups",
            value: real.len().to_string(),
        },
        StatTileView {
            label: "Members",
            value: members.to_string(),
        },
        StatTileView {
            label: "Unassigned",
            value: unassigned.to_string(),
        },
    ]
}
