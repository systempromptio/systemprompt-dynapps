//! Errors the workflow state machine and receipt signer report.
//!
//! `Validation` carries the message a reviewer sees; the remaining variants
//! name a specific gate or key failure so a caller can map them to distinct
//! responses instead of matching on text.

#[derive(Debug, thiserror::Error)]
pub enum RequirementsError {
    #[error("{0}")]
    Validation(String),
    #[error("Missing {phase}")]
    MissingPhase { phase: &'static str },
    #[error("Validate {phase} first")]
    NotValidated { phase: &'static str },
    #[error("{phase} has failed or stale validation")]
    StaleValidation { phase: &'static str },
    #[error("Select the publication destination before review")]
    DestinationRequired,
    #[error("Human review is required")]
    ReviewRequired,
    #[error("Approval is stale or rejected")]
    ApprovalStale,
    #[error("Awaiting human approval")]
    AwaitingApproval,
    #[error("Request review first")]
    ReviewNotRequested,
    #[error("Review is stale or already decided")]
    ReviewDecided,
    #[error("Workflow signing seed must be 32 bytes (got {0} bytes)")]
    InvalidSeed(usize),
    #[error("Invalid public key (got {0} bytes)")]
    InvalidPublicKey(usize),
    #[error("Wrong receipt key")]
    WrongReceiptKey,
    #[error("Receipt signature is invalid")]
    Signature(#[from] ed25519_dalek::SignatureError),
    #[error("Receipt encoding is invalid")]
    Base64(#[from] base64::DecodeError),
    #[error("Payload cannot be canonicalised")]
    Canonicalize(#[from] serde_json::Error),
}

pub type Result<T, E = RequirementsError> = core::result::Result<T, E>;
