//! Where the farm keeps its files: run directories and synced farm trees
//! under the storage data root, and the plugins Claude Code loads from the
//! composed services tree.

use std::path::{Path, PathBuf};

use systemprompt::identifiers::PluginId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FarmPaths {
    pub data_root: PathBuf,
    pub composed_services_root: PathBuf,
}

impl FarmPaths {
    #[must_use]
    pub const fn new(data_root: PathBuf, composed_services_root: PathBuf) -> Self {
        Self {
            data_root,
            composed_services_root,
        }
    }

    #[must_use]
    pub fn runs_dir(&self) -> PathBuf {
        self.data_root.join("runs")
    }

    #[must_use]
    pub fn run_dir(&self, run_id: &str) -> PathBuf {
        self.runs_dir().join(run_id)
    }

    #[must_use]
    pub fn farm_dir(&self, farm_id: &str, commit: &str) -> PathBuf {
        self.data_root.join("farms").join(farm_id).join(commit)
    }

    #[must_use]
    pub fn fixture_dir(&self, farm_id: &str, commit: &str, name: &str) -> PathBuf {
        self.farm_dir(farm_id, commit).join("fixtures").join(name)
    }

    #[must_use]
    pub fn plugin_dir(&self, plugin_id: &PluginId) -> Option<PathBuf> {
        let direct = self
            .composed_services_root
            .join("plugins")
            .join(plugin_id.as_str());
        if has_manifest(&direct) {
            return Some(direct);
        }
        let marketplaces = self.composed_services_root.join("marketplaces");
        let entries = std::fs::read_dir(marketplaces).ok()?;
        entries
            .filter_map(Result::ok)
            .map(|entry| entry.path().join("plugins").join(plugin_id.as_str()))
            .find(|candidate| has_manifest(candidate))
    }
}

fn has_manifest(dir: &Path) -> bool {
    dir.join(".claude-plugin").join("plugin.json").is_file()
}
