//! This suite's throwaway database.
//!
//! The harness is `dynapps-test-common`: one implementation, one `CI` guard.
//! Databases and templates are named after the running test binary, so this
//! suite's never collide with another's.

pub(crate) use dynapps_test_common::TempDb;
