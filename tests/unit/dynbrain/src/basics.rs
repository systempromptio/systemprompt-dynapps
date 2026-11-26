//! Language codes, the content hash, the extension registration and the
//! vocabulary rule.

use std::fs;
use std::path::{Path, PathBuf};

use dynbrain::DynbrainExtension;
use dynbrain::content::content_hash;
use dynbrain::language::{fts_config, is_language_code};
use systemprompt::extension::Extension;

#[test]
fn language_codes_are_a_base_and_an_optional_region() {
    for code in ["es", "en", "ast", "es_ES", "pt-BR", "zh_Hant"] {
        assert!(is_language_code(code), "{code}");
    }
    for code in ["", "e", "ES", "spanish", "es_", "es_E", "es ES", "es_ES_x"] {
        assert!(!is_language_code(code), "{code}");
    }
}

#[test]
fn the_text_search_configuration_follows_the_base_language() {
    let cases = [
        ("es", "spanish"),
        ("es_ES", "spanish"),
        ("en-GB", "english"),
        ("nl", "dutch"),
        ("fr", "french"),
        ("de", "german"),
        ("pt_BR", "portuguese"),
        ("it", "italian"),
        ("ca", "simple"),
        ("", "simple"),
    ];
    for (code, config) in cases {
        assert_eq!(fts_config(code), config, "{code}");
    }
}

#[test]
fn the_content_hash_covers_title_description_and_body() {
    let base = content_hash("Title", "Description", "Body");
    assert_eq!(base.len(), 64);
    assert_eq!(base, content_hash("Title", "Description", "Body"));
    assert_ne!(base, content_hash("Title", "Description", "Body!"));
    assert_ne!(base, content_hash("Other", "Description", "Body"));
    assert_ne!(
        content_hash("ab", "", ""),
        content_hash("a", "b", ""),
        "moving text between fields changes the hash"
    );
}

#[test]
fn the_extension_owns_a_schema_that_requires_pgvector() {
    let extension = DynbrainExtension::default();
    assert_eq!(extension.metadata().id, "dynbrain");
    assert_eq!(extension.dependencies(), ["users", "web"]);
    let schemas = extension.schemas();
    assert_eq!(schemas.len(), 2);
    let files = &schemas[1].sql;
    for table in ["dynbrain_upload", "dynbrain_asset"] {
        assert!(
            files.contains(&format!("CREATE TABLE IF NOT EXISTS {table} (")),
            "{table}"
        );
    }
    assert!(
        !files.contains("BYTEA"),
        "the bytes live in core's file storage"
    );
    let sql = &schemas[0].sql;
    assert!(sql.contains("CREATE EXTENSION IF NOT EXISTS vector"));
    for table in [
        "dynbrain_space",
        "dynbrain_article",
        "dynbrain_article_variant",
        "dynbrain_article_revision",
        "dynbrain_grant",
        "dynbrain_access",
        "dynbrain_chunk",
        "dynbrain_rag_rule",
        "dynbrain_index_job",
        "dynbrain_query_cache",
    ] {
        assert!(
            sql.contains(&format!("CREATE TABLE IF NOT EXISTS {table} (")),
            "{table}"
        );
    }
    assert!(!sql.contains(" brain_"), "the core owns the `brain_` names");
}

// The crate's directory, found by climbing from this crate until it appears.
fn crate_dir() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        let candidate = dir.join("extensions/dynbrain");
        if candidate.join("Cargo.toml").is_file() {
            return candidate;
        }
        assert!(
            dir.pop(),
            "extensions/dynbrain not found above CARGO_MANIFEST_DIR"
        );
    }
}

fn files_under(dir: &Path) -> Vec<PathBuf> {
    fs::read_dir(dir)
        .expect("read a directory")
        .map(|entry| entry.expect("a directory entry").path())
        .flat_map(|path| {
            if path.is_dir() {
                files_under(&path)
            } else {
                vec![path]
            }
        })
        .collect()
}

#[test]
fn no_file_of_the_brain_names_the_proxy_source() {
    let crate_dir = crate_dir();
    let repo = crate_dir
        .parent()
        .and_then(Path::parent)
        .expect("the repository root")
        .to_path_buf();
    let mut files = files_under(&crate_dir);
    files.extend(files_under(&repo.join("extensions/mcp/brain")));
    files.push(repo.join("services/config/dynbrain.yaml"));
    assert!(files.len() >= 30, "the Brain's files were found");
    let offending: Vec<&PathBuf> = files
        .iter()
        .filter(|path| {
            String::from_utf8_lossy(&fs::read(path).expect("read a file"))
                .to_lowercase()
                .contains("odoo")
        })
        .collect();
    assert!(offending.is_empty(), "{offending:?}");
}
