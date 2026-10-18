//! The workspace files an eval reads, kept in core's file storage rather than
//! on the disk of the runner that executed the run.
//!
//! A run executes on whichever node claimed it, but its eval is judged by the
//! node that launched the eval, so assertions and the judge cannot read the
//! run's workspace directly. When a farm run that scores an eval case turns
//! terminal, the runner node calls [`snapshot`]: every file the case's
//! assertions name, plus the agent's declared output file, is read from the
//! workspace and stored under `farm/runs/<farm run id>/files/<path>`.
//! [`fetch`] reads the same names back into a [`RunFiles`] on any node, and
//! the assertions and the judge read only that. A file the run did not write
//! is simply absent.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path};

use futures_util::TryStreamExt;
use systemprompt::traits::{FileStorage, FileStorageError, StoredFileId};

use crate::error::{FarmError, FarmResult};
use crate::services::farm_sync::eval_yaml::Assertion;
use crate::types::FarmRunId;

pub const MAX_FILE_BYTES: u64 = 1024 * 1024;

/// The files of one finished run that its eval may read, keyed by their path
/// relative to the run's workspace.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RunFiles(BTreeMap<String, String>);

impl RunFiles {
    #[must_use]
    pub const fn new(files: BTreeMap<String, String>) -> Self {
        Self(files)
    }

    #[must_use]
    pub fn get(&self, path: &str) -> Option<&str> {
        self.0.get(&normalise(path)?).map(String::as_str)
    }
}

#[must_use]
pub fn normalise(path: &str) -> Option<String> {
    let rel = Path::new(path);
    let clean = rel
        .components()
        .all(|c| matches!(c, Component::Normal(_) | Component::CurDir));
    let parts: Vec<String> = rel
        .components()
        .filter_map(|c| match c {
            Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect();
    (clean && !parts.is_empty()).then(|| parts.join("/"))
}

#[must_use]
pub fn referenced_paths(assertions: &[Assertion], output_path: Option<&str>) -> BTreeSet<String> {
    assertions
        .iter()
        .filter_map(|a| match a {
            Assertion::FileExists { path } => Some(path.as_str()),
            Assertion::OutputContains { path, .. } | Assertion::Regex { path, .. } => {
                path.as_deref()
            },
            _ => None,
        })
        .chain(output_path)
        .filter_map(normalise)
        .collect()
}

fn object(run_id: &FarmRunId, path: &str) -> StoredFileId {
    StoredFileId::new(format!("farm/runs/{}/files/{path}", run_id.as_str()))
}

pub async fn snapshot(
    files: &dyn FileStorage,
    run_id: &FarmRunId,
    workspace: &Path,
    paths: &BTreeSet<String>,
) -> FarmResult<usize> {
    let mut stored = 0;
    for path in paths {
        let full = workspace.join(path);
        let Ok(meta) = tokio::fs::metadata(&full).await else {
            continue;
        };
        if !meta.is_file() || meta.len() > MAX_FILE_BYTES {
            continue;
        }
        let body = tokio::fs::read(&full).await?;
        files
            .store(Path::new(object(run_id, path).as_str()), &body)
            .await
            .map_err(|source| FarmError::Storage {
                object: path.clone(),
                source,
            })?;
        stored += 1;
    }
    Ok(stored)
}

pub async fn fetch(
    files: &dyn FileStorage,
    run_id: &FarmRunId,
    paths: &BTreeSet<String>,
) -> FarmResult<RunFiles> {
    let mut found = BTreeMap::new();
    for path in paths {
        let read = match files.open(&object(run_id, path), None).await {
            Ok(read) => {
                read.body
                    .try_fold(Vec::new(), |mut body, chunk| async move {
                        body.extend_from_slice(&chunk);
                        Ok(body)
                    })
                    .await
            },
            Err(error) => Err(error),
        };
        match read {
            Ok(body) => {
                found.insert(path.clone(), String::from_utf8_lossy(&body).into_owned());
            },
            Err(FileStorageError::NotFound(_)) => {},
            Err(source) => {
                return Err(FarmError::Storage {
                    object: path.clone(),
                    source,
                });
            },
        }
    }
    Ok(RunFiles::new(found))
}
