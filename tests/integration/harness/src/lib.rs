//! Cross-harness compatibility matrix for the AI gateway.
//!
//! Every coding tool the gateway serves (Claude Code, Claude Desktop/Cowork,
//! `OpenCode`, Codex CLI, Cursor/Continue) is replayed from a sanitised request
//! fixture in `tests/fixtures/harness/` through the real inbound parser and the
//! real outbound adapter of every provider in `services/ai/providers.yaml`
//! (plus one synthetic provider per remaining wire), against a wiremock
//! upstream. The captured outbound request must pass a strict validator for
//! its wire — roles alternate, every tool call is answered by the next turn —
//! and carry the headers the provider declares. The `regressions` module pins
//! the production shapes that once broke: request `ba6f9208` (parallel
//! `OpenCode` tool results reaching `DeepSeek` as two user turns), an
//! interrupted tool call, and one session switching Claude → `DeepSeek` →
//! `OpenCode` Go → Claude.
//!
//! Gemini CLI is not a harness here: the gateway has no Gemini inbound
//! surface (no `generateContent` route, no `InboundWireProtocol::Gemini`), so
//! it cannot point at the gateway at all. The gemini wire is still covered
//! outbound, through the synthetic `harness-gemini` provider.

#[cfg(test)]
mod dispatch;
#[cfg(test)]
mod fixtures;
#[cfg(test)]
mod matrix;
#[cfg(test)]
mod providers;
#[cfg(test)]
mod regressions;
#[cfg(test)]
mod replay;
#[cfg(test)]
mod responses;
#[cfg(test)]
mod validate;
