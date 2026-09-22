//! The two export links the conversations page offers: the table the page
//! shows (with its breakdown) and the full-record document of the same set.
//! Both carry the page's own filters, so a file answers the question the
//! page was asked.

use super::query::{ConversationAnalysisQuery, export_preset};

pub(super) const DATASET: &str = "analysis-conversations";

fn export_query(params: &ConversationAnalysisQuery) -> String {
    params
        .preserved(&["page", "since", "preset"])
        .iter()
        .map(|(k, v)| format!("{k}={}", urlencoding::encode(v)))
        .chain(std::iter::once(format!("preset={}", export_preset(params))))
        .collect::<Vec<_>>()
        .join("&")
}

pub(super) fn export_view(params: &ConversationAnalysisQuery) -> crate::export::ExportView {
    crate::export::ExportView::new(
        &[DATASET, "analysis-conversation-breakdown"],
        &export_query(params),
    )
}

pub(super) fn document_export_view(
    params: &ConversationAnalysisQuery,
) -> crate::export::document::DocumentExportView {
    crate::export::document::DocumentExportView::filtered("analysis", &export_query(params))
}
