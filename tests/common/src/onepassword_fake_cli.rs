//! A fake `op` (1Password CLI) installed into a temp `bin` directory.
//!
//! The binary is `tests/fixtures/onepassword/op.sh`, which answers `op read`
//! and `op item get` from a JSON store this struct writes; see the script's
//! header for the store shape. It needs `python3` on `PATH`, which
//! [`FakeOpCli::available`] reports so a suite can skip through
//! [`crate::skip_or_panic`] rather than fail obscurely.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde_json::{Value, json};

use crate::onepassword_fake::FakeConnect;

#[derive(Debug)]
pub struct FakeOpCli {
    pub bin_dir: PathBuf,
    pub store_path: PathBuf,
    store: Mutex<Value>,
    _dir: tempfile::TempDir,
}

impl FakeOpCli {
    pub fn available() -> bool {
        std::process::Command::new("python3")
            .arg("--version")
            .output()
            .is_ok_and(|o| o.status.success())
    }

    /// Install `op` with an empty store that accepts only `token_expected`.
    pub fn install(token_expected: &str) -> Self {
        let dir = tempfile::tempdir().expect("temp dir");
        let bin_dir = dir.path().join("bin");
        std::fs::create_dir_all(&bin_dir).expect("bin dir");
        let target = bin_dir.join("op");
        std::fs::copy(
            crate::repo_path("tests/fixtures/onepassword/op.sh"),
            &target,
        )
        .expect("copy fake op");
        make_executable(&target);
        let cli = Self {
            bin_dir,
            store_path: dir.path().join("store.json"),
            store: Mutex::new(json!({ "token": token_expected, "fail": null, "vaults": [] })),
            _dir: dir,
        };
        cli.flush();
        cli
    }

    pub fn op_path(&self) -> PathBuf {
        self.bin_dir.join("op")
    }

    /// Mirror every vault and item of `connect` into the CLI's store.
    pub fn write_store(&self, connect: &FakeConnect) {
        self.write_vaults(connect.export()["vaults"].clone());
    }

    /// Replace the vaults with raw store JSON (an array of vault objects).
    pub fn write_vaults(&self, vaults: Value) {
        self.store.lock().expect("store")["vaults"] = vaults;
        self.flush();
    }

    /// Every later invocation (bar `--version`) exits `exit_code` with
    /// `stderr`.
    pub fn set_fail(&self, exit_code: i32, stderr: &str) {
        self.store.lock().expect("store")["fail"] =
            json!({ "exit_code": exit_code, "stderr": stderr });
        self.flush();
    }

    pub fn clear_fail(&self) {
        self.store.lock().expect("store")["fail"] = Value::Null;
        self.flush();
    }

    /// `PATH` with the fake first, and `FAKE_OP_STORE`, for a child process.
    pub fn env(&self) -> Vec<(String, String)> {
        let path = std::env::var("PATH").unwrap_or_default();
        vec![
            (
                "PATH".to_owned(),
                format!("{}:{path}", self.bin_dir.display()),
            ),
            (
                "FAKE_OP_STORE".to_owned(),
                self.store_path.display().to_string(),
            ),
        ]
    }

    fn flush(&self) {
        let body =
            serde_json::to_vec_pretty(&*self.store.lock().expect("store")).expect("store json");
        let tmp = self.store_path.with_extension("tmp");
        std::fs::write(&tmp, body).expect("write store");
        std::fs::rename(&tmp, &self.store_path).expect("replace store");
    }
}

#[cfg(unix)]
fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt as _;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).expect("chmod op");
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) {}
