//! The files of a source's documents, named as the native Brain names its
//! own.
//!
//! `get` of a proxy instance leaves the Markdown as a native `get` would: every
//! reference to a file of the source becomes `asset://<id of the source>`, and
//! `asset_urls` carries the source's own short-lived link for each one, from
//! its `asset_link` as the same person. A reference is a file of the source
//! when its URL path matches one of `file_urls` (`{id}` standing for the id,
//! the host ignored), or when it is a relative path that the source's
//! `asset_list` gives an id for. A file the person may not read at the source
//! keeps its `asset://` reference and gets no link.

use std::collections::BTreeMap;

// JSON: the source's answers are JSON documents.
use serde_json::{Map, Value, json};
use systemprompt::models::execution::context::RequestContext;

use super::ProxyBrain;
use crate::assets::markdown::SCHEME;
use crate::brain::operation::Operation;

pub const MAX_LINKED_FILES: usize = 50;

// JSON: the source's `get` answer, with its file references as `asset://`.
pub(super) async fn with_assets(
    brain: &ProxyBrain,
    context: &RequestContext,
    mut view: Value,
) -> Value {
    let Some(markdown) = view
        .get("markdown")
        .and_then(Value::as_str)
        .map(str::to_owned)
    else {
        return view;
    };
    let article = view.get("article_id").cloned().unwrap_or(Value::Null);
    let targets = link_targets(&markdown);
    let paths = if targets.iter().any(|target| is_relative(target)) {
        listed_paths(brain, context, &article).await
    } else {
        Vec::new()
    };
    let found = source_files(&targets, &brain.config().file_urls, &paths);
    let mut ids: Vec<i64> = found.iter().map(|(_, id)| *id).collect();
    ids.sort_unstable();
    ids.dedup();
    let urls = links(brain, context, &article, &ids).await;
    view["markdown"] = Value::String(rewrite(&markdown, &found));
    view["asset_urls"] = Value::Object(urls);
    view
}

pub fn link_targets(markdown: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for (at, _) in markdown.match_indices("](") {
        let target: String = markdown[at + 2..]
            .chars()
            .take_while(|c| !c.is_whitespace() && *c != ')')
            .collect();
        if !target.is_empty() && !found.contains(&target) {
            found.push(target);
        }
    }
    found
}

pub fn source_files(
    targets: &[String],
    file_urls: &[String],
    paths: &[(String, i64)],
) -> Vec<(String, i64)> {
    targets
        .iter()
        .filter(|target| !target.starts_with(SCHEME))
        .filter_map(|target| {
            url_id(target, file_urls)
                .or_else(|| path_id(target, paths))
                .map(|id| (target.clone(), id))
        })
        .collect()
}

pub fn rewrite(markdown: &str, found: &[(String, i64)]) -> String {
    found
        .iter()
        .fold(markdown.to_owned(), |text, (target, id)| {
            text.replace(&format!("]({target}"), &format!("]({SCHEME}{id}"))
        })
}

fn url_id(target: &str, file_urls: &[String]) -> Option<i64> {
    let path = target.split_once("://").map_or(target, |(_, rest)| {
        rest.find('/').map_or("", |at| &rest[at..])
    });
    file_urls.iter().find_map(|pattern| {
        let (prefix, suffix) = pattern.split_once("{id}")?;
        let rest = path.strip_prefix(prefix)?;
        let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
        let after = rest[digits.len()..].strip_prefix(suffix)?;
        match after.chars().next() {
            Some(c) if c.is_ascii_alphanumeric() => None,
            _ => digits.parse().ok(),
        }
    })
}

fn is_relative(target: &str) -> bool {
    !(target.contains("://")
        || target.starts_with('/')
        || target.starts_with('#')
        || target.starts_with("mailto:")
        || target.starts_with("data:"))
}

fn path_id(target: &str, paths: &[(String, i64)]) -> Option<i64> {
    if !is_relative(target) {
        return None;
    }
    let mut name = target;
    while let Some(rest) = name.strip_prefix("./").or_else(|| name.strip_prefix("../")) {
        name = rest;
    }
    let tail = format!("/{name}");
    paths
        .iter()
        .find(|(path, _)| path == name || path.ends_with(&tail))
        .map(|(_, id)| *id)
}

async fn listed_paths(
    brain: &ProxyBrain,
    context: &RequestContext,
    article: &Value,
) -> Vec<(String, i64)> {
    let listed = match brain
        .relay(
            context,
            Operation::AssetList,
            json!({ "article_id": article }),
        )
        .await
    {
        Ok(listed) => listed,
        Err(error) => {
            tracing::debug!(%error, "dynbrain: the source listed no files");
            return Vec::new();
        },
    };
    listed
        .get("files")
        .and_then(Value::as_array)
        .map_or_else(Vec::new, |files| {
            files
                .iter()
                .filter_map(|file| {
                    let id = file.get("attachment_id").and_then(Value::as_i64)?;
                    let path = file
                        .get("path")
                        .and_then(Value::as_str)
                        .filter(|path| !path.is_empty())
                        .or_else(|| file.get("name").and_then(Value::as_str))?;
                    Some((path.to_owned(), id))
                })
                .collect()
        })
}

async fn links(
    brain: &ProxyBrain,
    context: &RequestContext,
    article: &Value,
    ids: &[i64],
) -> Map<String, Value> {
    let mut urls = BTreeMap::new();
    for id in ids.iter().take(MAX_LINKED_FILES) {
        let args = json!({ "attachment_id": id, "article_id": article });
        match brain.relay(context, Operation::AssetLink, args).await {
            Ok(link) => {
                if let Some(url) = link.get("download_url").and_then(Value::as_str) {
                    urls.insert(format!("{SCHEME}{id}"), Value::String(url.to_owned()));
                }
            },
            Err(error) => {
                tracing::debug!(%error, file = id, "dynbrain: no source link for a file");
            },
        }
    }
    urls.into_iter().collect()
}
