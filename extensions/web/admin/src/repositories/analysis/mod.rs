//! Analysis repositories: every figure is read from retained reporting data
//! (feedback snapshots, snapshot contributions, normalized facts); nothing is
//! computed live over raw events.
pub mod inventory_index;
pub mod marketplace_versions;
pub mod portfolio_snapshots;
pub mod publications;
pub mod retained;
pub mod snapshot_figures;
