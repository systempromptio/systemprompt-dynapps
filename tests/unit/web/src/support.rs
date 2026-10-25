//! Helpers shared by the front-end standards gates.

use std::path::{Path, PathBuf};

use systemprompt::governance::policy::RateWindowRepository;
use systemprompt::loader::ConfigLoader;
use systemprompt::manifest::ServicesConfig;
use systemprompt::manifest::services::{GatewayRoute, GatewayState};

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

// The committed `services/` tree, loaded exactly as the server boots it.
pub(crate) fn committed_services() -> ServicesConfig {
    ConfigLoader::load_from_path(&repo_root().join("services/config/config.yaml"))
        .expect("services/ tree loads")
}

// The committed tree with `routes` declared ahead of its own gateway routes.
pub(crate) fn services_with_routes(routes: Vec<GatewayRoute>) -> ServicesConfig {
    let mut services = committed_services();
    let mut gateway = services
        .gateway
        .take()
        .expect("services/ declares a gateway")
        .config()
        .into_owned();
    gateway.routes.splice(0..0, routes);
    services.gateway = Some(GatewayState::Resolved(gateway));
    services
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

// A throwaway git repository holding one gate script and the fixture files a
// test writes, so shell gates that walk `git ls-files` see exactly that tree.
pub(crate) struct GateFixture {
    dir: tempfile::TempDir,
    script: String,
}

impl GateFixture {
    pub(crate) fn new(script: &str) -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let scripts = dir.path().join("scripts");
        std::fs::create_dir_all(&scripts).expect("scripts dir");
        std::fs::copy(
            repo_root().join("scripts").join(script),
            scripts.join(script),
        )
        .expect("copy gate script");
        let fixture = Self {
            dir,
            script: script.to_owned(),
        };
        fixture.git(&["init", "-q"]);
        fixture.git(&["config", "user.email", "gate@test"]);
        fixture.git(&["config", "user.name", "gate"]);
        fixture
    }

    pub(crate) fn root(&self) -> &Path {
        self.dir.path()
    }

    pub(crate) fn write(&self, rel: &str, content: &str) -> &Self {
        let path = self.dir.path().join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("fixture dir");
        }
        std::fs::write(path, content).expect("fixture file");
        self
    }

    pub(crate) fn copy_from_repo(&self, rel: &str) -> &Self {
        let content = std::fs::read_to_string(repo_root().join(rel))
            .unwrap_or_else(|e| panic!("read {rel}: {e}"));
        self.write(rel, &content)
    }

    fn git(&self, args: &[&str]) {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(self.dir.path())
            .output()
            .expect("run git");
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    pub(crate) fn run(&self, env: &[(&str, &str)]) -> GateOutcome {
        self.git(&["add", "-A"]);
        let mut cmd = std::process::Command::new("bash");
        cmd.arg(self.dir.path().join("scripts").join(&self.script))
            .current_dir(self.dir.path());
        for (key, value) in env {
            cmd.env(key, value);
        }
        let out = cmd.output().expect("run gate");
        GateOutcome {
            code: out.status.code(),
            stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        }
    }
}

pub(crate) struct GateOutcome {
    pub(crate) code: Option<i32>,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
}

impl GateOutcome {
    pub(crate) fn passed(&self) -> bool {
        self.code == Some(0)
    }

    pub(crate) fn text(&self) -> String {
        format!("{}{}", self.stdout, self.stderr)
    }
}
