//! Bounded field projections for arbitrary-width tables. `PostgreSQL` functions
//! accept at most 100 arguments, so JSON objects are assembled in chunks.
use super::browse::literal;
use super::catalog::Column;
use super::quote_identifier;
pub(crate) const TRUNCATED: &str = " [display truncated]";

pub(crate) fn fields(columns: &[Column], limit: usize) -> String {
    let pairs: Vec<_> = columns.iter().map(|c| {
        let field = format!("t.{}::text",quote_identifier(&c.name));
        format!("{}, left({field},{limit}) || CASE WHEN length({field}) > {limit} THEN {} ELSE '' END",
            literal(&c.name),literal(TRUNCATED))
    }).collect();
    if pairs.is_empty() {
        return "'{}'::jsonb".into();
    }
    pairs
        .chunks(40)
        .map(|chunk| format!("jsonb_build_object({})", chunk.join(",")))
        .collect::<Vec<_>>()
        .join(" || ")
}
