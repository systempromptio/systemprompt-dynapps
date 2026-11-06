//! The third-party frontend allowlist (`services/web/vendor.yaml`), asserted
//! from cargo: the manifest and `storage/files/js/vendor/` agree byte for
//! byte, every licence is one deny.toml allows, the generated asset
//! registration matches the manifest, the workspace import map names only
//! listed modules, no first-party JS or template reaches vendor code any other
//! way, and the vendored Loro speaks the same wire version as the server's.
//! `scripts/check-vendor.sh` runs the same rules as a lint gate.

use crate::support::{repo_root, walk};

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::Path;

use base64::Engine as _;
use sha2::{Digest, Sha256, Sha384};
use systemprompt_web_admin_core::vendor::assets::vendor_assets;
use systemprompt_web_admin_core::vendor::{VendorFileKind, VendorManifest, manifest, vendor_head};

const LORO_JS: &str = "loro-crdt";
const LORO_RUST: &str = "loro";

fn load() -> VendorManifest {
    manifest().expect("services/web/vendor.yaml parses")
}

fn listed_paths(m: &VendorManifest) -> BTreeSet<String> {
    m.packages
        .iter()
        .flat_map(|p| p.files.iter().map(|f| f.path.clone()))
        .collect()
}

fn files_under(dir: &Path) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in std::fs::read_dir(&d).unwrap_or_else(|e| panic!("read {}: {e}", d.display())) {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                stack.push(path);
            } else {
                let rel = path.strip_prefix(dir).expect("under vendor dir");
                out.insert(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    out
}

#[test]
fn every_package_is_pinned_sourced_and_explained() {
    let m = load();
    assert!(!m.packages.is_empty(), "vendor.yaml lists no packages");
    for p in &m.packages {
        assert!(
            p.version.chars().all(|c| c.is_ascii_digit() || c == '.'),
            "{}: version {:?} is not exact",
            p.name,
            p.version
        );
        assert!(
            p.source.integrity.starts_with("sha512-"),
            "{}: no registry integrity",
            p.name
        );
        assert!(
            p.source.tarball.starts_with("https://registry.npmjs.org/"),
            "{}",
            p.name
        );
        assert!(
            !p.why.trim().is_empty() && !p.owner.trim().is_empty(),
            "{}: why/owner",
            p.name
        );
        for f in &p.files {
            let dir = format!("{}@{}/", p.name, p.version);
            assert!(f.path.starts_with(&dir), "{} is not under {dir}", f.path);
        }
    }
}

#[test]
fn vendor_dir_and_manifest_agree_byte_for_byte() {
    let root = repo_root();
    let dir = root.join("storage/files/js/vendor");
    let m = load();
    let listed = listed_paths(&m);
    let on_disk = files_under(&dir);
    let unlisted: Vec<_> = on_disk.difference(&listed).collect();
    let missing: Vec<_> = listed.difference(&on_disk).collect();
    assert!(
        unlisted.is_empty(),
        "vendor files not in vendor.yaml: {unlisted:?}"
    );
    assert!(
        missing.is_empty(),
        "vendor.yaml files missing on disk: {missing:?}"
    );
    for f in m.packages.iter().flat_map(|p| &p.files) {
        let data = std::fs::read(dir.join(&f.path)).expect("vendored file readable");
        let sha256 = Sha256::digest(&data)
            .iter()
            .fold(String::with_capacity(64), |mut hex, b| {
                write!(hex, "{b:02x}").expect("writing to a String cannot fail");
                hex
            });
        assert_eq!(
            sha256, f.sha256,
            "{}: sha256 differs from vendor.yaml",
            f.path
        );
        let sri = format!(
            "sha384-{}",
            base64::engine::general_purpose::STANDARD.encode(Sha384::digest(&data))
        );
        assert_eq!(sri, f.sri, "{}: sri differs from vendor.yaml", f.path);
    }
}

fn allowed_licences(root: &Path) -> BTreeSet<String> {
    let deny: toml::Table = std::fs::read_to_string(root.join("deny.toml"))
        .expect("deny.toml")
        .parse()
        .expect("deny.toml parses");
    deny["licenses"]["allow"]
        .as_array()
        .expect("licenses.allow")
        .iter()
        .filter_map(|v| v.as_str().map(str::to_owned))
        .collect()
}

#[test]
fn every_licence_is_in_the_deny_allow_list() {
    let allowed = allowed_licences(&repo_root());
    for p in load().packages {
        assert!(
            allowed.contains(&p.license),
            "{}: licence {}",
            p.name,
            p.license
        );
        for dep in &p.bundled {
            assert!(
                allowed.contains(&dep.license),
                "{} inlines {} ({})",
                p.name,
                dep.name,
                dep.license
            );
        }
    }
}

#[test]
fn registered_vendor_assets_are_exactly_the_manifest() {
    let root = repo_root();
    let storage_js = root.join("storage/files/js");
    let registered: BTreeSet<String> = vendor_assets(&storage_js)
        .iter()
        .map(|a| a.destination().trim_start_matches("js/vendor/").to_owned())
        .collect();
    assert_eq!(
        registered,
        listed_paths(&load()),
        "admin-core vendor/assets.rs is stale: run scripts/vendor-assets-rs.py"
    );
}

#[test]
fn import_map_names_every_module_with_its_integrity() {
    let m = load();
    let head = vendor_head(&m).expect("vendor head");
    assert!(
        !head.import_map.contains("</"),
        "import map not script-safe"
    );
    for p in &m.packages {
        for f in &p.files {
            match f.kind {
                VendorFileKind::Module => {
                    let entry = format!(
                        "\"{}\":\"{}\"",
                        f.specifier.as_deref().unwrap_or(&p.name),
                        f.url()
                    );
                    assert!(head.import_map.contains(&entry), "import map lacks {entry}");
                    assert!(
                        head.import_map.contains(&f.sri),
                        "import map lacks SRI for {}",
                        f.path
                    );
                },
                VendorFileKind::Style => assert!(head.styles.iter().any(|s| s.url == f.url())),
                VendorFileKind::Wasm => assert!(head.wasm.contains(&f.sri)),
            }
        }
    }
}

fn specifiers(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let trimmed = line.trim_start();
    let is_import_stmt =
        trimmed.starts_with("import") || trimmed.starts_with("export") || line.contains('}');
    for needle in [
        "from '",
        "from \"",
        "import('",
        "import(\"",
        "import '",
        "import \"",
    ] {
        if needle.starts_with("from") && !is_import_stmt {
            continue;
        }
        if needle.starts_with("import ") && !trimmed.starts_with(needle) {
            continue;
        }
        for (idx, _) in line.match_indices(needle) {
            let rest = &line[idx + needle.len()..];
            let quote = needle.chars().last().unwrap_or('"');
            if let Some(end) = rest.find(quote) {
                out.push(rest[..end].to_owned());
            }
        }
    }
    out
}

#[test]
fn first_party_js_reaches_vendor_code_only_through_the_import_map() {
    let root = repo_root();
    let js_root = root.join("storage/files/js");
    let vendor = js_root.join("vendor");
    let allowed: BTreeSet<String> = load()
        .packages
        .iter()
        .flat_map(|p| p.files.iter().filter_map(|f| f.specifier.clone()))
        .collect();
    let mut files = Vec::new();
    walk(&js_root, "js", &mut files);
    let mut violations = Vec::new();
    for file in files.iter().filter(|f| !f.starts_with(&vendor)) {
        let content = std::fs::read_to_string(file).expect("js readable");
        for spec in content.lines().flat_map(specifiers) {
            let external =
                spec.starts_with("http:") || spec.starts_with("https:") || spec.starts_with("//");
            let direct = spec.starts_with("/js/vendor/");
            let bare = !spec.starts_with('/') && !spec.starts_with('.') && !allowed.contains(&spec);
            if external || direct || bare {
                violations.push(format!("{}: {spec}", file.display()));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "imports outside the vendor allowlist:\n{}",
        violations.join("\n")
    );
}

#[test]
fn only_the_vendor_head_partial_names_vendor_paths() {
    let root = repo_root();
    let mut templates = Vec::new();
    walk(&root.join("storage/files/admin"), "hbs", &mut templates);
    walk(&root.join("services/web/templates"), "html", &mut templates);
    let offenders: Vec<String> = templates
        .iter()
        .filter(|t| !t.ends_with("partials/vendor-head.hbs"))
        .filter(|t| std::fs::read_to_string(t).is_ok_and(|c| c.contains("/js/vendor/")))
        .map(|t| t.display().to_string())
        .collect();
    assert!(
        offenders.is_empty(),
        "templates naming /js/vendor/: {offenders:?}"
    );
}

fn locked_version(lock: &str, crate_name: &str) -> Option<String> {
    let header = format!("name = \"{crate_name}\"\n");
    let at = lock.find(&header)? + header.len();
    let line = lock[at..].lines().next()?;
    line.strip_prefix("version = \"")?
        .strip_suffix('"')
        .map(str::to_owned)
}

#[test]
fn vendored_loro_matches_the_server_engine_version() {
    let root = repo_root();
    let lock = std::fs::read_to_string(root.join("Cargo.lock")).expect("Cargo.lock");
    let server = locked_version(&lock, LORO_RUST).expect("loro in Cargo.lock");
    let client = load()
        .packages
        .into_iter()
        .find(|p| p.name == LORO_JS)
        .map(|p| p.version)
        .expect("loro-crdt in vendor.yaml");
    assert_eq!(
        client, server,
        "vendored loro-crdt must equal the server's loro crate"
    );
}
