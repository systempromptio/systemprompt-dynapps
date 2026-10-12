---
title: "MCP, Tool Governance & Distribution"
description: "Govern tool calls with the four-stage pre-execution chain, run a declarative MCP server registry with instant revocation, and distribute, version and roll back skills and plugins through signed catalogs."
author: "Dynapps"
slug: "enterprise-tool-governance"
keywords: "mcp, tools, governance, registry, revocation, signing, plugins, skills, blocklist, warn mode, versions, rollback, tool schema"
kind: "guide"
public: true
tags: ["enterprise", "mcp", "admin"]
published_at: "2026-08-25"
updated_at: "2026-09-29"
after_reading_this:
  - "Trace a tool call through the four-stage pre-execution governance chain"
  - "Know that the chain records but does not refuse today, and where to read what it would have refused"
  - "Administer MCP servers, auth scopes and entitlements at /admin/mcp"
  - "Revoke a server or a token and see it take effect immediately"
  - "Distribute, version and roll back skills through signed catalogs and kits"
related_docs:
  - title: "Governance Modes: Warn and Enforce"
    url: "/documentation/governance-modes"
  - title: "Content Safety, PII & Guardrails"
    url: "/documentation/enterprise-safety-guardrails"
  - title: "Access Control: Who Reaches What, and Why"
    url: "/documentation/access-control"
---

# MCP, Tool Governance & Distribution

**TL;DR:** A governed tool call passes a synchronous four-stage chain
before it runs: scope check, secret scan, blocklist, rate limit. Every
decision is audited under a trace id. **The chain has run in warn mode since
2026-09-04.** It records what it would have refused and lets the call
continue. Authentication and access-control entitlements are still enforced.
MCP servers are declared in `services/mcp/` and administered at `/admin/mcp`,
with per-server OAuth scopes, entitlement by role or Entra group, and immediate
revocation. Skills and plugins are delivered through signed manifests. Each
marketplace version is a content hash that can be compared and rolled back.

## Pre-execution governance

The chain runs **before the tool executes**, synchronously, on every
governed call. Local tools in Claude Code are governed through the hooks that
the `dynapps-commons` plugin installs. Routing model inference alone does not
govern local tools. The stages, as configured in
`services/governance/config.yaml`:

1. **Scope check.** Tools prefixed `mcp__systemprompt__` (the admin MCP
   server) need an admin caller.
2. **Secret scan.** The tool arguments are checked against credential
   patterns owned by Dynapps (GitHub, Anthropic, OpenAI and Google keys, PEM
   keys, JWTs, database URLs that embed a password, structured
   `password`/`secret` fields). A high-entropy check runs alongside and only
   observes.
3. **Blocklist.** Tool names containing `delete`, `drop` or `destroy`.
4. **Rate limit.** 300 calls per 60 seconds for each session and user.

Under enforce, the **first deny wins** and the call stops. Under warn, which
is how the instance runs today, each stage that would have denied writes a
`decision = 'warn'` row and the call proceeds, so a call can be recorded
against several stages at once. [Governance Modes](/documentation/governance-modes)
explains the measurement window and how to switch each stage back to enforce.

```bash
systemprompt infra logs trace list --limit 20
systemprompt infra logs trace list --decision warn --since 24h
systemprompt infra logs trace show <trace-id>
systemprompt infra logs governance report --since 7d --group-by tool
```

What the secret and PII scanners detect on the gateway is covered in
[Content Safety, PII & Guardrails](/documentation/enterprise-safety-guardrails).

## The MCP server registry

Each server is one YAML file under `services/mcp/`. This instance declares
five:

| Server | What it reaches |
|---|---|
| `systemprompt` | this platform's admin CLI. It cannot be removed and is admin-only |
| `atlassian` | Jira and Confluence, under each person's own Atlassian account |
| `salesforce` | Salesforce's hosted MCP, authenticated as the signed-in person |
| `sfcore_dev` | the SFCore team's development-workflow server, called with a short-lived token this instance signs |

Each server declares:

- **Auth requirements:** the OAuth audience (`mcp`) and the scopes a caller's
  token must carry.
- **Entitlement:** which roles, groups and projects reach it, declared in
  `services/access-control/rules.yaml`. A person outside those bands sees no
  server at all, rather than an error on every call.
- **Revocation:** `enabled: false` takes a server out of service, and an
  entitlement can be removed or given an expiry in the console. Issued tokens
  are revoked one by one through **JTI-based revocation**, without waiting for
  them to expire.

`/admin/mcp` lists the fleet. `/admin/mcp/<id>` shows one server's tools,
callers, call log, connections, declaration, and who may reach it.

## Tool schemas

A server with a missing or invalid manifest, or one whose declared database
schema fails to sync, is **refused at registration**, so a broken server is
caught before anyone calls it. Tool input schemas are captured at discovery.
On a gateway request, the request detail page puts the client's `tools` array
beside the one the gateway sent upstream, paired by name, so you can see what
the model was actually given.

### Tool schema validation

A provider refuses a whole request when one offered tool schema is malformed,
so tool schemas are meta-validated before that can happen. Each `inputSchema`,
and the `outputSchema` when a tool declares one, is checked against:

- the **JSON Schema meta-schema** its `$schema` names, Draft 2020-12 when it
  names none (Draft 7 and the other standard drafts are recognised);
- the shapes providers reject even when the meta-schema allows them: the top
  level must be **`type: object`**, a top-level **`required`** may name only
  keys that `properties` declares, and **no `$ref` may leave the document**
  (only `#/...` references; nothing is fetched).

Where it runs:

| Where | What happens on a malformed schema |
|---|---|
| The bundled `systemprompt` MCP server, at start | It refuses to serve, and the error names the server, the tool, the schema side and the violation. |
| CI | The `check-mcp-tool-schemas.sh` gate checks every schema declared under `services/mcp/` and proves the checker against fixtures; the unit tests run the full check over the bundled server's tools. |
| The request detail page | A client tool whose schema fails carries a **schema errors** badge and lists each violation. |

The checker is `extensions/mcp/shared/src/tool_schema.rs`. Servers that
list their tools only at runtime (external and connector servers) are
registered by core and are checked on the request page, not at registration.

## Signed distribution, versions and rollback

Skills and plugins reach workstations through marketplaces, administered at
`/admin/marketplaces`, `/admin/plugins` and `/admin/skills`. The bridge
manifest records each skill's SHA-256 and is **Ed25519-signed**. A client
verifies the signature before installing, so nothing unsigned or tampered
with reaches a workstation. A signature proves integrity and origin; it
grants no model, MCP or tool access, which stays governed by access control.

Skills are the unit of prompt distribution, and they carry their own version
lifecycle:

- **Versions.** A marketplace version *is* the content hash of what it
  serves. `/admin/analysis/versions` shows each marketplace's history,
  compares two versions skill by skill, and shows which devices received
  which version. See [Versions](/documentation/analysis-versions).
- **Updates.** Content kept in an external kit is published by the kit's CI
  and made live with **Import** on `/admin/sync`, with no restart. See
  [Update a Skill](/documentation/update-a-skill) and
  [Code ↔ Instance](/documentation/services-sync).
- **Revocation.** A disabled skill is left out of every bundle, and a managed
  revision can be withdrawn. See [Skills lifecycle](/documentation/skills-lifecycle).
- **Rollback.** Re-pin the kit to its previous digest and import it again. The
  cache keeps the last two trees, so a rollback is a re-point, not a download.

There is no separate parameterised prompt-template object. Prompts ship as
skills.

## Check it yourself

- `/admin/governance`: the **Decisions** tab lists each chain stage's rows;
  under warn they are all `warn`.
- `/admin/mcp`: the five servers, each with its entitlement.
- `/admin/analysis/versions`: marketplace versions by hash, with history and
  compare.
- `systemprompt infra logs governance report --since 7d --group-by policy`
- `systemprompt core skills list`
