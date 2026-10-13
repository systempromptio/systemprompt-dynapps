//! Capturing a farm's directory from its Git source at a ref. Production
//! uses core's sandboxed `git` capture; tests hand in a tree directly.

use std::collections::BTreeMap;
use std::path::PathBuf;

use systemprompt::marketplace::managed::{
    GitCaptureRequest, GitSourceCapture, NativeGitSourceCapture,
};

use crate::error::{FarmError, FarmResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedFile {
    pub bytes: Vec<u8>,
    pub executable: bool,
}

/// The farm directory at one commit, keyed by path relative to that directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedFarm {
    pub commit: String,
    pub files: BTreeMap<String, CapturedFile>,
}

pub trait FarmSourceCapture: Send + Sync + std::fmt::Debug {
    fn capture(
        &self,
        repo_https_url: &str,
        reference: &str,
        subdir: &str,
    ) -> FarmResult<CapturedFarm>;
}

#[must_use]
pub fn github_https_url(owner_repo: &str) -> String {
    format!("https://github.com/{owner_repo}.git")
}

/// Core's capture: resolves the ref, fetches that commit into a private
/// scratch directory under `scratch_root`, and reads the subtree's blobs
/// (bounded at 256 files and a 120 s deadline).
#[derive(Debug, Clone)]
pub struct NativeCapture {
    inner: NativeGitSourceCapture,
}

impl NativeCapture {
    #[must_use]
    pub const fn new(scratch_root: PathBuf) -> Self {
        Self {
            inner: NativeGitSourceCapture::new(scratch_root),
        }
    }
}

impl FarmSourceCapture for NativeCapture {
    fn capture(
        &self,
        repo_https_url: &str,
        reference: &str,
        subdir: &str,
    ) -> FarmResult<CapturedFarm> {
        if !repo_https_url.starts_with("https://") {
            return Err(FarmError::Git(format!(
                "{repo_https_url:?}: only https sources are captured"
            )));
        }
        let request = GitCaptureRequest {
            repository: repo_https_url,
            reference,
            subdirectory: None,
            root: subdir.trim_matches('/'),
            credential: None,
        };
        let captured = self
            .inner
            .capture(&request)
            .map_err(|source| FarmError::GitCapture {
                source_ref: format!("{repo_https_url}@{reference}"),
                source,
            })?;
        let files = captured
            .files
            .0
            .into_iter()
            .map(|(path, file)| {
                (
                    path,
                    CapturedFile {
                        bytes: file.bytes,
                        executable: file.executable,
                    },
                )
            })
            .collect();
        Ok(CapturedFarm {
            commit: captured.commit,
            files,
        })
    }
}

/// A fixed tree, for tests and for syncing from a local checkout.
#[derive(Debug, Clone)]
pub struct InMemoryCapture {
    pub farm: CapturedFarm,
}

impl InMemoryCapture {
    #[must_use]
    pub const fn new(farm: CapturedFarm) -> Self {
        Self { farm }
    }
}

impl FarmSourceCapture for InMemoryCapture {
    fn capture(&self, _repo: &str, _reference: &str, _subdir: &str) -> FarmResult<CapturedFarm> {
        Ok(self.farm.clone())
    }
}
