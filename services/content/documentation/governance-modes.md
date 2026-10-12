---
title: "Governance Modes: Warn and Enforce"
description: "Why governance, safety scanning and quotas all run in warn mode on this instance, what each plane records while it does, how to read the numbers, and how to switch a plane back to enforce."
author: "Dynapps"
slug: "governance-modes"
keywords: "governance, warn mode, enforce, measurement window, safety scanners, quotas, block list, policy, would have blocked"
kind: "guide"
public: true
tags: ["enterprise", "governance", "admin"]
published_at: "2026-09-29"
updated_at: "2026-10-08"
after_reading_this:
  - "Know which of the three enforcement planes refuse traffic today (none of them)"
  - "Read what each plane would have refused, from the console or the CLI"
  - "Tell a category that warn mode is absorbing from a scanner that finds nothing"
  - "Put one plane back into enforce without touching the other two"
related_docs:
  - title: "Content Safety, PII & Guardrails"
    url: "/documentation/enterprise-safety-guardrails"
  - title: "MCP, Tool Governance & Distribution"
    url: "/documentation/enterprise-tool-governance"
  - title: "Cost Management, Budgets & FinOps"
    url: "/documentation/enterprise-cost-management"
---

# Governance Modes: Warn and Enforce

**TL;DR:** Three planes judge every governed request: the **governance
chain**, the **gateway safety scanners** and the **quota windows**. Since
**2026-09-04** all three run in **warn mode** on this instance. Every stage
still evaluates and every finding is still recorded, but **nothing is
refused**. Authentication is separate and always enforced: a missing, invalid
or expired credential is still denied. Warn mode is a measurement window. Read
the numbers, fix the tunables that misfire, then return each plane to
`enforce` one at a time.

## Why the instance is in warn mode

The chain and the scanners were switched on in enforce mode for the
enterprise rollout on 2026-08-25. Within days, false positives denied
ordinary developer traffic. `jailbreak` and `pii_credit_card` had already
come out of the scanners' block list on 2026-08-31 for that reason. Taking
categories out one at a time, each only after a broken workflow proved it
wrong, was not a way to reach a good policy. On 2026-09-04 all three planes
moved to warn so the evidence could build up without blocking work.

## The three planes

| Plane | Configured in | Runs on | Switch | Records to |
|---|---|---|---|---|
| Governance chain | `services/governance/config.yaml` | governed tool calls and gateway prompts | `governance.mode` | `governance_decisions` |
| Safety scanners | `services/gateway/policies.yaml` | gateway requests, and buffered responses | `safety.mode` | `ai_safety_findings` |
| Quota windows | `services/gateway/policies.yaml` | gateway requests | `quota_mode` | `governance_decisions` (policy `quota`) |

### Governance chain

Four stages run in order: **scope check → secret scan → blocklist → rate
limit**. Their settings on this instance:

- **Scope check.** Tools prefixed `mcp__systemprompt__` are admin-only.
- **Secret scan.** A list of credential patterns owned by Dynapps: GitHub,
  Anthropic, OpenAI and Google keys, PEM private keys, JWTs, database URLs
  that embed a password, and structured `password`/`secret`/`client_secret`
  fields. A high-entropy check runs alongside it, but only as an observation.
- **Blocklist.** Tool names containing `delete`, `drop` or `destroy`.
- **Rate limit.** 300 calls per 60 seconds for each session and user.

Under warn, a stage that would have denied writes a `decision = 'warn'` row
with the reason it would have denied. It does not stop the chain, so one call
can be recorded against several stages. Warn is not the same as
`enabled: false`. A disabled stage does not run and records nothing.

`mode` is inherited. The top-level value applies to every policy that does
not set its own, and an unrecognised value fails boot rather than falling back
to either mode.

### Safety scanners

The scanners run are `heuristic`, `secrets` and `pii_extended`. They produce
these categories:

| Category | Scanner | Would block under enforce |
|---|---|---|
| `jailbreak` | heuristic (a pinned phrase list) | no, audit-only |
| `pii_email` | heuristic | no, audit-only |
| `pii_credit_card` | heuristic | no, audit-only |
| `pii_ssn` | pii_extended | **yes**, on requests and responses |
| `pii_phone` | pii_extended | no, audit-only |
| `secret` | secrets (responses only) | **yes**, on responses |

The block lists, `block_categories: [pii_ssn]` for requests and
`block_response_categories: [secret, pii_ssn]` for responses, keep the
pre-warn enforcement set. Under warn they refuse nothing. They mark which
findings the report counts as *would have blocked*, so every finding is
written with `blocked = false`. Under enforce, a streamed response is still
only audited: its tokens are on the wire before the scan finishes.

**Redaction ignores the mode.** Core can rewrite a request finding in place,
as `[REDACTED:<category>]`, for every category listed in
`safety.redact_categories`. That rewrite, and its fail-closed refusal of
content that cannot be redacted (a finding with no location, or a span inside
a signed block or protocol field), apply under `warn` as well as `enforce`.
This is why `services/gateway/policies.yaml` lists no `redact_categories`:
switching redaction on would let warn mode refuse requests. It is switched on
together with the enforce flip. Separately, each scanner's
`scanner_settings.<name>.fail_mode` decides whether a scanner fault or timeout
blocks; here `secrets` is `fail_mode: open` with a 2-second timeout, so a fault
is recorded as a `scanner_failure` finding.

### Quota windows

| Subject | Window | Threshold |
|---|---|---|
| each user | 1 hour | 600 requests or $20 |
| each project (`project`) | 1 day | $100 |
| the whole instance (`organization`) | 1 day | $200 |

Every window still runs in full under warn. Admission **reserves** the
request's estimated tokens and cost in each window, completion settles the
bucket to the audited usage, and a failed request releases its reservation, so
the usage figures are the same as they would be under enforce. A breach is
recorded as a `warn` under policy `quota`, and counted in
`systemprompt_quota_denials_total{mode="warn"}`, rather than answered with
HTTP 429. Treat these numbers
as **warning thresholds, not ceilings**. The instance has no spend cap. The
windows can also be scoped to a `group`, `role`, `cost_centre`, `practice` or
`api_key`, or run for a calendar month, but none of those is declared today.
An API key's own budget and request ceiling (see
[User & Access Management](/documentation/enterprise-user-access#per-key-limits-and-scope-bindings))
are quota windows too and follow the same `quota_mode`. Edit them live at
`/admin/gateway/policies`; see
[Cost Management](/documentation/enterprise-cost-management).

## Reading the measurement

Open **`/admin/governance`**. It has three tabs over one time window and
scope:

- **Decisions:** the chain's log.
- **Safety:** the scanners' log.
- **Hooks:** the plumbing underneath.

The strip across the top shows both planes side by side: each chain stage's
deny count beside the scanners' findings and blocks.
**`/admin/governance/quotas`** shows current usage against each quota window,
per subject, and marks any bucket past its threshold as a warning that did not
refuse the request.

From the CLI:

```bash
systemprompt infra logs governance report --since 7d
systemprompt infra logs governance report --since 7d --group-by tool
systemprompt infra logs governance report --since 30d --group-by user --format csv > warnings.csv
systemprompt infra logs trace list --decision warn --since 24h
```

The report has two tables. The first groups chain warnings by `policy`
(default), `tool` or `user`. The second covers the safety plane, one row per
category and phase, with its **findings** and **blocked** counts. A category
with many findings is one warn mode is absorbing. A category with no findings
at all is a scanner that is catching nothing.

## Returning a plane to enforce

Change one plane, watch it for a few days, then move to the next.

1. **Fix the tunables the report blames.** Examples: a secret pattern that
   matches test fixtures, a blocklist word that hits a harmless tool name, a
   quota threshold set below real usage.
2. **Switch the one plane.**
   - Chain: set `mode: enforce` on a single policy under
     `services/governance/config.yaml`, keeping the others on the inherited
     `warn`. Set it on the top-level `governance.mode` only once every stage
     is ready.
   - Scanners: set `safety.mode: enforce` in `services/gateway/policies.yaml`.
     The block lists then take effect exactly as written.
   - Quotas: set `quota_mode: enforce`. A breach then answers HTTP 429 with a
     `Retry-After` and a machine-readable `error.quota` body, and the
     thresholds become ceilings. A request whose reserved estimate alone
     breaches a cost ceiling is refused before the provider is called.
3. **Deploy it.** Both files are read at boot. The gateway policy table can
   also be edited live at `/admin/gateway/policies`, and `/admin/sync` shows
   whether the table and the file have drifted apart.
4. **Watch `/admin/governance`.** Denials now appear where warnings used to.

Tests pin the current modes and block lists, so any change to them is visible
in review.

## Check it yourself

- `/admin/governance`: the KPI strip shows findings with zero blocks.
- `/admin/governance/quotas`: each window's usage for the current period.
- `/admin/gateway/policies`: the live `default-quotas` policy with `quota_mode`
  and `safety.mode`.
- `systemprompt infra logs governance report --since 7d`
