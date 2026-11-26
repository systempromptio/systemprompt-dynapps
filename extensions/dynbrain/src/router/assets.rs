//! The file routes.
//!
//! A file is served with the type read from its bytes. Only raster images
//! are `inline`; everything else, SVG and HTML included, is an `attachment`.
//! Every file goes out with `X-Content-Type-Options: nosniff` and
//! `Content-Security-Policy: sandbox`, so a browser never runs what it holds,
//! and no answer is cached by a shared cache.
//!
//! On a proxy instance the file operations go to the source, its upload URL
//! takes the bytes, and `GET assets/{id}` redirects to the source's own
//! short-lived link for the person; no byte is kept here.

use axum::Json;
use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Redirect, Response};
// JSON: an operation's reply, as the Brain answered it.
use serde_json::Value;

use super::error::HttpFault;
use super::extract::{Id, Params, Payload, Token};
use super::relay::relay;
use super::state::BrainState;
use crate::assets::link::SignedLink;
use crate::assets::sniff;
use crate::auth::Caller;
use crate::brain::assets::{
    AssetFile, AssetLinkArgs, AssetListArgs, AssetUploadArgs, Received, RequestUploadArgs,
};
use crate::brain::fault::BrainFault;
use crate::brain::operation::Operation;
use crate::ids::{ArticleId, AssetId, UploadId};

pub async fn upload_url(
    caller: Caller,
    State(state): State<BrainState>,
    Id(id): Id,
    Payload(mut args): Payload<RequestUploadArgs>,
) -> Result<(StatusCode, Json<Value>), HttpFault> {
    args.article_id = Some(ArticleId(id));
    args.path = None;
    let ticket = relay(&state, &caller, Operation::RequestUpload, &args).await?;
    Ok((StatusCode::CREATED, ticket))
}

pub async fn receive(
    State(state): State<BrainState>,
    Token(upload): Token,
    Params(link): Params<SignedLink>,
    request: Request,
) -> Result<(StatusCode, Json<Received>), HttpFault> {
    if is_form(request.headers()) {
        return Err(HttpFault::BadRequest(
            "send the file as the raw request body, not as a form",
        ));
    }
    let limit = state.brain.upload_limit();
    // Why: a Brain that keeps no files, a proxy one, has no upload URL here.
    if limit == 0 {
        return Err(HttpFault::NotFound);
    }
    let bytes = axum::body::to_bytes(
        request.into_body(),
        usize::try_from(limit).unwrap_or(usize::MAX),
    )
    .await
    .map_err(|error| {
        tracing::debug!(%error, "dynbrain: upload body refused");
        HttpFault::Brain(BrainFault::TooLarge(limit))
    })?;
    let received = state
        .brain
        .receive_upload(&UploadId(upload), &link, &bytes)
        .await?;
    Ok((StatusCode::CREATED, Json(received)))
}

pub async fn attach(
    caller: Caller,
    State(state): State<BrainState>,
    Id(id): Id,
    Payload(mut args): Payload<AssetUploadArgs>,
) -> Result<(StatusCode, Json<Value>), HttpFault> {
    args.article_id = Some(ArticleId(id));
    args.path = None;
    let attached = relay(&state, &caller, Operation::AssetUpload, &args).await?;
    Ok((StatusCode::CREATED, attached))
}

pub async fn list(
    caller: Caller,
    State(state): State<BrainState>,
    Id(id): Id,
    Params(mut args): Params<AssetListArgs>,
) -> Result<Json<Value>, HttpFault> {
    args.article_id = Some(ArticleId(id));
    args.path = None;
    relay(&state, &caller, Operation::AssetList, &args).await
}

pub async fn link(
    caller: Caller,
    State(state): State<BrainState>,
    Id(id): Id,
    Params(mut args): Params<AssetLinkArgs>,
) -> Result<Json<Value>, HttpFault> {
    args.attachment_id = Some(AssetId(id));
    relay(&state, &caller, Operation::AssetLink, &args).await
}

pub async fn content(
    caller: Caller,
    State(state): State<BrainState>,
    Id(id): Id,
) -> Result<Response, HttpFault> {
    if let Some(url) = state
        .brain
        .asset_redirect(&caller.0, &caller.context(), AssetId(id))
        .await?
    {
        let mut response = Redirect::temporary(&url).into_response();
        response.headers_mut().insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("private, no-store"),
        );
        return Ok(response);
    }
    let file = state.brain.open_asset(&caller.0, AssetId(id)).await?;
    Ok(served(file))
}

pub async fn raw(
    State(state): State<BrainState>,
    Id(id): Id,
    Params(link): Params<SignedLink>,
) -> Result<Response, HttpFault> {
    let file = state.brain.open_signed(AssetId(id), &link).await?;
    Ok(served(file))
}

fn served(file: AssetFile) -> Response {
    let disposition = if sniff::is_inline(&file.mimetype) {
        "inline"
    } else {
        "attachment"
    };
    let mut response = Body::from_stream(file.body).into_response();
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&file.mimetype)
            .unwrap_or_else(|_invalid| HeaderValue::from_static("application/octet-stream")),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!(
            "{disposition}; filename=\"{}\"",
            header_name(&file.name)
        ))
        .unwrap_or_else(|_invalid| HeaderValue::from_static("attachment")),
    );
    headers.insert(header::CONTENT_LENGTH, HeaderValue::from(file.size_bytes));
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("sandbox"),
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
    response
}

fn is_form(headers: &HeaderMap) -> bool {
    headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.to_ascii_lowercase().starts_with("multipart/"))
}

fn header_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | ' ') {
                c
            } else {
                '_'
            }
        })
        .collect()
}
