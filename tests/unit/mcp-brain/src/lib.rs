//! Unit tests for the `brain` MCP server's tool surface:
//! - the eighteen tools, with their names and no others: no prefix, none of the
//!   Git tools, no `version`
//! - every input schema is well formed, declares its required arguments and
//!   says what the `docs_*` tools said where agents were tested against it
//! - no description or schema names the source system or its product
//! - the annotations: reads are read only and `trash` alone is destructive
//! - no tool name matches the governance rule that blocks names holding
//!   `delete`, `drop`, `destroy`, `anonymise` or `erase`
//! - the faults of an operation as tool errors, and the empty reply of a filter
//!   that names an undeclared property

#[cfg(test)]
mod faults;
#[cfg(test)]
mod surface;
