//! `fixtures/<name>/`: the workspace seeds a run starts from, grouped out of
//! the captured tree and written under the farm's commit directory.
//! `fixtures/README.md` is ground truth and is never copied.

use std::collections::BTreeMap;
use std::path::Path;

use super::capture::CapturedFile;
use super::validate::{definition_error, is_identifier, is_safe_relative_path};
use crate::error::FarmResult;

pub const FIXTURES_DIR: &str = "fixtures";
const FIXTURES_README: &str = "fixtures/README.md";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureFile {
    pub relpath: String,
    pub bytes: Vec<u8>,
    pub executable: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FixtureManifest {
    pub file_count: usize,
    pub bytes: u64,
}

pub fn group_fixtures(
    files: &BTreeMap<String, CapturedFile>,
) -> FarmResult<BTreeMap<String, Vec<FixtureFile>>> {
    let mut groups: BTreeMap<String, Vec<FixtureFile>> = BTreeMap::new();
    let prefix = format!("{FIXTURES_DIR}/");
    for (path, file) in files {
        let Some(rest) = path.strip_prefix(&prefix) else {
            continue;
        };
        if path == FIXTURES_README {
            continue;
        }
        let Some((name, relpath)) = rest.split_once('/') else {
            return Err(definition_error(
                path,
                "files directly under fixtures/ must live in a fixture directory",
            ));
        };
        if !is_identifier(name) {
            return Err(definition_error(
                path,
                format!("fixture name {name:?} is invalid"),
            ));
        }
        if !is_safe_relative_path(relpath) {
            return Err(definition_error(
                path,
                "fixture path must be relative without `..`",
            ));
        }
        groups
            .entry(name.to_owned())
            .or_default()
            .push(FixtureFile {
                relpath: relpath.to_owned(),
                bytes: file.bytes.clone(),
                executable: file.executable,
            });
    }
    Ok(groups)
}

#[must_use]
pub fn fixture_manifest(files: &[FixtureFile]) -> FixtureManifest {
    FixtureManifest {
        file_count: files.len(),
        bytes: files.iter().map(|f| f.bytes.len() as u64).sum(),
    }
}

pub fn materialise_fixture(dir: &Path, files: &[FixtureFile]) -> FarmResult<FixtureManifest> {
    if dir.exists() {
        std::fs::remove_dir_all(dir)?;
    }
    std::fs::create_dir_all(dir)?;
    for file in files {
        if !is_safe_relative_path(&file.relpath) {
            return Err(definition_error(
                &file.relpath,
                "fixture path must be relative without `..`",
            ));
        }
        let target = dir.join(&file.relpath);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&target, &file.bytes)?;
        set_mode(&target, file.executable)?;
    }
    Ok(fixture_manifest(files))
}

#[cfg(unix)]
fn set_mode(path: &Path, executable: bool) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mode = if executable { 0o755 } else { 0o644 };
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
}

#[cfg(not(unix))]
fn set_mode(_path: &Path, _executable: bool) -> std::io::Result<()> {
    Ok(())
}
