//! Workspace snapshots taken before and after a run, and the diff between
//! them that the run row and the Artifacts tab show.

use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

const SKIPPED_DIRS: &[&str] = &[".git", "node_modules"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileStat {
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffChange {
    Added,
    Modified,
    Deleted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffFile {
    pub path: String,
    pub change: DiffChange,
    pub bytes: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceDiff {
    pub added: usize,
    pub modified: usize,
    pub deleted: usize,
    pub files: Vec<DiffFile>,
}

#[must_use]
pub fn snapshot(dir: &Path) -> Vec<FileStat> {
    let mut stats: Vec<FileStat> = WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| {
            !(entry.file_type().is_dir()
                && SKIPPED_DIRS.contains(&entry.file_name().to_string_lossy().as_ref()))
        })
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| {
            let relative = entry.path().strip_prefix(dir).ok()?;
            let (bytes, sha256) = hash_file(entry.path())?;
            Some(FileStat {
                path: relative.to_string_lossy().replace('\\', "/"),
                bytes,
                sha256,
            })
        })
        .collect();
    stats.sort_by(|a, b| a.path.cmp(&b.path));
    stats
}

fn hash_file(path: &Path) -> Option<(u64, String)> {
    let mut file = std::fs::File::open(path).ok()?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    let mut total = 0u64;
    loop {
        let read = file.read(&mut buf).ok()?;
        if read == 0 {
            break;
        }
        total += read as u64;
        hasher.update(&buf[..read]);
    }
    Some((total, hex::encode(hasher.finalize())))
}

#[must_use]
pub fn workspace_diff(before: &[FileStat], after: &[FileStat]) -> WorkspaceDiff {
    let old: BTreeMap<&str, &FileStat> = before.iter().map(|s| (s.path.as_str(), s)).collect();
    let new: BTreeMap<&str, &FileStat> = after.iter().map(|s| (s.path.as_str(), s)).collect();
    let mut diff = WorkspaceDiff::default();
    for (path, stat) in &new {
        let change = match old.get(path) {
            None => DiffChange::Added,
            Some(prev) if prev.sha256 != stat.sha256 => DiffChange::Modified,
            Some(_) => continue,
        };
        diff.push(path, change, stat.bytes);
    }
    for (path, stat) in &old {
        if !new.contains_key(path) {
            diff.push(path, DiffChange::Deleted, stat.bytes);
        }
    }
    diff.files.sort_by(|a, b| a.path.cmp(&b.path));
    diff
}

impl WorkspaceDiff {
    fn push(&mut self, path: &str, change: DiffChange, bytes: u64) {
        match change {
            DiffChange::Added => self.added += 1,
            DiffChange::Modified => self.modified += 1,
            DiffChange::Deleted => self.deleted += 1,
        }
        self.files.push(DiffFile {
            path: path.to_owned(),
            change,
            bytes,
        });
    }
}
