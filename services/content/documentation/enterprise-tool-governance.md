---
title: "MCP, Tool Governance & Distribution"
description: "Govern every tool call with a four-stage pre-execution chain, run a declarative MCP server registry with instant revocation, and ship signed skills and plugins."
author: "Astound Digital"
slug: "enterprise-tool-governance"
keywords: "mcp, tools, governance, registry, revocation, signing, plugins, skills, blocklist, warn mode"
kind: "guide"
public: true
tags: ["enterprise", "mcp", "admin"]
published_at: "2026-08-25"
updated_at: "2026-09-04"
after_reading_this:
  - "Trace a tool call through the four-stage pre-execution governance chain"
  - "Administer MCP servers, auth scopes, and entitlements at /admin/catalog/mcp"
  - "Run a stage in warn mode to measure what enforcement would cost before turning it on"
  - "Revoke a server or a token and see it take effect immediately"
  - "Distribute skills and plugins through the Ed25519-signed catalog"
related_docs:
  - title: "Content Safety, PII & Guardrails"
    url: "/documentation/enterprise-safety-guardrails"
  - title: "Audit Trail, Traceability & Observability"
    url: "/documentation/enterprise-audit-observability"
  - title: "Enterprise Roadmap & Known Limitations"
    url: "/documentation/enterprise-roadmap"
---

# MCP, Tool Governance & Distribution

**TL;DR:** Every MCP tool call passes a synchronous four-stage chain — scope check, secret scan, blocklist, rate limit — first-deny-wins, before the tool runs, with every decision audited under a trace id. Any stage can be run in warn mode, which records what it would have refused without refusing it. MCP servers live in a declarative registry at `/admin/catalog/mcp` with per-server auth, role and AD-group entitlement, and immediate revocation; skills and plugins ship through Ed25519-signed manifests.

## Pre-execution governance

The chain runs **before the tool executes**, synchronously, on every call:

1. **Scope check** — does the caller's token carry the scope and audience this server demands?
2. **Secret scan** — do the tool arguments carry credentials that must not leave the boundary?
3. **Blocklist** — is this tool or pattern explicitly denied?
4. **Rate limit** — is the caller within their quota window?

Evaluation is **first-deny-wins**: the first failing stage stops the call, and the tool never runs. Every decision — allow or deny — is audited with trace linkage, so a denied call is as visible as an executed one:

```bash
systemprompt infra logs trace list --limit 20
systemprompt infra logs trace show <trace-id>
```

The chain's content-safety companion (what the secret and PII scanners actually detect) is covered in [Content Safety, PII & Guardrails](/documentation/enterprise-safety-guardrails).

## Warn mode

A stage that is misfiring leaves you two bad options. Retune a threshold you have no data for, or switch the stage off and lose the evidence along with the denials. Warn mode is the third: the stage runs, evaluates, and records exactly what it would have refused, and the call proceeds.

Set it per policy, or once for the whole chain, in `services/governance/config.yaml`:

```yaml
governance:
  mode: warn            # the default for every policy below
  policies:
  - id: secret_scan
    enabled: true
  - id: scope_check
    enabled: true
    mode: enforce       # this one still refuses
```

`mode` is inherited: the top-level value applies to any policy that does not name its own. An unrecognised value fails the boot job rather than falling back, because reading `warnn` as `enforce` would block traffic you believed you had unblocked, and reading it as `warn` would disable enforcement nobody asked to disable.

Warn mode is **not** the same as `enabled: false`. A disabled stage does not run and writes nothing. A warning stage runs and writes a `governance_decisions` row with `decision = 'warn'` carrying the reason it would have denied on, so a warn row and a deny row are directly comparable. It also does not halt the chain, so a call that trips three warning stages is recorded against all three rather than only the first.

The gateway safety scanners are a separate plane with its own switch, `safety.mode: warn` in `services/gateway/policies.yaml`. It leaves every scanner running and every finding persisted, and only drops the refusal — findings written under it carry `blocked = false`, which is what lets a report say how many calls a block list would have cost.

Read both planes back over one window:

```bash
systemprompt infra logs governance report --since 7d
systemprompt infra logs governance report --since 7d --group-by tool
systemprompt infra logs governance report --group-by user --format csv > warnings.csv
systemprompt infra logs trace list --decision warn --since 24h
```

The same window is on the dashboard at `/admin/governance/warnings`, grouped by policy, tool or user, with a CSV export. The two numbers to read there are a category's **findings** count beside its **blocked** count: many findings and no blocks is a rule warn mode is currently absorbing, and no findings at all is a scanner earning nothing.

Warn mode is a measurement window, not a posture. Once the report shows which tunables are wrong, fix those and return the stages that no longer misfire to `mode: enforce`, one at a time.

## The MCP server registry

`/admin/catalog/mcp` administers the declarative registry. Each server declares:

- **Auth requirements** — the token audience and scopes a caller must hold (for example, audience `mcp` with scope `admin` for the administrative server).
- **Entitlement** — which roles or AD groups (the project groups included) may reach the server, so access follows the directory automatically.
- **Revocation** — a server can be disabled with immediate effect, and **JTI-based token revocation** kills individual issued tokens without waiting for expiry.

## Fail-fast schema validation

Registration validates up front: a server with a **missing or invalid manifest**, or whose declared schema fails to sync to the database, is **refused at registration** rather than discovered broken at call time. Tool input schemas are captured at discovery, so what a tool accepts is on record. (One caveat: captured schemas are not meta-validated against the JSON-Schema spec itself — see the [roadmap](/documentation/enterprise-roadmap).)

## Signed distribution of skills and plugins

Skills and plugins are distributed through `/admin/catalog/plugins` and `/admin/catalog/skills`, carried by **Ed25519-signed manifests**: a client verifies the signature before installing, so nothing unsigned or tampered-with reaches a workstation, and central revocation removes an artifact from circulation immediately.

Signed manifests can carry prompt content, but there is no first-class **versioned prompt-template object** with parameters, pinning, and rollback yet — that lifecycle is on the [Enterprise Roadmap](/documentation/enterprise-roadmap).

## Verified evidence

Every capability on this page is proven by tagged end-to-end tests run against a seeded instance. To replicate: `just start`, then `just e2e-seed --reset`, then the command in the table. Screenshots regenerate with `just e2e-screens`.

| Ref | Verified behaviour | Replicate with |
|---|---|---|
| REQ-040 | A signed skill/plugin installs after signature verification and disappears from circulation on central revocation | `just e2e-req REQ-040` |
| REQ-041 | Each of the four chain stages denies its case pre-execution, first-deny-wins, with the decision audited under a trace id | `just e2e-req REQ-041` |
| REQ-042 | The registry enforces per-server auth/scopes/audience and role/group entitlement, and JTI revocation kills an issued token | `just e2e-req REQ-042` |
| REQ-043 | Registration of a server with a missing or invalid manifest fails fast; tool schemas are captured at discovery | `just e2e-req REQ-043` |

Deeper gateway-level checks for these behaviours also run in the platform's integration suite (`just test-integration`).

### Screenshots

![The MCP server catalog with the systemprompt server registered and its admin scope visible](/files/images/evidence/req-042-mcp-catalog.png)

![The plugin catalog listing centrally-managed bundles](/files/images/evidence/req-040-plugin-catalog.png)

![The skills catalog listing distributable skill entries](/files/images/evidence/req-040-skills-catalog.png)

![A denied tool call surfacing as a DENY with its governing policy and tool named](/files/images/evidence/req-026-requests-audit.png)
