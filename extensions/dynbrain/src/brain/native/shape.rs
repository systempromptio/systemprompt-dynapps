//! The pure helpers of the operations.
//!
//! They cover the access comparison, choosing the variant to show, paths and
//! slugs, validating a write against its space and merging a change into a
//! variant.
//!
//! A path names the ancestors the reader can read; each run of ancestors they
//! cannot read is shown as one `…`, so a path never reveals a title or a slug
//! of a page its reader has no access to.

use std::collections::BTreeSet;

use crate::access::Level;
use crate::brain::args::Object;
use crate::brain::fault::BrainFault;
use crate::brain::views::VariantInfo;
use crate::error::BrainError;
use crate::ids::ArticleId;
use crate::model::{Article, Space, Variant, VariantWrite};

const HIDDEN_ANCESTOR: &str = "…";

#[derive(Debug, Clone, Copy)]
pub(super) struct Change<'a> {
    pub(super) title: Option<&'a str>,
    pub(super) markdown: Option<&'a str>,
    pub(super) front_matter: Option<&'a Object>,
    pub(super) metadata: Option<&'a Object>,
}

pub(super) fn require(level: Level, need: Level) -> Result<(), BrainFault> {
    if level >= need {
        return Ok(());
    }
    Err(BrainFault::Forbidden)
}

pub(super) fn pick(
    mut variants: Vec<Variant>,
    lang: Option<&str>,
    default: &str,
) -> Option<Variant> {
    let wanted = lang
        .into_iter()
        .chain([default])
        .find_map(|code| variants.iter().position(|variant| variant.lang == code));
    let index = wanted.or_else(|| (!variants.is_empty()).then_some(0))?;
    Some(variants.swap_remove(index))
}

pub(super) fn blank(article: &Article, space: &Space) -> Variant {
    Variant {
        article: article.id,
        lang: space.default_lang.clone(),
        title: article.slug.clone(),
        description: String::new(),
        body: String::new(),
        front_matter: Object::new(),
        properties: Object::new(),
        content_hash: String::new(),
        revision: 0,
        updated_by: None,
        created_at: article.created_at,
        updated_at: article.updated_at,
    }
}

pub(super) fn variant_info(variant: &Variant, space: &Space) -> VariantInfo {
    VariantInfo {
        lang: variant.lang.clone(),
        is_default: variant.lang == space.default_lang,
    }
}

pub(super) fn path_of(
    space: &Space,
    article: &Article,
    all: &[Article],
    seen: &BTreeSet<ArticleId>,
) -> String {
    let mut parts = vec![article.slug.as_str()];
    let mut parent = article.parent;
    while let Some(id) = parent {
        let Some(up) = all.iter().find(|candidate| candidate.id == id) else {
            break;
        };
        let shown = if seen.contains(&up.id) {
            up.slug.as_str()
        } else {
            HIDDEN_ANCESTOR
        };
        if shown != HIDDEN_ANCESTOR || parts.last() != Some(&HIDDEN_ANCESTOR) {
            parts.push(shown);
        }
        parent = up.parent;
    }
    parts.push(&space.slug);
    parts.reverse();
    parts.join("/")
}

pub(super) fn lang_of(lang: Option<&str>, space: &Space) -> String {
    lang.map_or_else(|| space.default_lang.clone(), ToOwned::to_owned)
}

pub(super) fn checked(space: &Space, lang: &str, write: &VariantWrite) -> Result<(), BrainFault> {
    if write.title.trim().is_empty() {
        return Err(BrainFault::Invalid("a document needs a title"));
    }
    if !space.langs.iter().any(|known| known == lang) {
        return Err(BrainFault::Refused(BrainError::LanguageNotInSpace {
            lang: lang.to_owned(),
            space: space.id,
        }));
    }
    space.properties.validate(&write.properties)?;
    Ok(())
}

pub(super) fn compose(change: &Change<'_>, base: Option<&Variant>) -> VariantWrite {
    let mut write = base.map_or_else(
        || VariantWrite {
            title: String::new(),
            description: String::new(),
            body: String::new(),
            front_matter: Object::new(),
            properties: Object::new(),
        },
        |variant| VariantWrite {
            title: variant.title.clone(),
            description: variant.description.clone(),
            body: variant.body.clone(),
            front_matter: variant.front_matter.clone(),
            properties: variant.properties.clone(),
        },
    );
    if let Some(title) = change.title.filter(|title| !title.trim().is_empty()) {
        title.clone_into(&mut write.title);
    }
    if let Some(markdown) = change.markdown {
        markdown.clone_into(&mut write.body);
    }
    for (key, value) in change.front_matter.into_iter().flatten() {
        match (key.as_str(), value.as_str()) {
            ("description", Some(text)) => text.clone_into(&mut write.description),
            ("description", None) => {},
            _ => {
                write.front_matter.insert(key.clone(), value.clone());
            },
        }
    }
    for (key, value) in change.metadata.into_iter().flatten() {
        if value.is_null() {
            write.properties.remove(key);
        } else {
            write.properties.insert(key.clone(), value.clone());
        }
    }
    write
}

pub(super) fn slugify(title: &str) -> String {
    let flat: String = title.chars().map(fold).collect();
    let joined = flat
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let slug: String = joined.chars().take(100).collect();
    if slug.is_empty() {
        return "article".to_owned();
    }
    slug
}

fn fold(letter: char) -> char {
    let lower = letter.to_lowercase().next().unwrap_or(letter);
    match lower {
        'a'..='z' | '0'..='9' => lower,
        'á' | 'à' | 'ä' | 'â' => 'a',
        'é' | 'è' | 'ë' | 'ê' => 'e',
        'í' | 'ì' | 'ï' | 'î' => 'i',
        'ó' | 'ò' | 'ö' | 'ô' => 'o',
        'ú' | 'ù' | 'ü' | 'û' => 'u',
        'ñ' => 'n',
        'ç' => 'c',
        _ => '-',
    }
}
