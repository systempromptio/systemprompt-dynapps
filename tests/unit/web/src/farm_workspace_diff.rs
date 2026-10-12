//! Workspace snapshots and the added / modified / deleted diff a run records.

use systemprompt_web_admin_farm::services::runner::diff::{DiffChange, snapshot, workspace_diff};

fn write(root: &std::path::Path, rel: &str, body: &str) {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap_or_else(|err| panic!("{err}"));
    }
    std::fs::write(path, body).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn diff_classifies_added_modified_and_deleted_files() {
    let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
    let root = dir.path();
    write(root, "keep.txt", "same");
    write(root, "change.txt", "before");
    write(root, "gone.txt", "bye");
    let before = snapshot(root);
    assert_eq!(before.len(), 3);

    write(root, "change.txt", "after");
    std::fs::remove_file(root.join("gone.txt")).unwrap_or_else(|err| panic!("{err}"));
    write(root, "out/report.md", "# report");
    let after = snapshot(root);

    let diff = workspace_diff(&before, &after);
    assert_eq!((diff.added, diff.modified, diff.deleted), (1, 1, 1));
    let files: Vec<(&str, DiffChange)> = diff
        .files
        .iter()
        .map(|f| (f.path.as_str(), f.change))
        .collect();
    assert_eq!(
        files,
        vec![
            ("change.txt", DiffChange::Modified),
            ("gone.txt", DiffChange::Deleted),
            ("out/report.md", DiffChange::Added),
        ]
    );
}

#[test]
fn snapshot_skips_git_and_node_modules() {
    let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
    let root = dir.path();
    write(root, ".git/HEAD", "ref");
    write(root, "node_modules/pkg/index.js", "x");
    write(root, "src/main.rs", "fn main() {}");
    let paths: Vec<String> = snapshot(root).into_iter().map(|s| s.path).collect();
    assert_eq!(paths, vec!["src/main.rs".to_owned()]);
}

#[test]
fn unchanged_workspace_has_an_empty_diff() {
    let dir = tempfile::tempdir().unwrap_or_else(|err| panic!("{err}"));
    write(dir.path(), "a.txt", "a");
    let snap = snapshot(dir.path());
    let diff = workspace_diff(&snap, &snap);
    assert!(diff.files.is_empty());
    assert_eq!(snap[0].bytes, 1);
    assert_eq!(snap[0].sha256.len(), 64);
}
