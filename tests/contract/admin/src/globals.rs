//! Process-wide globals the admin handlers read, installed once per test
//! binary.
//!
//! `Config`, `ProfileBootstrap`, the secrets bundle, and the JWT signing
//! authority are all `OnceLock`s in core, and `cargo test` runs every test in
//! one process against one set of them. Installing them behind a `Once` keeps
//! the whole binary agreed on a single issuer, which is what makes a token
//! minted in [`crate::principal`] validate inside the handler.
//!
//! The profile is a checked-in fixture rather than the developer's
//! `.systemprompt` profile, which is git-ignored. Depending on the latter
//! would make the suite pass locally and silently skip in CI — the one place
//! the gate has to bite — and would couple the assertions to whatever a given
//! machine happens to have configured.

use std::path::PathBuf;
use std::sync::{Once, OnceLock};

static INIT: Once = Once::new();
static READY: OnceLock<bool> = OnceLock::new();

// Any absolute URL works: it only has to be a well-formed issuer that both
// the minter and the validator agree on.
const ISSUER: &str = "http://localhost:8099";

pub(crate) use astound_test_common::repo_root;

// Install the profile, secrets, config, and an ephemeral signing key.
//
// Returns `false` only if the fixture cannot be materialised; the suite then
// self-skips the same way it does without a database.
pub(crate) fn init() -> bool {
    INIT.call_once(|| {
        let ready = try_init();
        READY
            .set(ready)
            .expect("call_once runs this exactly once, so READY is unset");
    });
    *READY.get().unwrap_or(&false)
}

fn try_init() -> bool {
    let profile = write_fixture_profile();

    systemprompt::config::ProfileBootstrap::init_from_path(&profile)
        .expect("initialise the contract fixture profile");
    // Why: the secrets bootstrap is async since core 0.50 (Vault source), but
    // `init` is a synchronous `Once` guard called from inside `#[tokio::test]`
    // functions. Blocking on the current thread there panics with "cannot start
    // a runtime from within a runtime", so the throwaway runtime is driven on a
    // thread of its own — correct whether or not the caller already has one.
    std::thread::scope(|scope| {
        scope
            .spawn(|| {
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("runtime for the secrets bootstrap")
                    .block_on(systemprompt::config::SecretsBootstrap::try_init())
                    .expect("load the fixture profile's secrets");
            })
            .join()
            .expect("the secrets bootstrap thread did not panic");
    });
    systemprompt::config::try_init_config(None).expect("build config from the fixture profile");
    // The gateway catalog the admin routes serve comes from the services tree the
    // fixture profile points at — the checked-in one, catalog and routes included.
    systemprompt::loader::ServicesBootstrap::try_init()
        .expect("load the services tree the fixture profile points at");

    // The local profile ships no signing key. A key generated per process is
    // enough because the same authority both mints and validates here.
    let key = systemprompt_security::keys::RsaSigningKey::generate()
        .expect("generate an ephemeral RSA signing key");
    systemprompt_security::keys::authority::install_for_test(key);
    true
}

// The checked-in profile, with `__REPO__` resolved to this checkout.
//
// Deliberately a fixture rather than the developer's `.systemprompt` profile,
// which is git-ignored: relying on it would make the suite pass locally and
// silently skip in CI, which is the one place the gate has to bite.
const FIXTURE_PROFILE: &str = include_str!("../fixtures/profile.yaml");

// Minimal secrets bundle. Nothing here is a real credential — the suite talks
// to no provider, and the signing key is generated in-process.
//
// `database_url` is required by the schema but unused: the harness opens its
// own throwaway database in `crate::tempdb` and hands that pool to the
// router, so it is echoed back rather than pointed anywhere live.
//
// `anthropic` is the `api_key_secret` the fixture provider names. The services
// loader demotes a provider with no credential in the store to
// `surface: backend`, and the per-user catalog hides every route on a backend
// provider — so without this entry both fixture routes vanish before any ACL
// rule is consulted, and the catalog cases fail for a reason that has nothing
// to do with access control.
fn fixture_secrets() -> String {
    let database_url = std::env::var("SYSTEMPROMPT_TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .unwrap_or_else(|_| "postgres://unused:unused@localhost:5432/postgres".to_owned());
    format!(
        r#"{{
  "database_url": "{database_url}",
  "oauth_at_rest_pepper": "contract-suite-pepper-not-a-real-secret",
  "mcp_credential_broker_secret": "contract-suite-broker-not-a-real-secret",
  "manifest_signing_secret_seed": "Y29udHJhY3Qtc3VpdGUtc2VlZC1ub3QtcmVhbC0wMDA=",
  "encryption_master_key": "{MASTER_KEY_HEX}",
  "anthropic": "contract-suite-anthropic-key-not-a-real-secret"
}}"#
    )
}

// A fixed 32-byte key, hex-encoded. `secret_crypto::load_master_key` refuses
// anything that is not exactly that, and without it every secrets route
// answers 500 before reaching the code the cases are about.
const MASTER_KEY_HEX: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

// Materialise the fixture profile under `tests/target/`, which is already
// ignored. Secrets live beside the profile because `secrets_path` is relative
// to it, and `paths.bin` is an empty directory beside it too: profile path
// validation requires it to exist, and a runner that has only built this
// workspace has no `target/debug` to borrow.
fn write_fixture_profile() -> PathBuf {
    let root = repo_root();
    // Why: the tree is removed and rewritten on every init, so two contract
    // runs sharing one path (a peer's and yours) race each other into
    // "No such file or directory" mid-copy. A per-process path is private.
    let dir = root.join(format!(
        "tests/target/contract-profile-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(dir.join("bin")).expect("create fixture profile and bin directories");

    let yaml = FIXTURE_PROFILE
        .replace("__PROFILE_DIR__", &dir.to_string_lossy())
        .replace("__REPO__", &root.to_string_lossy())
        .replace(
            "jwt_issuer: http://localhost:8099",
            &format!("jwt_issuer: {ISSUER}"),
        );
    std::fs::write(dir.join("profile.yaml"), yaml).expect("write fixture profile");
    std::fs::write(dir.join("secrets.json"), fixture_secrets()).expect("write fixture secrets");
    write_fixture_services(&root, &dir.join("services"));

    dir.join("profile.yaml")
}

// The provider catalog and gateway routes are services files, not profile
// sections, and the suite needs its own: two routes with fixed ids and no
// default provider, so an unrouted model exists for the ACL detector cases.
// Everything else — MCP servers, skills, plugins, roles — must stay the
// checked-in tree, so it is copied under the fixture and only the two `ai/`
// files are replaced.
const FIXTURE_PROVIDERS: &str = include_str!("../fixtures/providers.yaml");
const FIXTURE_GATEWAY: &str = include_str!("../fixtures/gateway.yaml");

fn write_fixture_services(root: &std::path::Path, dest: &std::path::Path) {
    if dest.exists() {
        std::fs::remove_dir_all(dest).expect("clear the previous fixture services tree");
    }
    copy_tree(&root.join("services"), dest);
    std::fs::write(dest.join("ai/providers.yaml"), FIXTURE_PROVIDERS)
        .expect("write fixture providers.yaml");
    std::fs::write(dest.join("ai/gateway.yaml"), FIXTURE_GATEWAY)
        .expect("write fixture gateway.yaml");
    enable_identity_server(&dest.join("mcp/sfcore_dev.yaml"));
}

// `sfcore_dev` ships disabled until its upstream verifies identity tokens; the
// suite enables its copy either way so the accessor's signing path is
// exercised.
fn enable_identity_server(path: &std::path::Path) {
    let yaml = std::fs::read_to_string(path).expect("read sfcore_dev.yaml");
    std::fs::write(path, yaml.replacen("enabled: false", "enabled: true", 1))
        .expect("enable fixture sfcore_dev.yaml");
}

fn copy_tree(from: &std::path::Path, to: &std::path::Path) {
    std::fs::create_dir_all(to).expect("create fixture services directory");
    for entry in std::fs::read_dir(from).expect("read services directory") {
        let entry = entry.expect("read services entry");
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("copy services file");
        }
    }
}

// The issuer the admin cookie validator will check tokens against.
pub(crate) fn jwt_issuer() -> String {
    systemprompt::models::Config::get()
        .expect("config installed by init()")
        .jwt_issuer
        .clone()
}
