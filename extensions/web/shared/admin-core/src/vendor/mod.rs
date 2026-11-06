//! The third-party frontend allowlist, as the workspace page head sees it.
//!
//! `services/web/vendor.yaml` is the single declaration of vendored frontend
//! code. A page that opts into the workspace layout gets [`VendorHead`]: an
//! import map naming only the listed modules, with SRI for each, the listed
//! stylesheets, and the listed WASM binaries. Every other page gets nothing,
//! so the rest of the console never loads vendor code.

pub mod assets;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

const MANIFEST: &str = include_str!("../../../../../../services/web/vendor.yaml");
const URL_PREFIX: &str = "/js/vendor/";

#[derive(Debug, Deserialize)]
pub struct VendorManifest {
    pub packages: Vec<VendorPackage>,
}

#[derive(Debug, Deserialize)]
pub struct VendorPackage {
    pub name: String,
    pub version: String,
    pub license: String,
    pub source: VendorSource,
    pub why: String,
    pub owner: String,
    pub approved_by: String,
    pub approved_on: String,
    pub bundled: Vec<BundledDependency>,
    pub files: Vec<VendorFile>,
}

#[derive(Debug, Deserialize)]
pub struct VendorSource {
    pub tarball: String,
    pub integrity: String,
    pub entry: String,
    pub bundler: String,
}

#[derive(Debug, Deserialize)]
pub struct BundledDependency {
    pub name: String,
    pub version: String,
    pub license: String,
    pub integrity: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VendorFileKind {
    Module,
    Wasm,
    Style,
}

#[derive(Debug, Deserialize)]
pub struct VendorFile {
    pub path: String,
    pub kind: VendorFileKind,
    #[serde(default)]
    pub specifier: Option<String>,
    pub sha256: String,
    pub sri: String,
}

impl VendorFile {
    #[must_use]
    pub fn url(&self) -> String {
        format!("{URL_PREFIX}{}", self.path)
    }
}

pub fn manifest() -> Result<VendorManifest, serde_yaml::Error> {
    serde_yaml::from_str(MANIFEST)
}

#[derive(Debug, thiserror::Error)]
pub enum VendorError {
    #[error("vendor.yaml does not parse: {0}")]
    Manifest(#[from] serde_yaml::Error),
    #[error("vendor import map does not serialize: {0}")]
    ImportMap(#[from] serde_json::Error),
}

pub fn workspace_head() -> Result<VendorHead, VendorError> {
    Ok(vendor_head(&manifest()?)?)
}

#[derive(Debug, Serialize)]
struct ImportMap {
    imports: BTreeMap<String, String>,
    integrity: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VendorLink {
    pub url: String,
    pub integrity: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VendorHead {
    pub import_map: String,
    pub styles: Vec<VendorLink>,
    pub wasm: String,
}

pub fn vendor_head(manifest: &VendorManifest) -> Result<VendorHead, serde_json::Error> {
    let mut map = ImportMap {
        imports: BTreeMap::new(),
        integrity: BTreeMap::new(),
    };
    let mut styles = Vec::new();
    let mut wasm = BTreeMap::new();
    for package in &manifest.packages {
        for file in &package.files {
            let link = VendorLink {
                url: file.url(),
                integrity: file.sri.clone(),
            };
            match file.kind {
                VendorFileKind::Module => {
                    let specifier = file.specifier.as_deref().unwrap_or(&package.name);
                    map.imports.insert(specifier.to_owned(), link.url.clone());
                    map.integrity.insert(link.url, link.integrity);
                },
                VendorFileKind::Style => styles.push(link),
                VendorFileKind::Wasm => {
                    wasm.insert(package.name.clone(), link);
                },
            }
        }
    }
    Ok(VendorHead {
        import_map: script_safe(&serde_json::to_string(&map)?),
        styles,
        wasm: script_safe(&serde_json::to_string(&wasm)?),
    })
}

fn script_safe(json: &str) -> String {
    json.replace("</", "<\\/")
}
