//! Feature-hashed n-gram embeddings.
//!
//! Every lowercase alphanumeric token, and every run of up to `ngram`
//! adjacent tokens, is FNV-1a hashed; the hash picks a slot (`h % dim`) and a
//! sign (its top bit), and the slot accumulates `sign * ln(1 + tf)`. The
//! vector is L2-normalised so cosine distance in pgvector is meaningful, and
//! an empty text yields a fixed unit vector rather than NaN. It is a
//! lexical embedding — two texts are close when they share words and word
//! pairs — which is exactly what a free, deterministic, dependency-free
//! provider can promise; the hybrid retriever's BM25 arm and the reranker
//! carry the rest.

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

pub(super) fn embed(text: &str, dim: usize, ngram: usize) -> Vec<f32> {
    let dim = dim.max(1);
    let tokens = tokenize(text);
    let mut counts = std::collections::HashMap::<u64, u32>::new();
    for width in 1..=ngram.max(1) {
        for window in tokens.windows(width) {
            *counts.entry(hash_ngram(window)).or_insert(0) += 1;
        }
    }
    let mut vector = vec![0.0f32; dim];
    for (hash, tf) in counts {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "the modulo bounds the slot below dim"
        )]
        let slot = (hash % dim as u64) as usize;
        let sign = if hash >> 63 == 0 { 1.0 } else { -1.0 };
        #[expect(clippy::cast_precision_loss, reason = "small counts, exact in f32/f64")]
        let weight = (tf as f32).ln_1p();
        vector[slot] += sign * weight;
    }
    normalise(&mut vector);
    vector
}

pub(super) fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(str::to_lowercase)
        .collect()
}

fn hash_ngram(tokens: &[String]) -> u64 {
    let mut hash = FNV_OFFSET;
    for (i, token) in tokens.iter().enumerate() {
        if i > 0 {
            hash = fnv_step(hash, b' ');
        }
        for byte in token.bytes() {
            hash = fnv_step(hash, byte);
        }
    }
    hash
}

const fn fnv_step(hash: u64, byte: u8) -> u64 {
    (hash ^ byte as u64).wrapping_mul(FNV_PRIME)
}

fn normalise(vector: &mut [f32]) {
    let norm = vector.iter().map(|v| v * v).sum::<f32>().sqrt();
    if norm > 0.0 && norm.is_finite() {
        for v in vector.iter_mut() {
            *v /= norm;
        }
    } else {
        vector.fill(0.0);
        if let Some(first) = vector.first_mut() {
            *first = 1.0;
        }
    }
}
