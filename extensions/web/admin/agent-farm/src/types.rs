//! Rows read from systemprompt's own tables, mirroring the query columns.

/// The calls of one gateway session added up (the Cost page and the task
/// list read these for many sessions in one query).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SessionTotalRow {
    pub gateway_session: String,
    pub calls: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_creation_tokens: i64,
    pub cost_microdollars: i64,
}
