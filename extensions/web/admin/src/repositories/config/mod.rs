//! Persistence for the configured policy surface — what an operator has
//! declared may happen.
//!
//! Split out of `governance`, which now owns only the record of what actually
//! did happen. The two answer different questions and change for different
//! reasons: an edit here is an operator changing intent, an insert there is
//! the enforcement path recording an outcome.
//!
//! Most of this is not Postgres at all. The gateway routes live in the
//! profile YAML and access-control
//! rules are declared in `services/access-control/rules.yaml` — read here,
//! seeded once at boot by `repositories::sync::boot`, and otherwise only
//! compared against the database. The gateway ACL and its after-the-fact
//! detector live in the governance crate beside the routes they guard.

pub mod groups_yaml_loader;
pub mod groups_yaml_types;
pub mod security_policy;
