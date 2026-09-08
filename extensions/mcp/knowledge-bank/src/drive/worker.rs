//! Source-wide single-flight worker and replay-safe Drive discovery.

use super::aws::AwsExtraction;
use super::config::DriveSource;
use super::google::GoogleDrive;
use super::store;
use super::types::{DriveFile, FOLDER_MIME, SHORTCUT_MIME, assign_project, has_removed_ancestor};
use crate::error::KnowledgeBankError;
use crate::ingest::Ingestor;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use systemprompt::config::SecretsBootstrap;

pub fn spawn(ingestor: Arc<Ingestor>) {
    if !ingestor.config().google_drive.enabled {
        return;
    }
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            for (id, source) in &ingestor.config().google_drive.sources {
                if let Err(error) = run_source(&ingestor, id, source).await {
                    tracing::error!(source = id, error = %error, "Drive worker failed");
                    if let Err(record_error) =
                        store::source_error(ingestor.pool(), id, &error.to_string()).await
                    {
                        tracing::error!(source = id, error = %record_error, "Could not record Drive failure");
                    }
                }
            }
        }
    });
}

pub async fn run_source(
    ingestor: &Ingestor,
    id: &str,
    source: &DriveSource,
) -> Result<(), KnowledgeBankError> {
    let pool = ingestor.pool();
    let mut lock = pool.begin().await?;
    let acquired = sqlx::query_scalar!(
        "SELECT pg_try_advisory_xact_lock(hashtextextended($1, 39177)) AS \"locked!\"",
        source.drive_id
    )
    .fetch_one(&mut *lock)
    .await?;
    if !acquired {
        return Ok(());
    }
    let Some(state) = store::state(pool, id).await? else {
        return Ok(());
    };
    if state.drive_id != source.drive_id {
        return Err(KnowledgeBankError::Config(
            "Drive source configuration differs from its persisted identity".to_owned(),
        ));
    }
    let google = authenticated_client(source)?;
    if state.requested > state.completed {
        google.verify(&source.drive_id).await?;
        let mut token = state.page_token.clone();
        if token.is_none()
            || state.full_inventory
            || state.force_requested
            || state.rebuild_requested
        {
            token = Some(inventory(pool, id, source, &google).await?);
        }
        if let Some(token) = token {
            match changes(pool, id, source, &google, token).await {
                Err(KnowledgeBankError::Http(e))
                    if e.status().is_some_and(|s| s.as_u16() == 410) =>
                {
                    let token = inventory(pool, id, source, &google).await?;
                    changes(pool, id, source, &google, token).await?;
                },
                result => result?,
            }
        }
        update_projects(pool, id, source).await?;
        store::complete_listing(pool, id, state.requested, state.rebuild_requested).await?;
    }
    let aws = AwsExtraction {
        settings: ingestor.config().google_drive.extraction.clone(),
    };
    let version = format!(
        "{}:{}:{}",
        aws.settings.version, aws.settings.project_arn, aws.settings.region
    );
    store::invalidate_extractor(pool, id, &version).await?;
    for pending in store::pending(pool, id).await? {
        match super::process::process(ingestor, id, &pending, &google, &aws).await {
            Ok(()) => {},
            Err(error) => {
                tracing::warn!(source = id, file_id = %pending.file_id, error = %error, "Drive file processing failed");
                store::set_status(
                    pool,
                    id,
                    pending.file_id.as_str(),
                    "failed",
                    Some(&error.to_string()),
                )
                .await?;
            },
        }
    }
    lock.commit().await?;
    Ok(())
}

async fn inventory(
    pool: &sqlx::PgPool,
    id: &str,
    source: &DriveSource,
    google: &GoogleDrive,
) -> Result<String, KnowledgeBankError> {
    let token = google.start_token(&source.drive_id).await?;
    let generation = uuid::Uuid::new_v4().to_string();
    let mut page = None;
    loop {
        let result = google.files(&source.drive_id, page.as_deref()).await?;
        if result.incomplete_search {
            return Err(KnowledgeBankError::Backend(
                "Drive inventory was incomplete; pruning is suspended".to_owned(),
            ));
        }
        for file in result.files {
            store::save_file(pool, id, &file, Some(&generation)).await?;
        }
        match result.next_page_token {
            Some(next) => page = Some(next),
            None => break,
        }
    }
    store::finish_inventory(pool, id, &generation, &token).await?;
    Ok(token)
}

async fn changes(
    pool: &sqlx::PgPool,
    id: &str,
    source: &DriveSource,
    google: &GoogleDrive,
    mut token: String,
) -> Result<(), KnowledgeBankError> {
    loop {
        let result = google.changes(&source.drive_id, &token).await?;
        for change in result.changes {
            if let Some(file_id) = change.file_id {
                if change.removed {
                    store::mark_removed(pool, id, file_id.as_str()).await?;
                } else if let Some(file) = change.file {
                    if file.mime_type == FOLDER_MIME {
                        store::request_inventory(pool, id).await?;
                    }
                    if file.trashed
                        || file
                            .drive_id
                            .as_deref()
                            .is_some_and(|d| d != source.drive_id)
                    {
                        store::mark_removed(pool, id, file_id.as_str()).await?;
                    } else {
                        store::save_file(pool, id, &file, None).await?;
                    }
                }
            }
        }
        if let Some(next) = result.next_page_token {
            store::cursor(pool, id, &next).await?;
            token = next;
        } else if let Some(next) = result.new_start_page_token {
            store::cursor(pool, id, &next).await?;
            break;
        } else {
            return Err(KnowledgeBankError::Backend(
                "Drive change page omitted its cursor".to_owned(),
            ));
        }
    }
    Ok(())
}

async fn update_projects(
    pool: &sqlx::PgPool,
    id: &str,
    source: &DriveSource,
) -> Result<(), KnowledgeBankError> {
    let files: HashMap<String, DriveFile> = store::files(pool, id)
        .await?
        .into_iter()
        .map(|f| (f.id.clone(), f))
        .collect();
    let removed: std::collections::HashSet<String> =
        store::removed_ids(pool, id).await?.into_iter().collect();
    for file in files.values() {
        if has_removed_ancestor(file, &files, &removed) {
            store::mark_removed(pool, id, &file.id).await?;
            continue;
        }
        let project = assign_project(file, &files, source);
        store::assign(pool, id, &file.id, &project).await?;
        if file.mime_type == FOLDER_MIME {
            store::set_status(pool, id, &file.id, "folder", None).await?;
        } else if file.mime_type == SHORTCUT_MIME {
            let in_drive = file
                .shortcut_details
                .as_ref()
                .is_some_and(|s| files.contains_key(&s.target_id));
            store::set_status(
                pool,
                id,
                &file.id,
                if in_drive { "shortcut" } else { "unsupported" },
                if in_drive {
                    None
                } else {
                    Some("Shortcut target is outside this drive or inaccessible")
                },
            )
            .await?;
        }
    }
    Ok(())
}

fn authenticated_client(source: &DriveSource) -> Result<GoogleDrive, KnowledgeBankError> {
    let secrets = SecretsBootstrap::require().map_err(|_error| {
        KnowledgeBankError::NotConfigured("Profile secrets are unavailable".to_owned())
    })?;
    let credential = secrets.get(&source.credential_secret).ok_or_else(|| {
        KnowledgeBankError::NotConfigured(format!(
            "Configure service-account secret {} and add its email to the shared drive",
            source.credential_secret
        ))
    })?;
    GoogleDrive::new(credential)
}
