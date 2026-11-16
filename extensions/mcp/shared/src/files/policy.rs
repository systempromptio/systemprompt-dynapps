//! [`FilePolicy`]: the limits one server applies to the files its tools
//! move, and the checks on names, MIME types and inline bodies that need no
//! store.
//!
//! The MIME grammar is the one core's upload service admits (a
//! `type/subtype` of RFC 6838 token characters), so a type this layer
//! passes is never refused by core for its shape; core still sniffs the
//! bytes against it.

use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use super::error::FilesError;

pub const DEFAULT_INLINE_MAX_BYTES: u64 = 64 * 1024;
pub const DEFAULT_MAX_FILE_BYTES: u64 = 25 * 1024 * 1024;
pub const DEFAULT_REF_TTL_HOURS: i64 = 24;
pub const OCTET_STREAM: &str = "application/octet-stream";
const MAX_NAME_BYTES: usize = 255;
const DOWNLOAD_FALLBACK_NAME: &str = "download";

/// The file limits of one MCP server.
///
/// `upload_tool` is the name of the server's own upload tool, which the
/// errors point the model to. `allowed_mime` empty admits every well-formed
/// type; an entry `type/*` admits every subtype of `type`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilePolicy {
    pub upload_tool: String,
    pub inline_max_bytes: u64,
    pub max_file_bytes: u64,
    pub ref_ttl: chrono::Duration,
    pub allowed_mime: Vec<String>,
}

impl FilePolicy {
    #[must_use]
    pub fn new(upload_tool: impl Into<String>) -> Self {
        Self {
            upload_tool: upload_tool.into(),
            inline_max_bytes: DEFAULT_INLINE_MAX_BYTES,
            max_file_bytes: DEFAULT_MAX_FILE_BYTES,
            ref_ttl: chrono::Duration::hours(DEFAULT_REF_TTL_HOURS),
            allowed_mime: Vec::new(),
        }
    }

    pub fn mime(&self, given: &str) -> Result<String, FilesError> {
        let mime = normalize_mime(given).ok_or_else(|| FilesError::InvalidMime {
            given: given.to_owned(),
        })?;
        self.admit_mime(mime)
    }

    pub fn download_mime(&self, given: &str) -> Result<String, FilesError> {
        let mime = normalize_mime(given).unwrap_or_else(|| OCTET_STREAM.to_owned());
        self.admit_mime(mime)
    }

    #[must_use]
    pub fn allows_mime(&self, mime: &str) -> bool {
        self.allowed_mime.is_empty()
            || self.allowed_mime.iter().any(|entry| {
                let entry = entry.trim().to_ascii_lowercase();
                entry.strip_suffix("/*").map_or(entry == mime, |kind| {
                    mime.split_once('/').is_some_and(|(given, _)| given == kind)
                })
            })
    }

    fn admit_mime(&self, mime: String) -> Result<String, FilesError> {
        if self.allows_mime(&mime) {
            Ok(mime)
        } else {
            Err(FilesError::MimeNotAllowed {
                mime,
                accepted: self.allowed_mime.join(", "),
            })
        }
    }

    pub const fn check_size(&self, size: u64) -> Result<(), FilesError> {
        if size == 0 {
            return Err(FilesError::Empty);
        }
        if size > self.max_file_bytes {
            return Err(FilesError::TooLarge {
                size,
                limit: self.max_file_bytes,
            });
        }
        Ok(())
    }

    pub fn decode_inline(&self, encoded: &str) -> Result<Vec<u8>, FilesError> {
        let compact: String = encoded
            .chars()
            .filter(|c| !c.is_ascii_whitespace())
            .collect();
        let estimated = (compact.len() as u64).saturating_mul(3) / 4;
        let padding_allowance = 2;
        if estimated > self.inline_max_bytes.saturating_add(padding_allowance) {
            return Err(self.inline_too_large(estimated));
        }
        let bytes = STANDARD
            .decode(compact.as_bytes())
            .map_err(|_undecodable| FilesError::InvalidBase64)?;
        let size = bytes.len() as u64;
        if size > self.inline_max_bytes {
            return Err(self.inline_too_large(size));
        }
        if bytes.is_empty() {
            return Err(FilesError::Empty);
        }
        Ok(bytes)
    }

    fn inline_too_large(&self, size: u64) -> FilesError {
        FilesError::InlineTooLarge {
            size,
            limit: self.inline_max_bytes,
            upload_tool: self.upload_tool.clone(),
        }
    }
}

#[must_use]
pub fn normalize_mime(given: &str) -> Option<String> {
    let mime = given
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    let token = |part: &str| {
        !part.is_empty()
            && part
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"!#$&-^_.+".contains(&b))
    };
    let well_formed = mime
        .split_once('/')
        .is_some_and(|(kind, subtype)| token(kind) && token(subtype));
    well_formed.then_some(mime)
}

pub fn check_name(given: &str) -> Result<String, FilesError> {
    let name = given.trim();
    if name.is_empty() {
        return Err(FilesError::InvalidName("name must not be empty".to_owned()));
    }
    if name.contains(['/', '\\']) || name.chars().any(char::is_control) {
        return Err(FilesError::InvalidName(
            "name must be a file name, without path separators or control characters".to_owned(),
        ));
    }
    if name.len() > MAX_NAME_BYTES || name == "." || name == ".." {
        return Err(FilesError::InvalidName(format!(
            "name must be a file name of at most {MAX_NAME_BYTES} bytes"
        )));
    }
    Ok(name.to_owned())
}

#[must_use]
pub fn download_name(given: &str) -> String {
    let leaf: String = given
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or_default()
        .chars()
        .filter(|c| !c.is_control())
        .collect();
    let leaf = leaf.trim();
    let mut name = if leaf.is_empty() || leaf == "." || leaf == ".." {
        DOWNLOAD_FALLBACK_NAME.to_owned()
    } else {
        leaf.to_owned()
    };
    while name.len() > MAX_NAME_BYTES {
        name.pop();
    }
    name
}

#[must_use]
pub fn numbered_name(name: &str, n: u32) -> String {
    tagged_name(name, &n.to_string())
}

#[must_use]
pub fn tagged_name(name: &str, tag: &str) -> String {
    let suffix = format!(" ({tag})");
    let (stem, extension) = match name.rsplit_once('.') {
        Some((stem, extension)) if !stem.is_empty() && !extension.is_empty() => {
            (stem, format!(".{extension}"))
        },
        _ => (name, String::new()),
    };
    let room = MAX_NAME_BYTES.saturating_sub(suffix.len() + extension.len());
    let mut stem = stem.to_owned();
    while stem.len() > room {
        stem.pop();
    }
    format!("{stem}{suffix}{extension}")
}
