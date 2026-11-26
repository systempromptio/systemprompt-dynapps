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
//! - search and answer answer `503 unavailable` until the retrieval phases
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
mod space_level;
#[cfg(test)]
mod store;
#[cfg(test)]
mod upload_purge;
