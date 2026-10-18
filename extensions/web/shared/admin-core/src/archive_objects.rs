//! The object names the retention archives are stored under in core's file
//! storage, shared by the jobs that write them and the console route that
//! serves them.

pub const ARCHIVE_PREFIX: &str = "exports";

#[must_use]
pub fn archive_object(tier: &str, period: &str, file: &str) -> String {
    format!("{ARCHIVE_PREFIX}/{tier}/{period}/{file}")
}
