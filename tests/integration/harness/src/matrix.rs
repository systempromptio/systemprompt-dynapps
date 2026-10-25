//! Every harness fixture × every catalogue lane (`provider/endpoint`), each
//! cell validated.

use crate::dispatch::dispatch;
use crate::fixtures::{HARNESSES, captured, load};
use crate::providers::catalogue;
use crate::validate;

#[tokio::test]
async fn every_harness_reaches_every_provider_well_formed() {
    let lanes = catalogue();
    let mut cells = 0;
    let mut failures: Vec<String> = Vec::new();
    for harness in HARNESSES {
        let fixture = load(harness);
        for lane in &lanes {
            cells += 1;
            let outcome = match dispatch(&fixture, lane).await {
                Ok(captured) => validate::all(&fixture, lane, &captured),
                Err(e) => Err(e),
            };
            if let Err(e) = outcome {
                failures.push(format!("{} -> {}: {e}", fixture.name, lane.name));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {cells} cells failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
    assert_eq!(cells, HARNESSES.len() * lanes.len());
}

#[tokio::test]
async fn every_captured_cli_request_reaches_every_provider_well_formed() {
    let lanes = catalogue();
    let fixtures = captured();
    assert!(
        fixtures.len() >= 18,
        "expected the recorded claude/opencode/codex captures"
    );
    let mut failures: Vec<String> = Vec::new();
    for fixture in &fixtures {
        for lane in &lanes {
            let outcome = match dispatch(fixture, lane).await {
                Ok(captured) => validate::all(fixture, lane, &captured),
                Err(e) => Err(e),
            };
            if let Err(e) = outcome {
                failures.push(format!("{} -> {}: {e}", fixture.name, lane.name));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} cells failed:\n{}",
        failures.len(),
        fixtures.len() * lanes.len(),
        failures.join("\n")
    );
}

#[test]
fn catalogue_covers_every_wire() {
    use systemprompt::wire::WireProtocol;
    let wires: Vec<WireProtocol> = catalogue().iter().map(|lane| lane.wire).collect();
    for wire in [
        WireProtocol::Anthropic,
        WireProtocol::OpenAiChat,
        WireProtocol::OpenAiResponses,
        WireProtocol::Gemini,
    ] {
        assert!(wires.contains(&wire), "no provider speaks {wire:?}");
    }
}
