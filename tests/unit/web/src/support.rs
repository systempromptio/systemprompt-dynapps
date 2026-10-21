//! Helpers shared by the front-end standards gates.

use std::path::{Path, PathBuf};

use systemprompt::governance::policy::RateWindowRepository;

// The repository root, found by climbing from this crate until the directory
// holding the front-end sources these gates read appears.
//
// Why the search rather than a fixed number of `pop()`s: this crate's depth
// below the root is not a fact the gates should depend on. A hard-coded depth
// that goes stale resolves to a directory that simply has no templates or
// assets in it, and every gate then walks an empty tree and passes — the one
// failure mode a gate must never have.
pub(crate) fn repo_root() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        if dir.join("storage/files").is_dir() && dir.join("scripts").is_dir() {
            return dir;
        }
        assert!(
            dir.pop(),
            "repository root not found above CARGO_MANIFEST_DIR"
        );
    }
}

// Why: returning quietly on an unreadable directory hands every caller an
// empty corpus to find zero violations in, so the panic names the path.
pub(crate) fn walk(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("gate cannot read {}: {e}", dir.display()));
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, ext, out);
        } else if path.extension().is_some_and(|e| e == ext) {
            out.push(path);
        }
    }
}

// Why: core refuses a chain whose `rate_limit` window it cannot count. The
// chain-only gates here evaluate with stated `RateCounts` and never charge, so
// a pool that never connects satisfies the check without a database.
pub(crate) fn uncharged_rate_windows() -> RateWindowRepository {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .idle_timeout(None)
        .max_lifetime(None)
        .connect_lazy("postgres://unused:unused@127.0.0.1:1/unused")
        .expect("a lazy pool never connects");
    RateWindowRepository::new(pool)
}
