---
title: "Content Safety, PII & Guardrails"
description: "What the gateway safety scanners and the governance chain detect on every request, what they would block under enforce, and why both currently run in warn mode."
author: "Astound Digital"
slug: "enterprise-safety-guardrails"
keywords: "safety, guardrails, pii, ssn, secrets, jailbreak, redaction, scanners, warn mode, block list, phi"
kind: "guide"
public: true
tags: ["enterprise", "safety", "admin"]
published_at: "2026-08-25"
updated_at: "2026-09-29"
after_reading_this:
  - "Know which categories the scanners detect and which would block under enforce"
  - "Understand that nothing on this plane refuses traffic today, and why"
  - "Distinguish buffered-response blocking from streamed audit-only scanning"
  - "Know what the console masks when it displays a transcript"
related_docs:
  - title: "Governance Modes: Warn and Enforce"
    url: "/documentation/governance-modes"
  - title: "MCP, Tool Governance & Distribution"
    url: "/documentation/enterprise-tool-governance"
  - title: "Audit Trail, Traceability & Observability"
    url: "/documentation/enterprise-audit-observability"
---

# Content Safety, PII & Guardrails

**TL;DR:** Every gateway request goes through the governance chain first and
the safety scanners after it. The scanners check the request and any buffered
response for jailbreak phrases, credentials and PII (email, card number, US
SSN, phone), and every finding is recorded. **Both planes have run in warn
mode since 2026-09-04**, so nothing on this page refuses traffic today. The
block lists stay configured so the warnings report can count what enforce
mode *would* refuse: SSNs in either direction, and secrets in responses.
Transcripts in the console mask credentials and SSNs when displayed. See
[Governance Modes](/documentation/governance-modes) for why warn mode is on
and how to switch it off.

## Two planes, one request

1. **Governance chain** (`services/governance/config.yaml`): scope check →
   secret scan → blocklist → rate limit. It runs on governed tool calls and on
   gateway prompts, and it covers credentials on the way in. Details are in
   [MCP, Tool Governance & Distribution](/documentation/enterprise-tool-governance).
2. **Safety scanners** (`services/gateway/policies.yaml`): run after the
   chain, on the request and on the response.

## What the scanners detect

| Category | Scanner | Direction | Would block under enforce |
|---|---|---|---|
| `jailbreak` | `heuristic`, with a phrase list pinned in `policies.yaml` ("ignore previous instructions", "act as dan", …) | both | no, audit-only since 2026-08-31 |
| `pii_email` | `heuristic` | both | no |
| `pii_credit_card` | `heuristic` | both | no, audit-only since 2026-08-31 |
| `pii_ssn` | `pii_extended` | both | **yes** |
| `pii_phone` | `pii_extended` | both | no |
| `secret` | `secrets`: the chain's credential patterns applied to replies | response | **yes** |

`jailbreak` and `pii_credit_card` were removed from the block list after both
refused real developer traffic in production. The chain's `secret_scan`
covers credentials on requests, which is why the `secrets` scanner only reads
responses.

History rescanning (`history: off`) is off: each turn gets its own PII and
phrase scan when it is sent, and earlier turns are not scanned again. The
credential scan still reads the whole provider-bound body.

**Not detected:** health identifiers (PHI), toxicity or brand-risk content,
and anything that needs a trained classifier rather than a pattern. Those are
listed on the [roadmap](/documentation/enterprise-roadmap).

## Blocking vs. auditing under enforce

These rules apply once `safety.mode` is set back to `enforce`:

- **Requests:** a finding in `block_categories` is refused before the
  upstream call.
- **Buffered responses:** held until the scan finishes, so a finding in
  `block_response_categories` refuses the whole response.
- **Streamed responses:** audit-only. The tokens are already on the wire when
  the scan runs, so findings are recorded and the stream completes. Where
  blocking matters more than latency, use non-streaming calls.

Enforcement **refuses rather than redacts**. A flagged response is rejected
whole, not rewritten with the values masked. In-flight redaction is on the
[roadmap](/documentation/enterprise-roadmap).

## Display-layer redaction

Transcript text shown in the console (conversation history and search,
artifacts, tool-call and context views) passes through a redactor before it
reaches the page. It masks well-known credential shapes (AWS, GitHub, GitLab,
Slack, Anthropic, OpenAI, Stripe and Google keys, among others) and
high-entropy tokens, and it masks US SSNs down to the last four digits.
Reviewing an audit trail therefore does not expose those values again. It
does not mask email addresses, phone numbers or card numbers. The stored
payload is unchanged and expires under the
[retention windows](/documentation/data-retention).

## Where findings go

Every finding is a row in `ai_safety_findings` with its category, scanner,
phase and `blocked` flag. Under warn, `blocked` is always `false`. Findings
are linked to the request and its trace, so each one can be attributed to a
person through the chain described in
[Audit Trail, Traceability & Observability](/documentation/enterprise-audit-observability).

## Check it yourself

- `/admin/governance?tab=safety`: findings by category, with the blocked
  count beside each.
- `/admin/requests`: open a request to see its findings and the policy that
  judged it.
- `/admin/gateway/policies`: the live `safety` block (mode, scanners, block
  lists).
- The CLI safety table, with a findings and a blocked column per category:

  ```bash
  systemprompt infra logs governance report --since 7d
  ```
