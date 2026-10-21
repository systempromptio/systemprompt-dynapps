//! Every harness fixture × every catalogue provider, each cell validated.

use crate::dispatch::dispatch;
use crate::fixtures::{HARNESSES, load};
use crate::providers::catalogue;
use crate::validate;

#[tokio::test]
async fn every_harness_reaches_every_provider_well_formed() {
    let providers = catalogue();
    let mut cells = 0;
    let mut failures: Vec<String> = Vec::new();
    for harness in HARNESSES {
        let fixture = load(harness);
        for provider in &providers {
            cells += 1;
            let outcome = match dispatch(&fixture, provider).await {
                Ok(captured) => validate::all(&fixture, provider, &captured),
                Err(e) => Err(e),
            };
            if let Err(e) = outcome {
                failures.push(format!(
                    "{} -> {}: {e}",
                    fixture.name,
                    provider.name.as_str()
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {cells} cells failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
    assert_eq!(cells, HARNESSES.len() * providers.len());
}

#[test]
fn catalogue_covers_every_wire() {
    use systemprompt::wire::WireProtocol;
    let wires: Vec<WireProtocol> = catalogue().iter().map(|p| p.wire).collect();
    for wire in [
        WireProtocol::Anthropic,
        WireProtocol::OpenAiChat,
        WireProtocol::OpenAiResponses,
        WireProtocol::Gemini,
    ] {
        assert!(wires.contains(&wire), "no provider speaks {wire:?}");
    }
}
