//! The `dynbrain` repository against Postgres, in a throwaway database with
//! the full extension schema:
//!
//! - spaces: create, read, list, edit, and the languages a space refuses
//! - the article tree: order, children, and creating under a parent
//! - variants: every write bumps the revision and appends it, a language the
//!   space does not list or a property the schema refuses writes nothing, and
//!   revisions refuse updates
//! - permissions: nobody without a grant reaches an article but its owner and
//!   administrators; a space grant reaches every article; a grant reaches the
//!   subtree and revoking it takes it back; a child's grant keeps the parent's
//!   subjects and can lower one; `none` removes a subject for the subtree;
//!   `inherit_access` off cuts, and with `copy_inherited` copies; a group grant
//!   reaches its live members and a role grant the users holding the role; an
//!   inactive account reaches nothing; a grant on another space's article is
//!   refused
//! - moving: the subtree's closure is rewritten, so what it inherited from its
//!   old place is gone; moves below itself or into another space fail
//! - trash: an article goes with its subtree and comes back with it
//! - space level: the owner and administrators manage a space, and the rest
//!   reach it through space grants for them, their roles or their groups
//!
//! The Brain routes are driven in process, mounted as core mounts them, over a
//! throwaway database with a real RS256 token authority:
//!
//! - auth: the desktop and the mobile JWT of a live phone map to one person;
//!   every other credential, and a revoked phone, is `401`
//! - reading: spaces, an article in the language asked for, children, the
//!   outline (cut at `max_nodes`), the flat list, the export, the metadata and
//!   the schema show only what the person reaches, and what they do not reach
//!   is `404`
//! - writing: create, create a tree, update, move and trash need `write` (`403`
//!   for a reader, `404` for someone who cannot read), refuse what the space
//!   does not accept and leave nothing behind
//! - grants: space and article grants and `inherit_access` need `manage`;
//!   `none` is written and listed as a grant, never as access
//! - search and answer answer for the person of the token (`401` without one);
//!   a path hides the ancestors its reader cannot read, collapsed to one `…`; a
//!   list's `where` names declared properties and `false` means not set
//!
//! The indexing, with the embedding and synopsis models replaced by
//! deterministic doubles:
//!
//! - a write queues its content once; the same content twice, or content that
//!   goes back to what the chunks already match, is not indexed or embedded
//!   again
//! - nothing is indexed unless a rule or the article's mode says so, and the
//!   mode beats the rules; a rule added later applies when the index is
//!   requeued
//! - an indexed variant leaves chunks with their heading path, a text-search
//!   vector in the variant's language and an embedding with its fingerprint;
//!   the context prefix goes only into the embedding
//! - synopses only for chunks with no heading, kept for chunks whose text did
//!   not change
//! - no `asset://` reference reaches a chunk, a synopsis prompt or an embedding
//! - without a key the chunks wait with their text search and embed later; a
//!   different fingerprint is detected; the cost of each batch is recorded; the
//!   trash keeps chunks and holds back indexing
//! - the upload purge job deletes expired uploads that no article adopted, with
//!   their bytes, and nothing else
//!
//! The search and the answer context, over chunks written straight into the
//! index, with a one-hot query embedder and a scripted reranker:
//!
//! - denial: neither arm, the keyword fallback nor `answer` returns a chunk of
//!   an article the reader cannot read; the arms fill their pool with what the
//!   reader may read; a suspended account finds nothing; the trash is out of
//!   every path; a space or a language keeps the search inside it
//! - ranking: a chunk in both arms outranks one in the text arm only
//!   (Reciprocal Rank Fusion); a foreign fingerprint switches the vector arm
//!   off; a failing reranker leaves `unrated` and the fused order; the breaker
//!   opens on a failure on the wire and not on a refusal; a working reranker
//!   orders and rates; the threshold is a setting; one passage per document in
//!   the language asked for or detected; a repeated query is embedded once and
//!   unused ones age out; without an embedder the search matches keywords; the
//!   `where` filter
//! - answer: numbered blocks and citations in order, whole documents or their
//!   best chunks in reading order with gaps, keyword matches without a query
//!   vector, and no model called
//! - the Cohere client against a local server: its key, its scores, a refusal
//!   and an outage told apart, and the outbound URL guard
//!
//! The files of articles, over core's local-disk file storage:
//!
//! - the full cycle: a one-time upload URL, the POST of the bytes, adopting
//!   them as a file, `get` and `export` with signed URLs, and downloading the
//!   same bytes; the bytes are written and read through core's file storage,
//!   and purging an article deletes them
//! - denials: uploading without `write`, an upload id used twice or by another
//!   person, an expired upload link, a file over the limit, a link to a file of
//!   an unreadable article, a tampered or expired download link, a file in the
//!   trash; SVG and HTML are served as downloads, never as pages
//!
//! Proxy mode, against a fake source served over real MCP (rmcp's streamable
//! HTTP server) and called through core's MCP client:
//!
//! - every operation reaches the tool its mapping names, with the arguments
//!   renamed and the constants added; the person's token travels with the call
//!   and another person carries another; a 401 of the source is `forbidden`, a
//!   call without the person's token never reaches it, an unmapped operation is
//!   `unavailable`, and grants are `managed_at_source`
//! - the REST routes and the `brain` tools answer what the source answers
//! - files: `get` names the source's files `asset://<id>` with the source's
//!   links in `asset_urls`, `GET assets/{id}` redirects to the source's link
//!   (`404` where the source refuses), file writes go to the source as the
//!   person, and no byte is kept here

#[cfg(test)]
mod asset_support;
#[cfg(test)]
mod fixture;
#[cfg(test)]
mod http_access;
#[cfg(test)]
mod http_asset_denials;
#[cfg(test)]
mod http_assets;
#[cfg(test)]
mod http_auth;
#[cfg(test)]
mod http_read;
#[cfg(test)]
mod http_search;
#[cfg(test)]
mod http_support;
#[cfg(test)]
mod http_write;
#[cfg(test)]
mod ingest;
#[cfg(test)]
mod ingest_support;
#[cfg(test)]
mod moves;
#[cfg(test)]
mod permissions;
#[cfg(test)]
mod proxy;
#[cfg(test)]
mod proxy_files;
#[cfg(test)]
mod proxy_support;
#[cfg(test)]
mod search_access;
#[cfg(test)]
mod search_answer;
#[cfg(test)]
mod search_ranking;
#[cfg(test)]
mod search_support;
#[cfg(test)]
mod space_level;
#[cfg(test)]
mod store;
#[cfg(test)]
mod upload_purge;
