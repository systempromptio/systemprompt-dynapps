//! Read-only Drive API client with service-account OAuth and bounded downloads.

use super::types::{ChangePage, DriveFile, FilePage};
use crate::error::KnowledgeBankError;
use futures::StreamExt;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;
use tokio::io::AsyncWriteExt;

pub use super::types::export_format;

const FIELDS: &str = "id,name,mimeType,parents,driveId,modifiedTime,version,md5Checksum,size,webViewLink,trashed,shortcutDetails";

#[derive(Deserialize)]
struct ServiceAccount {
    client_email: String,
    private_key: String,
}

#[derive(Serialize)]
struct Claims<'a> {
    iss: &'a str,
    scope: &'static str,
    aud: &'static str,
    iat: i64,
    exp: i64,
}

#[derive(Deserialize)]
struct Token {
    access_token: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StartToken {
    start_page_token: String,
}

pub struct GoogleDrive {
    http: reqwest::Client,
    account: Option<ServiceAccount>,
    endpoint: String,
    token: tokio::sync::Mutex<(String, i64)>,
}

impl std::fmt::Debug for GoogleDrive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GoogleDrive").finish_non_exhaustive()
    }
}

impl GoogleDrive {
    pub fn new(credential: &str) -> Result<Self, KnowledgeBankError> {
        let account = serde_json::from_str(credential)?;
        let http = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(15))
            .timeout(Duration::from_mins(10))
            .redirect(reqwest::redirect::Policy::none())
            .build()?;
        Ok(Self {
            http,
            account: Some(account),
            endpoint: "https://www.googleapis.com/drive/v3".to_owned(),
            token: tokio::sync::Mutex::new((String::new(), 0)),
        })
    }

    #[doc(hidden)]
    pub fn with_access_token(endpoint: String, token: String) -> Result<Self, KnowledgeBankError> {
        let http = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(30))
            .build()?;
        Ok(Self {
            http,
            account: None,
            endpoint,
            token: tokio::sync::Mutex::new((token, i64::MAX)),
        })
    }

    async fn token(&self) -> Result<String, KnowledgeBankError> {
        let mut cached = self.token.lock().await;
        let now = chrono::Utc::now().timestamp();
        if cached.1 > now + 120 {
            return Ok(cached.0.clone());
        }
        let account = self
            .account
            .as_ref()
            .ok_or_else(|| KnowledgeBankError::Config("Access token expired".to_owned()))?;
        let claims = Claims {
            iss: &account.client_email,
            scope: "https://www.googleapis.com/auth/drive.readonly",
            aud: "https://oauth2.googleapis.com/token",
            iat: now,
            exp: now + 3600,
        };
        let key = EncodingKey::from_rsa_pem(account.private_key.as_bytes()).map_err(|_error| {
            KnowledgeBankError::Config("Invalid Google service-account private key".to_owned())
        })?;
        let assertion = jsonwebtoken::encode(&Header::new(Algorithm::RS256), &claims, &key)
            .map_err(|_error| {
                KnowledgeBankError::Config(
                    "Could not sign Google service-account assertion".to_owned(),
                )
            })?;
        let token: Token = self
            .http
            .post(claims.aud)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", assertion.as_str()),
            ])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        *cached = (token.access_token.clone(), now + 3600);
        drop(cached);
        Ok(token.access_token)
    }

    async fn response(
        &self,
        path: &str,
        query: &[(String, String)],
    ) -> Result<reqwest::Response, KnowledgeBankError> {
        for attempt in 0..6u32 {
            let response = self
                .http
                .get(format!("{}/{path}", self.endpoint))
                .bearer_auth(self.token().await?)
                .query(query)
                .send()
                .await?;
            let status = response.status();
            if (status.as_u16() == 429 || status.is_server_error()) && attempt < 5 {
                let retry = response
                    .headers()
                    .get("retry-after")
                    .and_then(|h| h.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(1 << attempt)
                    .min(60);
                tokio::time::sleep(Duration::from_secs(retry)).await;
                continue;
            }
            return Ok(response.error_for_status()?);
        }
        Err(KnowledgeBankError::Backend(
            "Drive retry budget exhausted".to_owned(),
        ))
    }

    pub(super) async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        query: Vec<(String, String)>,
    ) -> Result<T, KnowledgeBankError> {
        Ok(self.response(path, &query).await?.json().await?)
    }

    pub async fn verify(&self, drive: &str) -> Result<(), KnowledgeBankError> {
        #[derive(Deserialize)]
        struct Drive {
            id: String,
        }
        let result: Drive = self
            .get(&format!("drives/{}", urlencoding::encode(drive)), vec![])
            .await?;
        if result.id != drive {
            return Err(KnowledgeBankError::Config(
                "Unexpected shared drive".to_owned(),
            ));
        }
        Ok(())
    }

    pub async fn start_token(&self, drive: &str) -> Result<String, KnowledgeBankError> {
        let token: StartToken = self
            .get(
                "changes/startPageToken",
                pairs(&[("driveId", drive), ("supportsAllDrives", "true")]),
            )
            .await?;
        Ok(token.start_page_token)
    }

    pub async fn files(
        &self,
        drive: &str,
        page: Option<&str>,
    ) -> Result<FilePage, KnowledgeBankError> {
        self.get(
            "files",
            pairs(&[
                ("driveId", drive),
                ("corpora", "drive"),
                ("includeItemsFromAllDrives", "true"),
                ("supportsAllDrives", "true"),
                ("pageSize", "1000"),
                ("q", "trashed = false"),
                (
                    "fields",
                    &format!("nextPageToken,incompleteSearch,files({FIELDS})"),
                ),
                ("pageToken", page.unwrap_or("")),
            ]),
        )
        .await
    }

    pub async fn changes(
        &self,
        drive: &str,
        token: &str,
    ) -> Result<ChangePage, KnowledgeBankError> {
        self.get(
            "changes",
            pairs(&[
                ("driveId", drive),
                ("supportsAllDrives", "true"),
                ("includeItemsFromAllDrives", "true"),
                ("includeRemoved", "true"),
                ("pageToken", token),
                ("pageSize", "1000"),
                (
                    "fields",
                    &format!(
                        "nextPageToken,newStartPageToken,changes(fileId,removed,file({FIELDS}))"
                    ),
                ),
            ]),
        )
        .await
    }

    pub async fn download(
        &self,
        file: &DriveFile,
        path: &Path,
        limit: u64,
    ) -> Result<(), KnowledgeBankError> {
        if file
            .size
            .as_deref()
            .and_then(|n| n.parse::<u64>().ok())
            .is_some_and(|n| n > limit)
        {
            return Err(KnowledgeBankError::InvalidInput(
                "File exceeds configured download limit".to_owned(),
            ));
        }
        let id = urlencoding::encode(&file.id);
        let (route, query) = match export_format(&file.mime_type) {
            Some((mime, _)) => (format!("files/{id}/export"), pairs(&[("mimeType", mime)])),
            None => (
                format!("files/{id}"),
                pairs(&[("alt", "media"), ("supportsAllDrives", "true")]),
            ),
        };
        let mut chunks = self.response(&route, &query).await?.bytes_stream();
        let mut output = tokio::fs::File::create(path)
            .await
            .map_err(|error| io_error(&error))?;
        let mut written = 0u64;
        while let Some(chunk) = chunks.next().await {
            let chunk = chunk?;
            written = written.saturating_add(chunk.len() as u64);
            if written > limit {
                return Err(KnowledgeBankError::InvalidInput(
                    "Download exceeds configured limit".to_owned(),
                ));
            }
            output
                .write_all(&chunk)
                .await
                .map_err(|error| io_error(&error))?;
        }
        output.flush().await.map_err(|error| io_error(&error))?;
        Ok(())
    }
}

pub(super) fn pairs(values: &[(&str, &str)]) -> Vec<(String, String)> {
    values
        .iter()
        .filter(|(_, v)| !v.is_empty())
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect()
}

pub(super) fn io_error(error: &std::io::Error) -> KnowledgeBankError {
    KnowledgeBankError::Internal(error.to_string())
}
