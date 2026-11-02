//! The consent page and the error page, rendered server-side with no script.
//!
//! Every value from a request, the account or the theme is HTML-escaped. The
//! pages forbid framing, scripts and every origin but this one, and allow a
//! form to post only here and to redirect only to the `dynwork:` callback
//! (browsers apply `form-action` to the redirect a form post answers with).
//! They are never cached and send no referrer.

use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{Html, IntoResponse, Response};

use super::branding::Branding;
use super::{DECISION_PATH, copy};

const CSP: &str = "default-src 'none'; style-src 'unsafe-inline'; img-src 'self'; \
     form-action 'self' dynwork:; frame-ancestors 'none'; base-uri 'none'";

pub fn escape(text: &str) -> String {
    text.chars()
        .fold(String::with_capacity(text.len()), |mut out, ch| {
            match ch {
                '&' => out.push_str("&amp;"),
                '<' => out.push_str("&lt;"),
                '>' => out.push_str("&gt;"),
                '"' => out.push_str("&quot;"),
                '\'' => out.push_str("&#39;"),
                other => out.push(other),
            }
            out
        })
}

fn fill(template: &str, device: &str, instance: &str, account: &str) -> String {
    escape(template)
        .replace("%device%", &format!("<strong>{}</strong>", escape(device)))
        .replace(
            "%instance%",
            &format!("<strong>{}</strong>", escape(instance)),
        )
        .replace(
            "%account%",
            &format!("<strong>{}</strong>", escape(account)),
        )
}

fn document(branding: &Branding, title: &str, body: &str) -> String {
    let logo = branding.logo.as_deref().map_or_else(String::new, |logo| {
        format!(
            r#"<img class="logo" src="{}" alt="{}">"#,
            escape(logo),
            escape(&branding.display_name)
        )
    });
    format!(
        r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>{title}</title><style>
body{{font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,sans-serif;margin:0;background:#f5f5f7;color:#111}}
main{{max-width:28rem;margin:3rem auto;padding:2rem;background:#fff;border-radius:12px;box-shadow:0 1px 3px rgba(0,0,0,.12)}}
.logo{{max-height:40px;margin-bottom:1rem}} .instance{{color:#555;font-size:.9rem}} .warning{{color:#555;font-size:.85rem}}
form{{display:inline}} button{{font-size:1rem;padding:.6rem 1.4rem;border-radius:8px;border:1px solid {color};margin:.5rem .5rem 0 0;cursor:pointer}}
.allow{{background:{color};color:#fff}} .deny{{background:#fff;color:{color}}}
</style></head><body><main>{logo}<p class="instance">{instance}</p>{body}</main></body></html>"#,
        title = escape(title),
        color = branding.theme_color,
        instance = escape(&branding.display_name),
    )
}

fn respond(status: StatusCode, html: String) -> Response {
    let mut response = (status, Html(html)).into_response();
    let headers = response.headers_mut();
    for (name, value) in [
        (header::CONTENT_SECURITY_POLICY, CSP),
        (header::CACHE_CONTROL, "no-store"),
        (header::REFERRER_POLICY, "no-referrer"),
        (header::X_FRAME_OPTIONS, "DENY"),
    ] {
        headers.insert(name, HeaderValue::from_static(value));
    }
    response
}

#[derive(Debug)]
pub struct Consent<'a> {
    pub branding: &'a Branding,
    pub device_name: &'a str,
    pub account: &'a str,
    pub request: &'a str,
}

pub fn consent(page: &Consent<'_>) -> Response {
    let instance = &page.branding.display_name;
    let form = |decision: &str, class: &str, label: &str| {
        format!(
            r#"<form method="post" action="{action}"><input type="hidden" name="request" value="{request}"><input type="hidden" name="decision" value="{decision}"><button type="submit" class="{class}">{label}</button></form>"#,
            action = DECISION_PATH,
            request = escape(page.request),
            label = escape(label),
        )
    };
    let body = format!(
        "<h1>{heading}</h1><p>{text}</p><p class=\"warning\">{warning}</p>{allow}{deny}",
        heading = fill(
            copy::CONSENT_HEADING,
            page.device_name,
            instance,
            page.account
        ),
        text = fill(copy::CONSENT_BODY, page.device_name, instance, page.account),
        warning = escape(copy::CONSENT_WARNING),
        allow = form("allow", "allow", copy::ALLOW),
        deny = form("deny", "deny", copy::DENY),
    );
    respond(
        StatusCode::OK,
        document(page.branding, copy::CONSENT_TITLE, &body),
    )
}

pub fn error(branding: &Branding, status: StatusCode, message: &str) -> Response {
    let body = format!(
        "<h1>{}</h1><p>{}</p>",
        escape(copy::ERROR_TITLE),
        escape(message)
    );
    respond(status, document(branding, copy::ERROR_TITLE, &body))
}
