//! Row views for the skill page: one retained conversation with its session
//! link, spend and worded assessment.
use crate::repositories::analysis::retained::RetainedConversationRow;
use serde::Serialize;

pub(crate) fn dollars(microdollars: i64) -> String {
    crate::handlers::ssr::format::format_cost(microdollars)
}

#[derive(Debug, Clone, Serialize)]
pub struct RetainedConversationView {
    pub row: RetainedConversationRow,
    pub href: String,
    pub cost: String,
    pub host: String,
    pub first: String,
    pub last: String,
    pub assessment: String,
}

impl RetainedConversationView {
    #[must_use]
    pub fn new(row: RetainedConversationRow) -> Self {
        let assessment = match (row.assessment_status.as_deref(), row.score_millionths) {
            (Some("scored"), Some(score)) => format!("scored {}%", score / 10_000),
            (Some("scored"), None) => "scored".to_owned(),
            (Some(status), _) => status.to_owned(),
            (None, _) => "not assessed".to_owned(),
        };
        Self {
            href: format!(
                "/admin/sessions/{}",
                urlencoding::encode(row.session_id.as_str())
            ),
            cost: dollars(row.cost),
            host: row
                .host
                .clone()
                .unwrap_or_else(|| "unknown host".to_owned()),
            first: row.first_at.format("%Y-%m-%d %H:%M").to_string(),
            last: row.last_at.format("%Y-%m-%d %H:%M").to_string(),
            assessment,
            row,
        }
    }
}
