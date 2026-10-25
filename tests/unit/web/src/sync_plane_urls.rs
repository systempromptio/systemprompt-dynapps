//! Every registered plane has one sync location, read from the plane itself:
//! an owner page's `?tab=sync`, or Access review for access control, on a
//! path some admin router serves. No admin template links an `?entity=`
//! query, which `/admin/sync` never read.

use std::fs;
use std::path::Path;
use systemprompt_web_admin_core::repositories::sync::core_planes::{
    CORE_PLANE_IDS, core_plane_meta,
};

use systemprompt_web_admin_core::repositories::sync::access_control::REVIEW_URL;
use systemprompt_web_admin_core::repositories::sync::plane_ids;
use systemprompt_web_admin_core::repositories::sync::registry::planes;
use systemprompt_web_admin_governance as _;
use systemprompt_web_admin_identity as _;

use crate::support::repo_root;

fn collect(dir: &Path, ext: &str, out: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, ext, out);
        } else if path.extension().is_some_and(|e| e == ext) {
            out.push(fs::read_to_string(&path).unwrap_or_default());
        }
    }
}

fn router_sources() -> String {
    let mut out = Vec::new();
    collect(&repo_root().join("extensions/web"), "rs", &mut out);
    // Why: rustfmt splits a long `.route(` call across lines, so match with
    // whitespace removed.
    out.join("\n").split_whitespace().collect()
}

#[test]
fn every_plane_names_a_served_sync_location() {
    let routers = router_sources();
    let registered = planes();
    assert_eq!(registered.len(), 2, "expected the two install planes");
    let install = registered.iter().map(|plane| {
        let url = plane.sync_url();
        assert_eq!(
            url,
            format!("{}?tab=sync", plane.owner_url()),
            "{}",
            plane.id()
        );
        (plane.id(), url)
    });
    let core = CORE_PLANE_IDS.iter().map(|id| {
        let meta = core_plane_meta(id).expect("core plane meta");
        if meta.id == plane_ids::ACCESS_CONTROL {
            assert_eq!(meta.sync_url, REVIEW_URL);
        }
        (meta.id, meta.sync_url.to_owned())
    });
    for (id, url) in install.chain(core) {
        let path = url.split('?').next().unwrap_or_default();
        let segment = path.strip_prefix("/admin").unwrap_or(path);
        assert!(
            routers.contains(&format!(".route(\"{segment}\"")),
            "plane {id} links {url} but no admin router serves {segment}"
        );
    }
}

#[test]
fn no_admin_template_links_an_entity_query() {
    let mut templates = Vec::new();
    collect(
        &repo_root().join("storage/files/admin"),
        "hbs",
        &mut templates,
    );
    assert!(!templates.is_empty(), "no admin templates found");
    for body in &templates {
        assert!(!body.contains("?entity="), "a template links ?entity=");
        assert!(!body.contains("&entity="), "a template links &entity=");
    }
}
