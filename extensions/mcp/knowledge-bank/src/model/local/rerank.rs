//! BM25 reranking over the candidate set.
//!
//! Document frequencies come from the candidates themselves, so a term that
//! every candidate shares scores nothing and a term only one carries scores
//! most — which is the right prior when the candidates were already chosen
//! for relevance. Scores are normalised to `[0, 1]` against the best
//! candidate; ties keep input order, so the fusion order the retriever
//! produced is preserved wherever BM25 has no opinion.

use super::super::RerankResult;
use super::embed::tokenize;
use std::collections::HashMap;

const K1: f64 = 1.2;
const B: f64 = 0.75;

pub(super) fn rerank(query: &str, documents: &[String], top_n: usize) -> Vec<RerankResult> {
    if documents.is_empty() || top_n == 0 {
        return Vec::new();
    }
    let query_terms = tokenize(query);
    let docs: Vec<Vec<String>> = documents.iter().map(|d| tokenize(d)).collect();
    let total_len: usize = docs.iter().map(Vec::len).sum();
    // Why: token counts are small integers; f64 holds them exactly.
    #[expect(clippy::cast_precision_loss, reason = "small counts, exact in f32/f64")]
    let avg_len = (total_len as f64 / docs.len() as f64).max(1.0);
    let mut df = HashMap::<&str, usize>::new();
    for doc in &docs {
        let mut seen = std::collections::HashSet::new();
        for term in doc {
            if seen.insert(term.as_str()) {
                *df.entry(term.as_str()).or_insert(0) += 1;
            }
        }
    }
    #[expect(clippy::cast_precision_loss, reason = "small counts, exact in f32/f64")]
    let n = docs.len() as f64;
    let scores: Vec<f64> = docs
        .iter()
        .map(|doc| score(doc, &query_terms, &df, n, avg_len))
        .collect();
    let best = scores.iter().copied().fold(0.0f64, f64::max);
    let mut ranked: Vec<RerankResult> = scores
        .iter()
        .enumerate()
        .map(|(index, s)| RerankResult {
            index,
            relevance_score: if best > 0.0 { s / best } else { 0.0 },
        })
        .collect();
    ranked.sort_by(|a, b| {
        b.relevance_score
            .partial_cmp(&a.relevance_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.index.cmp(&b.index))
    });
    ranked.truncate(top_n);
    ranked
}

fn score(
    doc: &[String],
    query_terms: &[String],
    df: &HashMap<&str, usize>,
    n: f64,
    avg_len: f64,
) -> f64 {
    let mut tf = HashMap::<&str, usize>::new();
    for term in doc {
        *tf.entry(term.as_str()).or_insert(0) += 1;
    }
    #[expect(clippy::cast_precision_loss, reason = "small counts, exact in f32/f64")]
    let len = doc.len() as f64;
    query_terms
        .iter()
        .filter_map(|term| {
            let freq = *tf.get(term.as_str())?;
            #[expect(clippy::cast_precision_loss, reason = "small counts, exact in f32/f64")]
            let (freq, doc_freq) = (freq as f64, *df.get(term.as_str()).unwrap_or(&0) as f64);
            let idf = ((n - doc_freq + 0.5) / (doc_freq + 0.5)).ln_1p();
            let norm = freq * (K1 + 1.0) / K1.mul_add(1.0 - B + B * len / avg_len, freq);
            Some(idf * norm)
        })
        .sum()
}
