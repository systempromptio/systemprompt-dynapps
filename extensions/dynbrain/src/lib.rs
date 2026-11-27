//! The Brain's native store, a systemprompt extension.
//!
//! Documentation lives in spaces. A space holds a tree of articles; an
//! article has one variant per language (title, description, Markdown body,
//! free front matter and typed property values) and every write of a variant
//! appends a revision. Permissions are grants of `none`, `read`, `write` or
//! `manage` to a user, a role or a group, on a space or on an article. An
//! article inherits its parent's (or its space's) access unless it turns
//! `inherit_access` off, and its own grants add to or replace what it
//! inherits; the resolved result is kept in the access closure
//! (`dynbrain_access`), the one permission source retrieval reads.
//!
//! This crate holds the schema (Postgres with pgvector, including the
//! retrieval tables later phases fill) and [`BrainRepository`], the reads
//! and writes over it. Inheritance itself is the pure [`access::closure`].
//! On top of it, [`brain::Brain`] is the trait of the operations, with
//! [`brain::native::NativeBrain`] as its implementation over this store, and
//! the extension mounts `/api/dynbrain/v1` (see [`router`]) for the desktop
//! and mobile JWTs. Articles carry files ([`assets`]): the bytes live in
//! core's file storage, Markdown names them `asset://<id>`, and readers get
//! short-lived signed URLs. Writing a variant queues it for indexing, and a
//! scheduler job ([`ingest`]) cuts it into chunks, situates each one and
//! embeds it, the way dynbrain does. [`search`] runs dynbrain's hybrid
//! retrieval over that index, with the reader's access inside its SQL, and
//! assembles the context an answer is written from without calling a model.
//! An instance in proxy mode ([`config`], [`brain::proxy`]) answers every
//! operation from the tools of another MCP server instead, as the person who
//! asks.

pub mod access;
pub mod assets;
pub mod auth;
pub mod brain;
pub mod config;
pub mod console;
pub mod content;
pub mod error;
pub mod extension;
pub mod ids;
pub mod ingest;
pub mod language;
pub mod model;
pub mod properties;
pub mod repository;
pub mod router;
pub mod schema;
pub mod search;

pub use error::BrainError;
pub use extension::DynbrainExtension;
pub use repository::BrainRepository;

systemprompt::extension::register_extension!(DynbrainExtension);
