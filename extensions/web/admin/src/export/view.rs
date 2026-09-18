//! What a page hands its template so the export button and dialog can render.

use serde::Serialize;

use super::format::Format;
use super::model::{Column, Window};
use super::registry;

#[derive(Debug, Serialize)]
pub(crate) struct ExportFormatView {
    value: &'static str,
    label: &'static str,
}

#[derive(Debug, Serialize)]
pub(crate) struct ExportColumnGroupView {
    name: &'static str,
    columns: Vec<Column>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ExportDatasetView {
    id: &'static str,
    title: &'static str,
    description: &'static str,
    window: Window,
    columns: &'static [Column],
    groups: Vec<ExportColumnGroupView>,
}

// Why: groups keep the dataset's own column order inside each heading and
// appear in the order their first column does, so the checklist reads like
// the table it exports.
fn group_columns(columns: &'static [Column]) -> Vec<ExportColumnGroupView> {
    let mut groups: Vec<ExportColumnGroupView> = Vec::new();
    for column in columns {
        match groups.iter_mut().find(|g| g.name == column.group) {
            Some(group) => group.columns.push(*column),
            None => groups.push(ExportColumnGroupView {
                name: column.group,
                columns: vec![*column],
            }),
        }
    }
    groups
}

// Why: the button is a plain link that works without JavaScript — default
// columns, the page's own window, CSV — and the dialog reads the rest from
// this same context to offer everything else.
#[derive(Debug, Serialize)]
pub(crate) struct ExportView {
    href: String,
    query: String,
    multiple: bool,
    datasets: Vec<ExportDatasetView>,
    formats: Vec<ExportFormatView>,
}

impl ExportView {
    pub(crate) fn new(ids: &[&str], query: &str) -> Self {
        let datasets: Vec<ExportDatasetView> = ids
            .iter()
            .filter_map(|id| registry::find(id))
            .map(|d| ExportDatasetView {
                id: d.id(),
                title: d.title(),
                description: d.description(),
                window: d.window(),
                columns: d.columns(),
                groups: group_columns(d.columns()),
            })
            .collect();
        let first = datasets.first().map_or("", |d| d.id);
        let query = query.trim_start_matches('?').trim_matches('&').to_owned();
        let href = if query.is_empty() {
            format!("/admin/export/{first}?format=csv")
        } else {
            format!("/admin/export/{first}?{query}&format=csv")
        };
        Self {
            href,
            query,
            multiple: datasets.len() > 1,
            datasets,
            formats: Format::ALL
                .iter()
                .map(|f| ExportFormatView {
                    value: f.param(),
                    label: f.label(),
                })
                .collect(),
        }
    }

    pub(crate) fn single(id: &str, query: &str) -> Self {
        Self::new(&[id], query)
    }
}

// Why: every list page keeps its filters as `(name, value)` pairs; the export
// carries the same pairs so the file answers the question the page was asked.
pub(crate) fn query_string(pairs: &[(&str, Option<&str>)]) -> String {
    pairs
        .iter()
        .filter_map(|(name, value)| {
            value
                .filter(|v| !v.is_empty())
                .map(|v| format!("{name}={}", urlencoding::encode(v)))
        })
        .collect::<Vec<_>>()
        .join("&")
}
