//! The type of a file, read from its bytes.
//!
//! What a client says a file is never decides how it is served. The name
//! only narrows a type within its own family (a ZIP archive named `.xlsx` is
//! a spreadsheet, text named `.csv` is CSV), and never turns text into an
//! image. Only the raster images listed in [`is_inline`] are shown inside a
//! page; everything else, SVG and HTML included, is served as a download.

const OCTET_STREAM: &str = "application/octet-stream";
const ZIP: &str = "application/zip";
const PLAIN_TEXT: &str = "text/plain";
const HEAD_BYTES: usize = 1024;

const SIGNATURES: [(&[u8], &str); 6] = [
    (b"\x89PNG\r\n\x1a\n", "image/png"),
    (b"\xff\xd8\xff", "image/jpeg"),
    (b"GIF87a", "image/gif"),
    (b"GIF89a", "image/gif"),
    (b"%PDF-", "application/pdf"),
    (b"PK\x03\x04", ZIP),
];

pub fn detect(bytes: &[u8]) -> &'static str {
    if let Some(&(_, mime)) = SIGNATURES
        .iter()
        .find(|&&(magic, _)| bytes.starts_with(magic))
    {
        return mime;
    }
    if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && bytes[8..12] == *b"WEBP" {
        return "image/webp";
    }
    if std::str::from_utf8(bytes).is_err() {
        return OCTET_STREAM;
    }
    let head = String::from_utf8_lossy(&bytes[..bytes.len().min(HEAD_BYTES)]).to_ascii_lowercase();
    if head.contains("<svg") {
        return "image/svg+xml";
    }
    if ["<!doctype html", "<html", "<script", "<body", "<iframe"]
        .iter()
        .any(|tag| head.contains(tag))
    {
        return "text/html";
    }
    PLAIN_TEXT
}

pub fn refine(detected: &str, name: &str) -> String {
    let extension = name
        .rsplit_once('.')
        .map_or_else(String::new, |(_, extension)| extension.to_ascii_lowercase());
    let refined = match (detected, extension.as_str()) {
        (ZIP, "xlsx") => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        (ZIP, "docx") => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        (ZIP, "pptx") => {
            "application/vnd.openxmlformats-officedocument.presentationml.presentation"
        },
        (PLAIN_TEXT, "csv") => "text/csv",
        (PLAIN_TEXT, "md") => "text/markdown",
        _ => detected,
    };
    refined.to_owned()
}

pub fn is_inline(mimetype: &str) -> bool {
    matches!(
        mimetype,
        "image/png" | "image/jpeg" | "image/gif" | "image/webp"
    )
}
