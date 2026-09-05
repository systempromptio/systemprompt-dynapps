//! This suite's throwaway database.
//!
//! The harness is `astound-test-common`: one implementation, one `CI` guard.
//! Databases and templates are named after the running test binary, so this
//! suite's never collide with another's.

pub use astound_test_common::TempDb;
