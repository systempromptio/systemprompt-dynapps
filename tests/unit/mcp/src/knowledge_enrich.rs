//! Contextual enrichment (FR-PROC-4): the prefix the model adds makes the
//! stored chunk self-contained, while the line pointer is computed against
//! the original text and still resolves to the cited span; a failing or
//! empty model reply leaves the chunk unchanged.

use crate::knowledge_fake_model::FakeModel;
use systemprompt_mcp_knowledge_bank::ingest::chunker::find_line_range;
use systemprompt_mcp_knowledge_bank::ingest::enrich::{DocumentContext, enrich_chunk};

const BODY: &str = "# Checkout\n\n## Rounding\n\nTotals are rounded half to even.\n\n## Vault\n\nCards are tokenised at save time after consent.\n";

#[tokio::test]
async fn an_enriched_chunk_keeps_its_line_pointer_into_the_original() {
    let context = DocumentContext::build("Checkout", BODY);
    assert_eq!(context.outline, "Checkout\n  Rounding\n  Vault");
    let chunk = "Cards are tokenised at save time after consent.";
    let model = FakeModel::replying(&["This chunk is the Vault section of the Checkout page."]);

    let prefix = enrich_chunk(&model, &context, chunk)
        .await
        .expect("the model's sentences become the prefix");
    let stored = format!("{prefix}\n\n{chunk}");
    assert!(stored.starts_with("This chunk is the Vault section"));
    assert!(
        stored.ends_with(chunk),
        "the original text is never rewritten"
    );

    let (lines, _) = find_line_range(BODY, chunk, 0).expect("the chunk is found in the original");
    assert_eq!(
        lines, "9-9",
        "the pointer is to the original span, not the enriched text"
    );
    assert!(
        find_line_range(BODY, &stored, 0).is_none(),
        "the enriched text is not what the pointer is computed from"
    );
}

#[tokio::test]
async fn a_failing_or_empty_enrichment_leaves_the_chunk_unchanged() {
    let context = DocumentContext::build("Checkout", BODY);
    let empty = FakeModel::replying(&["   "]);
    assert_eq!(enrich_chunk(&empty, &context, "chunk").await, None);
    let exhausted = FakeModel::replying(&[]);
    assert_eq!(
        enrich_chunk(&exhausted, &context, "chunk").await,
        None,
        "a model failure degrades to the bare chunk (NFR-DEGRADE-1)"
    );
}
