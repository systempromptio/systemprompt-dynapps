//! Portable, exact revision files and source provenance are validated at
//! ingress.

use std::collections::BTreeMap;
use systemprompt::identifiers::SourceSnapshotId;
use systemprompt::marketplace::managed::{
    AssetDigest, AssetFile, RevisionFiles, RevisionManifest, SnapshotProvenance, SourceSpec,
};

fn files(path: &str) -> RevisionFiles {
    RevisionFiles(BTreeMap::from([(
        path.to_owned(),
        AssetFile {
            bytes: vec![0, 255, 128],
            media_type: "application/octet-stream".to_owned(),
            executable: false,
        },
    )]))
}

#[test]
fn revision_files_reject_traversal_and_excessive_content() {
    for path in [
        "../SKILL.md",
        "/SKILL.md",
        "a/../b",
        "a\\b",
        "C:/file",
        "a//b",
        "./x",
        "a\nfile",
    ] {
        assert!(files(path).validate().is_err(), "{path:?}");
    }
    assert!(files("assets/example.bin").validate().is_ok());
    let mut large = files("large.bin");
    large.0.get_mut("large.bin").expect("file").bytes = vec![0; 8 * 1024 * 1024 + 1];
    assert!(large.validate().is_err());
}

#[test]
fn manifest_hash_binds_permissions_paths_bytes_and_provenance() {
    let snapshot = SourceSnapshotId::generate();
    let original = files("SKILL.md");
    let first = RevisionManifest::from_files(snapshot.clone(), None, &original, BTreeMap::new())
        .expect("manifest");
    let same = RevisionManifest::from_files(snapshot.clone(), None, &original, BTreeMap::new())
        .expect("manifest");
    assert_eq!(first.digest().expect("hash"), same.digest().expect("hash"));
    let mut changed = original;
    changed.0.get_mut("SKILL.md").expect("file").executable = true;
    let second =
        RevisionManifest::from_files(snapshot, None, &changed, BTreeMap::new()).expect("manifest");
    assert_ne!(
        first.digest().expect("hash"),
        second.digest().expect("hash")
    );
    assert!(serde_json::from_str::<AssetDigest>("\"not-a-digest\"").is_err());
}

#[test]
fn git_snapshots_require_exact_commits_without_embedded_credentials() {
    let source = SourceSpec::Git {
        repository: "https://example.com/skills.git".to_owned(),
        reference: "main".to_owned(),
        subdirectory: None,
        credential_reference: None,
    };
    assert!(source.validate().is_ok());
    let mut snapshot = SnapshotProvenance {
        source_kind: "git".to_owned(),
        commit: Some("main".to_owned()),
        tree_digest: AssetDigest::of(b"tree"),
        importer_version: "v1".to_owned(),
    };
    assert!(snapshot.validate(&source).is_err());
    snapshot.commit = Some("a".repeat(40));
    assert!(snapshot.validate(&source).is_ok());
    let bad = SourceSpec::Git {
        repository: "https://token@example.com/skills.git".to_owned(),
        reference: "main".to_owned(),
        subdirectory: None,
        credential_reference: None,
    };
    assert!(bad.validate().is_err());
}
