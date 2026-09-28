//! One report row shaped into the report page's Handlebars context: headline,
//! assessment, theme cards, recommendations and the digest text. Pure — no I/O.

use crate::handlers::ssr::analysis_urls::ANALYSIS_CONVERSATIONS_URL;
use crate::handlers::ssr::format::{format_cost, format_token_total, local_time};
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories::analysis::reports::{
    AnalysisReportRow, ReportFindings, render_digest_text,
};

use super::view::{
    RecommendationView, ReportPageContext, ThemeView, assessment_icon, scope_label, severity_icon,
    status_tone, tiles,
};
use super::{REPORTS_URL, reports_help};

fn digest_summary(row: &AnalysisReportRow) -> String {
    let totals = &row.inputs.digest.totals;
    format!(
        "Reading {} conversations by {} people — {} skills, {} models, the best and worst judged, cost outliers and denials",
        totals.conversations,
        totals.people,
        row.inputs.digest.skills.len(),
        row.inputs.digest.models.len()
    )
}

fn theme_views(findings: Option<&ReportFindings>) -> Vec<ThemeView> {
    findings
        .map(|f| {
            f.themes
                .iter()
                .map(|t| ThemeView {
                    kind: t.kind.clone(),
                    severity: t.severity.as_str(),
                    icon: severity_icon(t.severity.as_str()),
                    theme_title: t.title.clone(),
                    detail: t.detail.clone(),
                    evidence: t.evidence.clone(),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn recommendation_views(findings: Option<&ReportFindings>) -> Vec<RecommendationView> {
    findings
        .map(|f| {
            f.recommendations
                .iter()
                .map(|r| RecommendationView {
                    action: r.action.clone(),
                    rationale: r.rationale.clone(),
                    priority: r.priority.as_str(),
                    priority_tone: r.priority.tone(),
                })
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn report_context(row: &AnalysisReportRow) -> ReportPageContext {
    let findings = row.findings.as_ref();
    let assessment = findings.map_or("", |f| f.assessment.as_str());
    ReportPageContext {
        page: "analysis-report",
        title: findings.map_or_else(|| scope_label(row), |f| f.headline.clone()),
        breadcrumbs: vec![
            BreadcrumbView::link("Analysis", ANALYSIS_CONVERSATIONS_URL),
            BreadcrumbView::link("Reports", REPORTS_URL),
            BreadcrumbView::current(scope_label(row)),
        ],
        report_id: row.id.clone(),
        digest_summary: digest_summary(row),
        status_url: format!("{REPORTS_URL}/{}/status", urlencoding::encode(&row.id)),
        scope_label: scope_label(row),
        scope_kind: row.scope_kind.clone(),
        window_display: format!(
            "{} → {}",
            row.window_start.format("%b %-d, %Y"),
            row.window_end.format("%b %-d, %Y")
        ),
        status: row.status.clone(),
        status_tone: status_tone(&row.status),
        pending: row.status == "pending",
        failed: row.status == "failed",
        headline: findings.map(|f| f.headline.clone()).unwrap_or_default(),
        assessment: assessment.to_owned(),
        assessment_label: findings.map_or("", |f| f.assessment.label()).to_owned(),
        assessment_tone: findings.map_or("muted", |f| f.assessment.tone()),
        assessment_icon: assessment_icon(assessment),
        generated_display: row.generated_at.map(local_time).unwrap_or_default(),
        model: row.model.clone().unwrap_or_default(),
        tokens_display: format_token_total(
            i64::from(row.input_tokens.unwrap_or(0)) + i64::from(row.output_tokens.unwrap_or(0)),
        ),
        cost_display: row.cost_microdollars.map_or_else(String::new, format_cost),
        requested_by: row.requested_by.clone(),
        error: row.last_error.clone().unwrap_or_default(),
        filter_query: row.inputs.filter_query.clone(),
        tiles: tiles(&row.inputs.digest.totals),
        themes: theme_views(findings),
        recommendations: recommendation_views(findings),
        digest_text: render_digest_text(&row.inputs.digest),
        regenerate_url: format!("{REPORTS_URL}/{}/regenerate", urlencoding::encode(&row.id)),
        back_url: REPORTS_URL,
        help: reports_help(),
    }
}
