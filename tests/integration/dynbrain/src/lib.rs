//! The `dynbrain` repository against Postgres, in a throwaway database with
//! the full extension schema:
//!
//! - spaces: create, read, list, edit, and the languages a space refuses
//! - the article tree: order, children, and creating under a parent
//! - variants: every write bumps the revision and appends it, a language the
//!   space does not list or a property the schema refuses writes nothing, and
//!   revisions refuse updates
//! - permissions: nobody without a grant reaches an article but its owner and
//!   administrators; a grant reaches the subtree and revoking it takes it back;
//!   `inherit = false` cuts inheritance; a group grant reaches its live members
//!   and a role grant the users holding the role; an inactive account reaches
//!   nothing
//! - moving: the subtree's closure is rewritten, so what it inherited from its
//!   old place is gone; moves below itself or into another space fail
//! - trash: an article goes with its subtree and comes back with it

#[cfg(test)]
mod fixture;
#[cfg(test)]
mod moves;
#[cfg(test)]
mod permissions;
#[cfg(test)]
mod store;
