//! IAM-authenticated S3 and Data Automation operations through AWS CLI v2.

use super::config::ExtractionSettings;
use super::google::io_error;
use crate::error::KnowledgeBankError;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;
use tokio::process::Command;

#[derive(Debug, Clone)]
pub struct AwsExtraction {
    pub settings: ExtractionSettings,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Invocation {
    pub invocation_arn: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvocationStatus {
    pub status: String,
    pub output_configuration: Option<S3Location>,
    pub error_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct S3Location {
    pub s3_uri: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Project<'a> {
    data_automation_project_arn: &'a str,
    stage: &'static str,
}

#[derive(Deserialize)]
struct Objects {
    #[serde(rename = "Contents", default)]
    contents: Vec<Object>,
}
#[derive(Deserialize)]
struct Object {
    #[serde(rename = "Key")]
    key: String,
}

impl AwsExtraction {
    async fn execute(&self, args: &[&str]) -> Result<Vec<u8>, KnowledgeBankError> {
        let child = Command::new(&self.settings.aws_executable)
            .args(args)
            .args([
                "--region",
                &self.settings.region,
                "--output",
                "json",
                "--no-cli-pager",
                "--no-cli-auto-prompt",
            ])
            .env("AWS_RETRY_MODE", "standard")
            .env("AWS_MAX_ATTEMPTS", "6")
            .kill_on_drop(true)
            .output();
        let result = tokio::time::timeout(Duration::from_mins(15), child)
            .await
            .map_err(|_error| KnowledgeBankError::Backend("AWS operation timed out".to_owned()))?
            .map_err(|error| io_error(&error))?;
        if !result.status.success() {
            return Err(KnowledgeBankError::Backend(format!(
                "AWS {} failed ({}); check IAM, region, bucket and AWS CLI v2 support",
                args.first().unwrap_or(&"operation"),
                result.status
            )));
        }
        Ok(result.stdout)
    }

    async fn json<T: DeserializeOwned>(&self, args: &[&str]) -> Result<T, KnowledgeBankError> {
        Ok(serde_json::from_slice(&self.execute(args).await?)?)
    }

    pub async fn start(
        &self,
        path: &Path,
        key: &str,
    ) -> Result<(String, String), KnowledgeBankError> {
        let input = format!("s3://{}/knowledge-drive/input/{key}", self.settings.bucket);
        let output = format!(
            "s3://{}/knowledge-drive/output/{key}/",
            self.settings.bucket
        );
        self.execute(&[
            "s3",
            "cp",
            &path.to_string_lossy(),
            &input,
            "--only-show-errors",
        ])
        .await?;
        let input_json = serde_json::to_string(&S3Location { s3_uri: input })?;
        let output_json = serde_json::to_string(&S3Location {
            s3_uri: output.clone(),
        })?;
        let project = serde_json::to_string(&Project {
            data_automation_project_arn: &self.settings.project_arn,
            stage: "LIVE",
        })?;
        let token = crate::ingest::document::content_hash(key);
        let result: Invocation = self
            .json(&[
                "bedrock-data-automation-runtime",
                "invoke-data-automation-async",
                "--input-configuration",
                &input_json,
                "--output-configuration",
                &output_json,
                "--data-automation-configuration",
                &project,
                "--data-automation-profile-arn",
                &self.settings.profile_arn,
                "--client-token",
                &token,
            ])
            .await?;
        Ok((result.invocation_arn, output))
    }

    pub async fn status(&self, arn: &str) -> Result<InvocationStatus, KnowledgeBankError> {
        self.json(&[
            "bedrock-data-automation-runtime",
            "get-data-automation-status",
            "--invocation-arn",
            arn,
        ])
        .await
    }

    pub async fn result(&self, uri: &str) -> Result<String, KnowledgeBankError> {
        let prefix = uri
            .strip_prefix(&format!("s3://{}/", self.settings.bucket))
            .filter(|p| p.starts_with("knowledge-drive/output/"))
            .ok_or_else(|| {
                KnowledgeBankError::Backend("Unexpected BDA output location".to_owned())
            })?;
        let prefix = if Path::new(prefix)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        {
            prefix.rsplit_once('/').map_or(prefix, |(parent, _)| parent)
        } else {
            prefix
        };
        let objects: Objects = self
            .json(&[
                "s3api",
                "list-objects-v2",
                "--bucket",
                &self.settings.bucket,
                "--prefix",
                prefix,
            ])
            .await?;
        let temp = tempfile::tempdir().map_err(|error| io_error(&error))?;
        let mut output = String::new();
        for object in objects.contents {
            if !object.key.contains("standard_output")
                || !Path::new(&object.key)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
            {
                continue;
            }
            let path = temp.path().join("output.json");
            let uri = format!("s3://{}/{}", self.settings.bucket, object.key);
            self.execute(&[
                "s3",
                "cp",
                &uri,
                &path.to_string_lossy(),
                "--only-show-errors",
            ])
            .await?;
            let bytes = tokio::fs::read(&path)
                .await
                .map_err(|error| io_error(&error))?;
            output.push_str(&super::output::normalize(&bytes)?);
        }
        if output.trim().is_empty() {
            return Err(KnowledgeBankError::Backend(
                "BDA returned no usable standard output; inspect output configuration".to_owned(),
            ));
        }
        Ok(output)
    }
}
