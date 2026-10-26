//! Callback exchange and verification report a safe stage and correlation
//! reference.

use super::{live_user, store_pending_verification, store_verified};
use crate::services::connector_oauth::{self as oauth, Grant};
use axum::http::HeaderMap;
use axum::http::header::CACHE_CONTROL;
use axum::response::{IntoResponse, Redirect, Response};
use oauth::callback_notice;
use oauth::failure::{FailureKind, FlowFailure};
use sqlx::PgPool;
use std::time::Instant;
use systemprompt::identifiers::UserId;
use systemprompt::security::keys::SigningKeys;
use systemprompt_web_admin_core::connections::report::VerificationReport;
use systemprompt_web_admin_core::error::AdminResult;
use tracing::Instrument;
use uuid::Uuid;

pub(super) struct CallbackSession<'a> {
    pub pool: &'a PgPool,
    pub keys: &'a SigningKeys,
    pub user: &'a UserId,
    pub headers: &'a HeaderMap,
}

pub(super) async fn finish(
    services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    session: CallbackSession<'_>,
    grant: Grant,
    code: &str,
) -> AdminResult<Response> {
    let reference = Uuid::new_v4();
    let span = tracing::info_span!("connector_callback", user_id = %session.user, provider = grant.provider.slug(), trace_id = %reference);
    complete(services, session, grant, code, reference)
        .instrument(span)
        .await
}

async fn complete(
    services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    session: CallbackSession<'_>,
    mut grant: Grant,
    code: &str,
    reference: Uuid,
) -> AdminResult<Response> {
    let CallbackSession {
        pool,
        keys,
        user,
        headers,
    } = session;
    let mut report = VerificationReport::for_provider(grant.provider.slug());
    let started = Instant::now();
    let exchange = oauth::exchange_classified(services, &mut grant, code).await;
    let outcome = match exchange {
        Ok(()) => {
            report.record("token", started, &Ok("Authorization exchanged".into()));
            oauth::verify::verify_reporting(services, &mut grant, &oauth::client()?, &mut report)
                .await
                .map_err(FlowFailure::verification)
        },
        Err(failure) => {
            report.record("token", started, &Err(FlowFailure::new(failure.kind).error));
            Err(failure)
        },
    };
    let actor = live_user(pool, keys, headers, false).await?;
    if let Err(failure) = outcome {
        if failure.kind != FailureKind::GrantRejected && grant.refresh_token.is_some() {
            store_pending_verification(pool, user, &grant).await?;
        }
        let stage = report
            .steps
            .iter()
            .find(|step| !step.ok)
            .map_or("token", |step| step.step);
        tracing::warn!(target: "connector_oauth", user_id = %user, provider = grant.provider.slug(), trace_id = %reference,
            stage, classification = failure.kind.code(), "connector callback failed");
        let destination =
            callback_notice::redirect(&grant.provider, stage, failure.kind, reference);
        return Ok(([(CACHE_CONTROL, "no-store")], Redirect::to(&destination)).into_response());
    }
    store_verified(pool, user, &grant).await?;
    tracing::info!(user_id = %user, provider = grant.provider.slug(), trace_id = %reference, "connector_connected");
    let destination = if actor.user_id == *user {
        format!("/admin/connectors#connector-{}", grant.provider.slug())
    } else {
        format!("/admin/identity/service-accounts/{user}")
    };
    Ok(([(CACHE_CONTROL, "no-store")], Redirect::to(&destination)).into_response())
}
