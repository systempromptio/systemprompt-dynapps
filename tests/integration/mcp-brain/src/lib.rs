//! The `brain` server's tools over a real Brain in a throwaway database with
//! the full extension schema, run as the people the requests name:
//! - search and get answer as the person: what their grant does not allow does
//!   not exist for them, and a grant given later shows it
//! - writing needs write access, `trash` is soft and takes an article out of
//!   sight, and a tree is created and outlined
//! - a call that names no person is refused before any operation runs
//! - an unknown tool and bad arguments are refused; a filter naming an
//!   undeclared property is an empty result with its error
//! - the files: the first call of `asset_upload` is the request for the upload
//!   URL, an unknown `upload_id` is refused, and an instance without file
//!   storage says so

#[cfg(test)]
mod access;
#[cfg(test)]
mod files;
#[cfg(test)]
mod support;
#[cfg(test)]
mod writes;
