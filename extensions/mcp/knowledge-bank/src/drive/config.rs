//! Service-owned Google Drive and AWS extraction configuration.

use crate::error::KnowledgeBankError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DriveSettings {
    pub enabled: bool,
    pub sources: BTreeMap<String, DriveSource>,
    pub extraction: ExtractionSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DriveSource {
    pub drive_id: String,
    pub credential_secret: String,
    #[serde(default = "projects_folder")]
    pub projects_folder: String,
    #[serde(default = "shared_folder")]
    pub shared_folder: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ExtractionSettings {
    pub region: String,
    pub bucket: String,
    pub profile_arn: String,
    pub project_arn: String,
    pub aws_executable: String,
    pub max_download_bytes: u64,
    pub version: String,
}

impl Default for ExtractionSettings {
    fn default() -> Self {
        Self {
            region: "eu-central-1".to_owned(),
            bucket: String::new(),
            profile_arn: String::new(),
            project_arn: String::new(),
            aws_executable: "aws".to_owned(),
            max_download_bytes: 10 * 1024 * 1024 * 1024,
            version: "drive-extraction-v1".to_owned(),
        }
    }
}

impl DriveSettings {
    pub fn validate(&self) -> Result<(), KnowledgeBankError> {
        if !self.enabled {
            return Ok(());
        }
        let e = &self.extraction;
        if !e.region.starts_with("eu-")
            || e.bucket.is_empty()
            || !e.profile_arn.contains(":data-automation-profile/eu.")
            || !e
                .profile_arn
                .starts_with(&format!("arn:aws:bedrock:{}:", e.region))
            || !e
                .project_arn
                .starts_with(&format!("arn:aws:bedrock:{}:", e.region))
            || e.max_download_bytes == 0
        {
            return Err(KnowledgeBankError::Config("Google Drive requires an EU BDA profile, project, S3 bucket and positive download limit".to_owned()));
        }
        for source in self.sources.values() {
            if source.drive_id.is_empty() || source.credential_secret.is_empty() {
                return Err(KnowledgeBankError::Config(
                    "Drive ID and service-account credential secret are required".to_owned(),
                ));
            }
        }
        Ok(())
    }
}

fn projects_folder() -> String {
    "Projects".to_owned()
}
fn shared_folder() -> String {
    "Shared".to_owned()
}
