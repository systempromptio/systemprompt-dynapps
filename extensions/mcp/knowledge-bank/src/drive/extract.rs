//! Resumable extraction, including Office rendering and multimodal BDA jobs.

use super::aws::AwsExtraction;
use super::google::{GoogleDrive, export_format, io_error};
use super::office;
use super::store::{self, PendingFile};
use super::types::DriveFile;
use crate::error::KnowledgeBankError;
use crate::ingest::document::content_hash;
use sqlx::PgPool;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub struct ExtractionContext<'a> {
    pub pool: &'a PgPool,
    pub source: &'a str,
    pub google: &'a GoogleDrive,
    pub aws: &'a AwsExtraction,
}

pub async fn extract(
    context: &ExtractionContext<'_>,
    file: &DriveFile,
    pending: &PendingFile,
) -> Result<Option<String>, KnowledgeBankError> {
    let ExtractionContext {
        pool,
        source,
        google,
        aws,
    } = *context;
    if let Some(arn) = &pending.invocation_arn {
        return poll(context, file, pending, arn).await;
    }
    if let Some(text) = &pending.extracted_text {
        return Ok(Some(text.clone()));
    }
    let extension = export_format(&file.mime_type).map_or_else(
        || crate::ingest::parsers::suffix_of(&file.name),
        |(_, ext)| ext.to_owned(),
    );
    if !supported(&extension) {
        return Err(KnowledgeBankError::InvalidInput(format!(
            "Unsupported format {} (.{extension}); a conversion is required",
            file.mime_type
        )));
    }
    let temp = tempfile::tempdir().map_err(|error| io_error(&error))?;
    let path = temp.path().join(format!("input.{extension}"));
    google
        .download(file, &path, aws.settings.max_download_bytes)
        .await?;
    if matches!(
        extension.as_str(),
        "md" | "txt" | "csv" | "json" | "html" | "xml"
    ) {
        let body = read_text(&path, &extension).await?;
        let key = content_hash(&body);
        store::extracted(pool, source, &file.id, &key, &body).await?;
        return Ok(Some(body));
    }
    let (input, local_text) = prepare_input(&path, temp.path(), &extension).await?;
    let bytes_hash = format!("{}:{}", file.version_key(), pending.extraction_attempt);
    let fingerprint = format!(
        "{source}:{}:{}:{}:{}",
        file.id, bytes_hash, aws.settings.version, aws.settings.project_arn
    );
    let key = format!(
        "{}.{}",
        content_hash(&fingerprint),
        input.extension().and_then(|e| e.to_str()).unwrap_or("bin")
    );
    let (arn, output) = aws.start(&input, &key).await?;
    store::checkpoint(
        pool,
        source,
        &file.id,
        store::Checkpoint {
            key: &key,
            arn: &arn,
            output: &output,
            local_text: &local_text,
        },
    )
    .await?;
    Ok(None)
}

pub fn supported(extension: &str) -> bool {
    matches!(
        extension,
        "md" | "txt"
            | "csv"
            | "json"
            | "html"
            | "xml"
            | "pdf"
            | "docx"
            | "tiff"
            | "tif"
            | "jpg"
            | "jpeg"
            | "png"
            | "xlsx"
            | "xls"
            | "ods"
            | "pptx"
            | "ppt"
            | "odp"
            | "mp4"
            | "mov"
            | "avi"
            | "mkv"
            | "webm"
            | "amr"
            | "flac"
            | "m4a"
            | "mp3"
            | "ogg"
            | "wav"
    )
}

async fn render_pdf(input: &Path, root: &Path) -> Result<std::path::PathBuf, KnowledgeBankError> {
    let output = root.join("rendered");
    tokio::fs::create_dir_all(&output)
        .await
        .map_err(|error| io_error(&error))?;
    let profile = format!(
        "-env:UserInstallation=file://{}",
        root.join("office-profile").display()
    );
    let run = tokio::process::Command::new("libreoffice")
        .env_clear()
        .env("PATH", std::env::var_os("PATH").unwrap_or_default())
        .env("LANG", "C.UTF-8")
        .args(["--headless", "--convert-to", "pdf", "--outdir"])
        .arg(&output)
        .arg(profile)
        .arg(input)
        .kill_on_drop(true)
        .output();
    let result = tokio::time::timeout(Duration::from_mins(3), run)
        .await
        .map_err(|_error| KnowledgeBankError::Backend("Office rendering timed out".to_owned()))?
        .map_err(|error| io_error(&error))?;
    let pdf = output.join("input.pdf");
    if !result.status.success() || !pdf.is_file() {
        return Err(KnowledgeBankError::InvalidInput(
            "Office rendering failed; check encryption and supported format".to_owned(),
        ));
    }
    Ok(pdf)
}

async fn poll(
    context: &ExtractionContext<'_>,
    file: &DriveFile,
    pending: &PendingFile,
    arn: &str,
) -> Result<Option<String>, KnowledgeBankError> {
    let ExtractionContext {
        pool, source, aws, ..
    } = *context;

    let result = aws.status(arn).await?;
    match result.status.as_str() {
        "Created" | "InProgress" => {
            store::set_status(pool, source, &file.id, "processing", None).await?;
            Ok(None)
        },
        "Success" => {
            let uri = result
                .output_configuration
                .as_ref()
                .map(|o| o.s3_uri.as_str())
                .or(pending.output_uri.as_deref())
                .ok_or_else(|| {
                    KnowledgeBankError::Backend(
                        "BDA completed without an output location".to_owned(),
                    )
                })?;
            let body = format!(
                "{}\n{}",
                pending.extracted_text.as_deref().unwrap_or(""),
                aws.result(uri).await?
            );
            store::extracted(
                pool,
                source,
                &file.id,
                pending.extraction_key.as_deref().unwrap_or(""),
                &body,
            )
            .await?;
            Ok(Some(body))
        },
        _ => {
            store::retry_extraction(pool, source, &file.id).await?;
            Err(KnowledgeBankError::Backend(format!(
                "BDA {}: {}",
                result.status,
                result.error_type.as_deref().unwrap_or("processing failed")
            )))
        },
    }
}


async fn read_text(path: &Path, extension: &str) -> Result<String, KnowledgeBankError> {
    let body = tokio::fs::read_to_string(path)
        .await
        .map_err(|error| io_error(&error))?;
    if extension == "html" {
        htmd::convert(&body).map_err(|e| KnowledgeBankError::InvalidInput(e.to_string()))
    } else {
        Ok(body)
    }
}

async fn prepare_input(
    path: &Path,
    root: &Path,
    extension: &str,
) -> Result<(std::path::PathBuf, String), KnowledgeBankError> {
    let mut local_text = String::new();
    let input = if matches!(extension, "xlsx" | "xls" | "ods" | "pptx" | "ppt" | "odp") {
        if matches!(extension, "xlsx" | "xls" | "ods") {
            local_text = office::workbook(path)?;
        } else if extension == "pptx" {
            local_text = office::presentation(
                &tokio::fs::read(path)
                    .await
                    .map_err(|error| io_error(&error))?,
            )?;
        }
        render_pdf(path, root).await?
    } else {
        path.to_path_buf()
    };
    Ok((input, local_text))
}
