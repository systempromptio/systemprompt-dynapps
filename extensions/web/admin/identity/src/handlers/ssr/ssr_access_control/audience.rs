//! The audience grid: every entity down the side, every role, group and
//! project across the top, and in each cell what the resolver decides for a
//! subject that holds exactly that one thing — decision first, with the band
//! that decided as the small word underneath.
//!
//! It is the picture the rule list cannot give: who reaches what, at a
//! glance, straight from the enforcement path rather than from a reading of
//! the rules. Cells are resolved in one batch so the page reads the rule
//! table once, however many columns it draws. Narrowing (`audience_filter`)
//! and the inspector for one subject or one entity (`audience_focus`) read
//! the same resolved cells, so a filtered grid and its inspector never
//! disagree.

use sqlx::PgPool;

use super::audience_filter::{self, Narrowed};
use super::audience_focus;
use super::audience_rows::{column_groups, legend, resolved_rows, row_groups};
use super::entities::AcQuery;
use super::view::{AcAudienceColumnView, AudienceGridView};
use systemprompt_web_admin_core::repositories::access_rules::rules::LedgerRuleRow;
use systemprompt_web_admin_core::repositories::entitlement::{
    MatrixSubject, SectionInput, group_subject, project_subject, resolve_subject_matrices,
    role_subject,
};
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::types::Role;

pub(super) const AUDIENCE_URL: &str = "/admin/access-control?tab=audience";

pub(super) struct Audience {
    pub subjects: Vec<MatrixSubject>,
    pub columns: Vec<AcAudienceColumnView>,
}

pub(super) fn subject_kind_label(kind: &str) -> &'static str {
    match kind {
        "role" => "Roles",
        "group" => "Groups",
        "project" => "Projects",
        _ => "Subjects",
    }
}

pub(super) fn audience_url(params: &[(&str, &str)]) -> String {
    let mut url = AUDIENCE_URL.to_owned();
    for (key, value) in params {
        url.push('&');
        url.push_str(key);
        url.push('=');
        url.push_str(&urlencoding::encode(value));
    }
    url
}

fn column(subject: &str, label: String, kind: &'static str) -> AcAudienceColumnView {
    let id = format!("{kind}:{subject}");
    AcAudienceColumnView {
        focus_url: audience_url(&[("subject", &id)]),
        id,
        subject: subject.to_owned(),
        label,
        kind,
        kind_label: subject_kind_label(kind),
        is_focused: false,
        allowed: 0,
        denied: 0,
    }
}

pub(super) async fn audience(pool: &PgPool, faults: &mut Faults) -> Audience {
    let mut subjects = Vec::new();
    let mut columns = Vec::new();
    for role in Role::ALL {
        subjects.push(role_subject(role.as_str()));
        columns.push(column(role.as_str(), role.as_str().to_owned(), "role"));
    }
    let groups = faults.take(
        "groups",
        systemprompt_web_admin_core::repositories::scope::containers::list_group_summaries(pool)
            .await,
    );
    for group in groups {
        subjects.push(group_subject(group.id.as_str()));
        columns.push(column(group.id.as_str(), group.name, "group"));
    }
    let projects = faults.take(
        "projects",
        systemprompt_web_admin_core::repositories::scope::containers::list_project_summaries(pool)
            .await,
    );
    for project in projects {
        subjects.push(project_subject(project.id.as_str()));
        columns.push(column(project.id.as_str(), project.name, "project"));
    }
    Audience { subjects, columns }
}

#[derive(Clone, Copy)]
pub(super) struct GridRead<'a> {
    pub(super) services: &'a systemprompt_web_admin_core::extension_services::ExtensionServices,
    pub(super) query: &'a AcQuery,
    pub(super) ledger: &'a [LedgerRuleRow],
}

pub(super) async fn grid(
    pool: &PgPool,
    audience: Audience,
    sections: &[SectionInput],
    read: GridRead<'_>,
    faults: &mut Faults,
) -> AudienceGridView {
    let GridRead {
        services,
        query,
        ledger,
    } = read;
    if audience.subjects.is_empty() || sections.is_empty() {
        return AudienceGridView::default();
    }
    let resolved = faults.take(
        "audience grid",
        resolve_subject_matrices(pool, services, &audience.subjects, sections).await,
    );
    if resolved.len() != audience.subjects.len() {
        return AudienceGridView::default();
    }
    let all_rows = resolved_rows(&resolved, &audience.columns, sections);
    let filters = audience_filter::filters(&audience.columns, &all_rows, query);
    let Narrowed { columns, rows } = audience_filter::narrow(audience.columns, all_rows, query);
    let focus = audience_focus::build(&columns, &rows, query, ledger);
    let kpis = audience_focus::kpis(&columns, &rows, query);
    AudienceGridView {
        column_groups: column_groups(&columns),
        has_rows: !rows.is_empty(),
        resolved: true,
        total_rows: rows.len(),
        total_columns: columns.len(),
        colspan: columns.len() + 1,
        row_groups: row_groups(rows),
        columns,
        kpis,
        legend: legend(),
        filters,
        focus,
    }
}
