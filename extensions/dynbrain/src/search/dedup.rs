//! One passage per document, and no near-duplicates, as `_dedup_hits` keeps.
//!
//! Documents stay in the order they ranked. Within a document the passages in
//! the preferred language come first, so a document written in several
//! languages is represented by the one the reader asked in; nothing is
//! filtered by language. A passage whose word set overlaps one already kept
//! by a Jaccard index of 0.85 or more (the mirror of an untranslated page) is
//! dropped.

use std::collections::{BTreeSet, HashMap};

use super::service::Passage;
use crate::ids::ArticleId;

pub const MAX_PER_DOCUMENT: usize = 1;
pub const JACCARD_THRESHOLD: f64 = 0.85;

/// Why a passage was left out of the results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dropped {
    SameDocument,
    NearDuplicate,
}

/// The passages kept and the ones dropped, each with its reason.
#[derive(Debug, Clone, PartialEq)]
pub struct Deduped {
    pub kept: Vec<Passage>,
    pub dropped: Vec<(Passage, Dropped)>,
}

pub fn dedup(passages: Vec<Passage>, prefer_lang: Option<&str>) -> Vec<Passage> {
    dedup_explained(passages, prefer_lang).kept
}

pub fn dedup_explained(passages: Vec<Passage>, prefer_lang: Option<&str>) -> Deduped {
    let mut order: Vec<ArticleId> = Vec::new();
    let mut groups: HashMap<ArticleId, Vec<Passage>> = HashMap::new();
    for passage in passages {
        let article = passage.article;
        groups
            .entry(article)
            .or_insert_with(|| {
                order.push(article);
                Vec::new()
            })
            .push(passage);
    }
    let mut kept: Vec<Passage> = Vec::new();
    let mut dropped: Vec<(Passage, Dropped)> = Vec::new();
    let mut kept_words: Vec<BTreeSet<String>> = Vec::new();
    for article in order {
        let Some(mut candidates) = groups.remove(&article) else {
            continue;
        };
        if let Some(lang) = prefer_lang {
            candidates.sort_by_key(|passage| passage.lang != lang);
        }
        let mut taken = 0;
        for passage in candidates {
            if taken >= MAX_PER_DOCUMENT {
                dropped.push((passage, Dropped::SameDocument));
                continue;
            }
            let words = words_of(&passage.content);
            if !words.is_empty() && kept_words.iter().any(|earlier| overlaps(&words, earlier)) {
                dropped.push((passage, Dropped::NearDuplicate));
                continue;
            }
            kept.push(passage);
            kept_words.push(words);
            taken += 1;
        }
    }
    Deduped { kept, dropped }
}

fn words_of(text: &str) -> BTreeSet<String> {
    text.to_lowercase()
        .split_whitespace()
        .map(str::to_owned)
        .collect()
}

fn overlaps(words: &BTreeSet<String>, earlier: &BTreeSet<String>) -> bool {
    if earlier.is_empty() {
        return false;
    }
    let shared = words.intersection(earlier).count();
    let union = words.union(earlier).count();
    shared as f64 / union as f64 >= JACCARD_THRESHOLD
}
