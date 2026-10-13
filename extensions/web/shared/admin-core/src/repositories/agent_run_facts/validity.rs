//! Whether a run's structured output is what its request asked for.
//!
//! The request's `output_schema` is relayed to the host verbatim, so the
//! only honest check is the same JSON Schema validation the host should
//! have done. A schema that does not compile counts as a failed check
//! rather than an absent one: the caller asked for a shape and did not get
//! a verifiable one. `None` when the request set no schema or the run
//! produced no output.

// JSON: both sides are protocol payloads — the caller's schema and the
// host's answer — read straight from JSONB.
use serde_json::Value;

#[must_use]
pub fn output_schema_valid(schema: Option<&Value>, output: Option<&Value>) -> Option<bool> {
    let schema = schema.filter(|s| !s.is_null())?;
    let output = output?;
    Some(jsonschema::validator_for(schema).is_ok_and(|validator| validator.is_valid(output)))
}
