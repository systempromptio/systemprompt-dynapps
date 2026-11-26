//! Retrieval: the hybrid search and the answer context, ported from dynbrain.
//!
//! [`Retriever`] runs one search the way `docs_search` does. Two arms read
//! the index in SQL, each with the reader's access inside the query: the
//! vector arm (pgvector HNSW, `hnsw.ef_search` 200) and the text arm
//! (`plainto_tsquery` in each chunk's language), 100 candidates each. They
//! are fused by Reciprocal Rank Fusion (`k = 60`), reranked by Cohere when
//! the instance has its key (failing open to the fused order, behind a
//! circuit breaker), and collapsed to one chunk per document. A query that
//! cannot be embedded falls back to a keyword match over the documents, also
//! filtered in SQL, and vectors of another embedding configuration switch the
//! vector arm off.
//!
//! The pure parts are their own modules: [`fusion`], [`rerank`] (the trace and
//! the breaker), [`dedup`], [`confidence`], [`context`] (the answer's text,
//! chunks whole and in reading order), [`filter`] (`where`), [`lang`] (the
//! stopword detector that picks the preferred variant) and [`query`] (the
//! cached query vector). [`cohere`] is the production reranker.

pub mod cohere;
pub mod confidence;
pub mod context;
pub mod dedup;
pub mod filter;
pub mod fusion;
pub mod lang;
pub mod query;
pub mod rerank;
pub mod service;

pub use service::{
    ArmScore, ExplainRow, Explained, Outcome, Passage, Retrieval, RetrievalMode, RetrievalRequest,
    RetrievalSettings, Retriever, SearchScope, Timings,
};
