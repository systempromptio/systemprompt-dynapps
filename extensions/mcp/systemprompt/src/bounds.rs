//! Bounds the size of a passthrough CLI result before it goes on the wire.
//!
//! An `infra logs audit` of a long session is a 100-400 KB single line. Sent
//! whole it exceeds what a host will hand a model as a tool result, and the
//! host then spills it to a file the model cannot read back. A bounded result
//! with a hint that names the narrower query is something the model can act
//! on; a spilled file is not.

use systemprompt::models::artifacts::{CliArtifact, TextArtifact};
use systemprompt_mcp_shared::truncate_on_char_boundary;

pub const MAX_RESULT_BYTES: usize = 64 * 1024;

const NARROW_HINT: &str = "Narrow the query: add --since/--until/--user or a smaller --limit, page \
                           with --before <cursor>, or use the typed tools (request_log, \
                           conversation_audit, usage_by_user, users), which page on their own.";

/// The bounded artifact and whether anything was cut.
#[derive(Debug)]
pub struct Bounded {
    pub artifact: CliArtifact,
    pub truncated: bool,
    // Why: (kept, received) for a table so the summary can say what was cut;
    // other shapes have no row count to report.
    pub rows: Option<(usize, usize)>,
}

fn serialized_len(artifact: &CliArtifact) -> usize {
    serde_json::to_vec(artifact).map_or(0, |b| b.len())
}

// Why: a table loses trailing rows (the newest survive); anything else is
// serialized and truncated as text. An artifact that fits is untouched.
#[must_use]
pub fn bound_artifact(artifact: CliArtifact, command: &str) -> Bounded {
    if serialized_len(&artifact) <= MAX_RESULT_BYTES {
        return Bounded {
            artifact,
            truncated: false,
            rows: None,
        };
    }
    match artifact {
        CliArtifact::Table { mut artifact } => {
            let received = artifact.items.len();
            while !artifact.items.is_empty()
                && serialized_len(&CliArtifact::Table {
                    artifact: artifact.clone(),
                }) > MAX_RESULT_BYTES
            {
                let keep = artifact.items.len() * 3 / 4;
                artifact.items.truncate(keep);
            }
            let kept = artifact.items.len();
            let title = format!(
                "{} — truncated to {kept} of {received} rows. {NARROW_HINT}",
                artifact.title.clone().unwrap_or_else(|| command.to_owned())
            );
            Bounded {
                artifact: CliArtifact::Table {
                    artifact: artifact.with_title(title),
                },
                truncated: true,
                rows: Some((kept, received)),
            }
        },
        CliArtifact::Text { artifact } => Bounded {
            artifact: CliArtifact::text(
                TextArtifact::new(truncate_on_char_boundary(
                    &artifact.content,
                    MAX_RESULT_BYTES,
                ))
                .with_title(format!("Output of `{command}` (truncated). {NARROW_HINT}")),
            ),
            truncated: true,
            rows: None,
        },
        other => {
            let text = match serde_json::to_string(&other) {
                Ok(text) => text,
                Err(error) => format!("unserializable artifact: {error}"),
            };
            let body = truncate_on_char_boundary(&text, MAX_RESULT_BYTES);
            Bounded {
                artifact: CliArtifact::text(
                    TextArtifact::new(body)
                        .with_title(format!("Output of `{command}` (truncated). {NARROW_HINT}")),
                ),
                truncated: true,
                rows: None,
            }
        },
    }
}
