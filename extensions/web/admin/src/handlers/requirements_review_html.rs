//! HTML for the browser review screen of a requirements bundle.

use systemprompt_requirements_model::model::{Phase, Review, StoredWorkflow, Workflow};
use uuid::Uuid;

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn changes_since(previous: &Workflow, current: &Workflow) -> String {
    let mut body =
        String::from("<details><summary>Changes since the previous revision</summary><ul>");
    for phase in Phase::ALL {
        let before = previous.artifacts.get(&phase);
        let after = current.artifacts.get(&phase);
        if before.map(|a| &a.hash) == after.map(|a| &a.hash) {
            continue;
        }
        body.push_str(&format!(
            "<li>{}: {}</li>",
            phase.name(),
            if after.is_some() {
                "content changed"
            } else {
                "invalidated and removed"
            }
        ));
        if let Some(before) = before {
            for (path, content) in &before.files {
                body.push_str(&format!(
                    "<li>Previous {}<pre>{}</pre></li>",
                    escape(path),
                    escape(content)
                ));
            }
        }
    }
    body.push_str("</ul></details>");
    body
}

fn artifacts_body(workflow: &Workflow) -> String {
    let mut body = String::new();
    for (phase, artifact) in &workflow.artifacts {
        body.push_str(&format!("<section><h2>{}</h2>", phase.name()));
        for (path, content) in &artifact.files {
            body.push_str(&format!(
                "<h3>{}</h3><pre>{}</pre>",
                escape(path),
                escape(content)
            ));
        }
        if let Some(validation) = &artifact.validation {
            body.push_str(&format!(
                "<details><summary>Validation: {}</summary><pre>{}</pre></details>",
                if validation.passed {
                    "passed"
                } else {
                    "failed"
                },
                escape(&validation.report)
            ));
        }
        body.push_str("</section>");
    }
    body.push_str("<details><summary>Source evidence</summary>");
    for source in &workflow.evidence {
        body.push_str(&format!(
            "<h3>{}</h3><p>{}</p><pre>{}</pre>",
            escape(&source.title),
            escape(&source.uri),
            escape(&source.body)
        ));
    }
    body.push_str("</details>");
    body
}

fn gate_body(raw: &str, nonce: Option<Uuid>, revision: i64, gate: &str, review: &Review) -> String {
    if review.receipt.is_none() && review.rejection.is_none() {
        let Some(nonce) = nonce else {
            return format!(
                "<p>{gate}: awaiting review. Open this page in the signed-in browser to record a decision.</p>"
            );
        };
        return format!(
            "<form method=post action='/admin/requirements/{raw}/decision'><h2>{gate} approval</h2><input type=hidden name=nonce value='{nonce}'><input type=hidden name=revision value='{revision}'><input type=hidden name=gate value='{gate}'><label>Review note <textarea name=note maxlength=2000 required></textarea></label><button name=decision value=approve>Approve this exact bundle</button><button name=decision value=reject>Request changes</button></form>"
        );
    }
    format!(
        "<p>{gate}: {}</p>",
        if review.receipt.is_some() {
            "approved"
        } else {
            "changes requested"
        }
    )
}

pub(crate) fn review_body(
    raw: &str,
    nonce: Option<Uuid>,
    stored: &StoredWorkflow,
    previous: Option<&Workflow>,
) -> String {
    let mut body = format!(
        "<!doctype html><html lang=en><meta charset=utf-8><meta name=viewport content='width=device-width,initial-scale=1'><title>Requirements review</title><link rel=stylesheet href='/files/css/requirements-review.css'><main><h1>{}</h1><p>Review revision {}. Your decision binds this exact evidence and artifact bundle.</p>",
        escape(&stored.workflow.name),
        stored.revision
    );
    body.push_str(&format!("<p><a href='/admin/requirements/{raw}/download'>Download evidence, artifacts and validation receipts (JSON)</a></p>"));
    if let Some(previous) = previous {
        body.push_str(&changes_since(previous, &stored.workflow));
    }
    body.push_str(&format!(
        "<h2>Delivery scope</h2><pre>{}</pre>",
        escape(&stored.workflow.delivery_scope)
    ));
    if let Some(destination) = &stored.workflow.destination {
        body.push_str(&format!("<h2>Publication destination</h2><p>Cloud: {} · Space: {} · Parent: {}</p><p>Title: {}</p>", escape(&destination.cloud_id), escape(&destination.space_id), escape(&destination.parent_id), escape(&destination.title)));
    }
    body.push_str(&artifacts_body(&stored.workflow));
    for (gate, review) in [
        ("context", &stored.workflow.context_review),
        ("publication", &stored.workflow.publication_review),
    ] {
        if let Some(review) = review {
            body.push_str(&gate_body(raw, nonce, stored.revision, gate, review));
        }
    }
    body.push_str("</main></html>");
    body
}
