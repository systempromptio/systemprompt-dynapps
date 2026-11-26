//! The language a query is written in, among the languages of the corpus.
//!
//! dynbrain's stopword heuristic: it counts the function words of each
//! candidate language, and a winner needs at least two hits and 1.3 times the
//! runner-up, or there is none. It only picks which variant represents a
//! document that exists in several languages; it never filters.

use std::cmp::Reverse;

const MIN_HITS: usize = 2;
const MIN_RATIO: f64 = 1.3;

const STOPWORDS: [(&str, &[&str]); 7] = [
    (
        "es",
        &[
            "el", "la", "los", "las", "de", "que", "y", "en", "un", "una", "por", "con", "para",
            "su", "se", "del", "al", "como", "más", "pero", "sus", "le", "ya", "o", "este", "esta",
            "cuando",
        ],
    ),
    (
        "en",
        &[
            "the", "of", "and", "to", "in", "a", "is", "that", "it", "for", "as", "with", "was",
            "on", "are", "this", "be", "by", "an", "which", "or", "from", "at", "you", "your",
            "can", "will",
        ],
    ),
    (
        "fr",
        &[
            "le", "la", "les", "de", "des", "et", "en", "un", "une", "que", "qui", "dans", "pour",
            "pas", "sur", "au", "avec", "ce", "il", "vous", "est", "plus", "par", "ne", "se", "sa",
            "ses",
        ],
    ),
    (
        "nl",
        &[
            "de", "het", "een", "en", "van", "in", "is", "dat", "op", "te", "voor", "met", "die",
            "niet", "aan", "zijn", "er", "maar", "om", "ook", "als", "dan", "je", "uit", "naar",
            "door", "worden",
        ],
    ),
    (
        "de",
        &[
            "der", "die", "das", "und", "in", "den", "von", "zu", "mit", "ist", "auf", "für",
            "des", "dem", "nicht", "ein", "eine", "als", "auch", "es", "an", "werden", "aus",
            "sich", "oder", "wird", "durch",
        ],
    ),
    (
        "pt",
        &[
            "o", "a", "os", "as", "de", "que", "e", "em", "um", "uma", "por", "com", "para", "não",
            "se", "do", "da", "dos", "das", "no", "na", "mais", "como", "ao", "mas", "seu", "sua",
        ],
    ),
    (
        "it",
        &[
            "il", "lo", "la", "i", "gli", "le", "di", "che", "e", "in", "un", "una", "per", "con",
            "non", "si", "del", "della", "sono", "come", "più", "al", "da", "se", "ma", "anche",
            "questo",
        ],
    ),
];

pub fn detect(text: &str, allowed: &[String]) -> Option<String> {
    let candidates: Vec<(&str, &[&str])> = allowed
        .iter()
        .filter_map(|lang| {
            STOPWORDS
                .iter()
                .find(|(code, _)| *code == lang.as_str())
                .copied()
        })
        .collect();
    if candidates.is_empty() {
        return None;
    }
    let words: Vec<String> = text
        .split(|letter: char| !letter.is_alphabetic())
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
        .collect();
    if words.is_empty() {
        return None;
    }
    let mut ranked: Vec<(&str, usize)> = candidates
        .iter()
        .map(|(code, list)| {
            let hits = words
                .iter()
                .filter(|word| list.contains(&word.as_str()))
                .count();
            (*code, hits)
        })
        .collect();
    ranked.sort_by_key(|(_, hits)| Reverse(*hits));
    let (best_lang, best) = ranked.first().copied()?;
    if best < MIN_HITS {
        return None;
    }
    let runner_up = ranked.get(1).map_or(0, |(_, hits)| *hits);
    if runner_up > 0 && (best as f64) < runner_up as f64 * MIN_RATIO {
        return None;
    }
    Some(best_lang.to_owned())
}
