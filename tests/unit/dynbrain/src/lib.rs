//! Unit tests for `dynbrain` pure logic:
//! - the access rule: what an ungranted article gives, space grants, own grants
//!   adding to the inherited ones or replacing a subject's level, `none`,
//!   `inherit_access` off cutting, what an article inherits, the owner and
//!   administrators, the subtree a rewrite covers
//! - levels and subject kinds as the schema stores them
//! - typed properties: the definitions a space may declare and the values a
//!   variant may carry, for every type
//! - language codes, their text-search configuration and the content hash
//! - the Brain's contract: how a fault answers over HTTP, how a store error
//!   becomes a fault, and the field names the operations take and return
//! - files: the type read from the bytes, which types are shown inline, the
//!   `asset://` references of a body and the indexed text without them (no
//!   URL), and signed links that hold for one purpose, target and time
//! - the ingestion: the chunker against chunks Python produced from documents
//!   of dynbrain (three of its docs and an edge-case text), a file reference
//!   becoming its alt text, eligibility denied by default, the context prefix
//!   by strategy (none for code), the synopsis prompt, and a batch that the
//!   provider refuses in part, all or with a rate limit
//! - the search: Reciprocal Rank Fusion, one passage per document and no
//!   near-duplicates, the preferred language, the confidence verdict and its
//!   threshold, reranking that fails open with its trace, the breaker that
//!   opens, doubles and closes, the query-cache key, the language of a query,
//!   the `where` filter, and the answer's context (whole documents, best chunks
//!   in reading order with gaps, marked cuts, numbered blocks)
//! - the extension registration, its schema, and the vocabulary rule: no file
//!   of the crate, of the `brain` server or of the Brain's config names the
//!   source system the proxy mode talks to
//! - proxy mode: the config file and what it refuses at startup (an unknown
//!   mode or key, no source, an empty tool, an undeclared server), arguments
//!   renamed with constants added, the eighteen operation names, the source's
//!   answers and refusals as faults, the source's file references turned into
//!   `asset://`, and the links key the API and the `brain` server share
//!
//! The repository runs against Postgres in the integration suite.

#[cfg(test)]
mod access;
#[cfg(test)]
mod assets;
#[cfg(test)]
mod basics;
#[cfg(test)]
mod bisect;
#[cfg(test)]
mod brain;
#[cfg(test)]
mod chunker;
#[cfg(test)]
mod ingest;
#[cfg(test)]
mod properties;
#[cfg(test)]
mod proxy;
#[cfg(test)]
mod search;
#[cfg(test)]
mod search_context;
