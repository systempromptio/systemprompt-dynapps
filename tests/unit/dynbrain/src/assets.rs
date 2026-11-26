//! The files of articles without a database: the type of a file read from
//! its bytes, what is shown inline, the `asset://` references of a body and
//! the text the indexer reads, and signed links.

use chrono::{Duration, Utc};
use dynbrain::assets::DEFAULT_MAX_UPLOAD_BYTES;
use dynbrain::assets::link::{LinkPurpose, LinkSigner, SignedLink};
use dynbrain::assets::markdown::{index_text, reference, references};
use dynbrain::assets::sniff::{detect, is_inline, refine};
use dynbrain::ids::{AssetId, UploadId};

#[test]
fn the_type_of_a_file_comes_from_its_bytes() {
    let cases: [(&[u8], &str); 10] = [
        (b"\x89PNG\r\n\x1a\nrest", "image/png"),
        (b"\xff\xd8\xff\xe0rest", "image/jpeg"),
        (b"GIF89arest", "image/gif"),
        (b"RIFF\x00\x00\x00\x00WEBPVP8 ", "image/webp"),
        (b"%PDF-1.7", "application/pdf"),
        (b"PK\x03\x04rest", "application/zip"),
        (
            b"<?xml version=\"1.0\"?>\n<SVG xmlns=\"x\"></SVG>",
            "image/svg+xml",
        ),
        (b"<!DOCTYPE HTML><p>hi", "text/html"),
        (b"just words, \xc3\xa1 too", "text/plain"),
        (b"\x00\x01\xff\xfe binary", "application/octet-stream"),
    ];
    for (bytes, expected) in cases {
        assert_eq!(detect(bytes), expected, "{bytes:?}");
    }
}

#[test]
fn the_name_narrows_a_type_only_within_its_family() {
    assert_eq!(
        refine("application/zip", "budget.XLSX"),
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
    );
    assert_eq!(refine("text/plain", "rows.csv"), "text/csv");
    assert_eq!(refine("text/plain", "notes.md"), "text/markdown");
    assert_eq!(refine("text/plain", "photo.png"), "text/plain");
    assert_eq!(refine("image/svg+xml", "photo.png"), "image/svg+xml");
    assert_eq!(refine("text/html", "page.jpg"), "text/html");
    assert_eq!(refine("application/zip", "archive"), "application/zip");
}

#[test]
fn only_raster_images_are_shown_inline() {
    for inline in ["image/png", "image/jpeg", "image/gif", "image/webp"] {
        assert!(is_inline(inline), "{inline}");
    }
    for download in [
        "image/svg+xml",
        "text/html",
        "application/pdf",
        "text/plain",
        "application/octet-stream",
    ] {
        assert!(!is_inline(download), "{download}");
    }
}

#[test]
fn references_are_found_once_in_the_order_they_appear() {
    let body = "![a](asset://12) and [b](asset://3)\n\n![again](asset://12) asset://x asset://7.";
    assert_eq!(references(body), [AssetId(12), AssetId(3), AssetId(7)]);
    assert_eq!(reference(AssetId(5)), "asset://5");
    assert!(references("no files here").is_empty());
}

#[test]
fn the_indexed_text_keeps_the_words_and_no_url() {
    let body = "Copias:\n\n![Diagrama de copias](asset://12 \"title\")\n\nVer [la plantilla](asset://3), \
                [el manual](https://example.test/manual) y asset://9 suelto.";
    let indexed = index_text(body);
    assert_eq!(
        indexed,
        "Copias:\n\nDiagrama de copias\n\nVer la plantilla, [el manual](https://example.test/manual) y  suelto."
    );
    assert!(!indexed.contains("asset://"));
    let only_files = index_text("![Esquema](asset://1) [Hoja](asset://2)");
    assert_eq!(only_files, "Esquema Hoja");
    assert!(!only_files.contains("://"), "no URL reaches a chunk");
}

#[test]
fn a_signed_link_holds_for_its_purpose_target_and_time_only() {
    let signer = LinkSigner::new(&[1; 32]).expect("a signer");
    let other = LinkSigner::new(&[2; 32]).expect("a signer");
    let now = Utc::now();
    let link = signer
        .sign(LinkPurpose::Download, "7", now + Duration::minutes(10))
        .expect("sign");
    assert!(signer.verify(LinkPurpose::Download, "7", &link, now));
    assert!(!signer.verify(LinkPurpose::Upload, "7", &link, now));
    assert!(!signer.verify(LinkPurpose::Download, "8", &link, now));
    assert!(!other.verify(LinkPurpose::Download, "7", &link, now));
    assert!(!signer.verify(
        LinkPurpose::Download,
        "7",
        &link,
        now + Duration::minutes(11)
    ));
    let stretched = SignedLink {
        exp: link.exp + 3600,
        sig: link.sig.clone(),
    };
    assert!(!signer.verify(LinkPurpose::Download, "7", &stretched, now));
    assert!(!signer.verify(LinkPurpose::Download, "7", &SignedLink::default(), now));
    assert_eq!(link.query(), format!("exp={}&sig={}", link.exp, link.sig));
    assert_eq!(link.sig.len(), 64);
}

#[test]
fn an_upload_slot_is_random_and_the_default_limit_is_25_mb() {
    let first = UploadId::generate();
    let second = UploadId::generate();
    assert_ne!(first, second);
    assert_eq!(first.as_str().len(), 32);
    assert!(first.as_str().chars().all(|c| c.is_ascii_hexdigit()));
    assert_eq!(DEFAULT_MAX_UPLOAD_BYTES, 25 * 1024 * 1024);
}
