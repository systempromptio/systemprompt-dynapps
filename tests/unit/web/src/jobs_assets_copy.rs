//! `copy_extension_assets` is the last step between a file in `storage/` and
//! the bytes the browser gets, and its required/optional split is what stops a
//! half-published site: a missing *required* asset aborts the whole copy, a
//! missing optional one is counted and skipped. Nested destinations must also
//! have their directories created — `web/dist/` starts empty on a fresh
//! deploy, so a copy that assumed the parent existed would fail every time.

use systemprompt::extension::{AssetDefinition, AssetType};
use systemprompt_web_extension::jobs::internals::{copy_all_assets, copy_asset};
use tempfile::TempDir;

#[tokio::test]
async fn copying_creates_missing_parent_directories() {
    let src = TempDir::new().unwrap();
    let dist = TempDir::new().unwrap();
    let source = src.path().join("admin-bundle.css");
    std::fs::write(&source, "body{}").unwrap();

    let asset = AssetDefinition::css(&source, "css/admin/admin-bundle.css");
    copy_asset(dist.path(), "web", &asset).await.unwrap();

    let copied = dist.path().join("css/admin/admin-bundle.css");
    assert_eq!(std::fs::read_to_string(copied).unwrap(), "body{}");
}

#[tokio::test]
async fn copying_a_missing_source_is_an_error() {
    let src = TempDir::new().unwrap();
    let dist = TempDir::new().unwrap();
    let asset = AssetDefinition::js(src.path().join("absent.js"), "js/absent.js");
    assert!(copy_asset(dist.path(), "web", &asset).await.is_err());
}

#[tokio::test]
async fn a_missing_optional_asset_is_counted_and_the_rest_still_copy() {
    let src = TempDir::new().unwrap();
    let dist = TempDir::new().unwrap();
    let present = src.path().join("present.css");
    std::fs::write(&present, "a{}").unwrap();

    let assets = vec![
        (
            "web",
            AssetDefinition::builder(
                src.path().join("absent.css"),
                "css/absent.css",
                AssetType::Css,
            )
            .optional()
            .build(),
        ),
        ("web", AssetDefinition::css(&present, "css/present.css")),
    ];

    let (copied, failed) = copy_all_assets(dist.path(), assets).await.unwrap();
    assert_eq!((copied, failed), (1, 1));
    assert!(dist.path().join("css/present.css").exists());
}

#[tokio::test]
async fn a_missing_required_asset_aborts_the_whole_copy() {
    let src = TempDir::new().unwrap();
    let dist = TempDir::new().unwrap();
    let later = src.path().join("later.css");
    std::fs::write(&later, "a{}").unwrap();

    let assets = vec![
        (
            "web",
            AssetDefinition::css(src.path().join("absent.css"), "css/absent.css"),
        ),
        ("web", AssetDefinition::css(&later, "css/later.css")),
    ];

    assert!(copy_all_assets(dist.path(), assets).await.is_err());
    assert!(!dist.path().join("css/later.css").exists());
}

#[tokio::test]
async fn an_empty_asset_list_copies_nothing_successfully() {
    let dist = TempDir::new().unwrap();
    let (copied, failed) = copy_all_assets(dist.path(), Vec::new()).await.unwrap();
    assert_eq!((copied, failed), (0, 0));
}

#[test]
fn module_versions_cover_absolute_relative_side_effect_and_multiline_imports() {
    use systemprompt_web_extension::jobs::internals::version_module_imports;
    let source = "import { errorMessage } from '/js/services/api.js';\nimport './widget.js';\nimport {\n  init\n} from '../services/events.js';\nconst text = \"from './unchanged.js'\";\n";
    let versioned = version_module_imports(source, "release123");
    assert!(versioned.contains("'/js/services/api.js?v=release123'"));
    assert!(versioned.contains("'./widget.js?v=release123'"));
    assert!(versioned.contains("'../services/events.js?v=release123'"));
    assert!(versioned.contains("from './unchanged.js'"));
}

#[tokio::test]
async fn changing_a_dependency_invalidates_every_published_importer() {
    let src = TempDir::new().unwrap();
    let dist = TempDir::new().unwrap();
    let page = src.path().join("page.js");
    let api = src.path().join("api.js");
    std::fs::write(&page, "import { call } from './api.js';").unwrap();
    std::fs::write(&api, "export const call = 1;").unwrap();
    let assets = || {
        vec![
            ("web", AssetDefinition::js(&page, "js/page.js")),
            ("web", AssetDefinition::js(&api, "js/api.js")),
        ]
    };
    copy_all_assets(dist.path(), assets()).await.unwrap();
    let first = std::fs::read_to_string(dist.path().join("js/page.js")).unwrap();
    std::fs::write(&api, "export const call = 2;").unwrap();
    copy_all_assets(dist.path(), assets()).await.unwrap();
    let second = std::fs::read_to_string(dist.path().join("js/page.js")).unwrap();
    assert_ne!(first, second);
    copy_all_assets(dist.path(), assets()).await.unwrap();
    assert_eq!(
        second,
        std::fs::read_to_string(dist.path().join("js/page.js")).unwrap()
    );
}
