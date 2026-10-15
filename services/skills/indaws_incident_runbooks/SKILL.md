---
name: indaws-incident-runbooks
description: >-
  Cross-stack diagnosis order and common-incident runbooks for the Dyncloud platform. Use when
  something is broken and the user needs a structured way to find the cause across backend → GitOps →
  ArgoCD → operator → cluster, or a known-incident playbook (ImagePullBackOff, OutOfSync,
  SecretSyncError, rate-limit, cert issues, env won't deploy). Holds the diagnosis order and the
  incident table so they survive an outage; the full runbooks, repos and resource names are read live
  from the documentation MCP, never from memory. Triggers: "incident", "it's down", "not working",
  "debug", "diagnose", "outage", "troubleshoot", "runbook".
version: 0.2.1
effort: medium
model: opus
allowed-tools: [Read, Bash, Grep, Glob]
---

# Incident Runbooks

When something breaks, follow a diagnosis **order** and use the incident playbook. This skill
**looks** (delegates to `cluster-inspector`) and **fixes via GitOps** (delegates to `gitops-change`); it
does not touch the cluster by hand. The exact detail — full runbooks, repo paths, resource names —
is **read live from the documentation MCP**, never made up.

## ⚠️ Safety rules

They apply **always** (detail in `references/shared/danger-gates.md`):

- Diagnosis = **read-only**. `kubectl` read-only; the only write allowed = ArgoCD `annotate …
  refresh=hard`. The remedy goes via **GitOps** (`gitops-change`), not by hand.
- If a remedy requires a manual `kubectl` write → **ask permission + explain blast-radius**.
- **Never** read secret values; **never** the v17 prod.
- **NEVER invent** the cause: if you can't observe it (missing mesh/data), say so and ask for what's missing.
- The available action level depends on the user's **Azure AD group** → `references/shared/access-model.md`.

## What it is / when

Use me when "something's off" and you don't know where to start, or for a known incident. To only
inspect → `cluster-inspector`. To apply the fix → `gitops-change`.

## Platform detail: ask the documentation MCP

The full runbooks — which repo and which file holds a given piece of state, the exact store,
release and resource names, hosts, versions, chart values, the step-by-step per incident — are
**documentation, read at the moment they are needed**. A copy in this file is a copy that goes stale.

- **Identify the server by its tool family, never by its name**: it is the one exposing
  `mcp__*__docs_*`. The middle segment is whatever the user called their server — something
  you read at runtime, never something you write down. Several matches or none → **ask which
  one before calling any tool**.
- **Preflight before the first lookup**, not after it fails. Then **scope by name**: discover
  the platform documentation with `docs_list_spaces` and match it **by its name**; never by a
  numeric id.
- **Ask in natural language and cite the source** document for every fact you state.
- The contract (identification, preflight, degradation) is
  **`references/shared/docs-access.md`**. **How to query well is `common-skills:dynbrain`** —
  read that skill and use it; don't reimplement it here.
- **MCP unavailable** → say it plainly, ask the user to connect it (the OAuth handshake is
  theirs to complete, you cannot do it for them), still answer with this skill's core (safety
  rules, the diagnosis order, the incident table, gotchas), and **mark explicitly what stays
  unconfirmed**. Never invent a version, a host or a path.

**This skill still works with the MCP down.** During an incident the documentation may be down
too — that is exactly why the diagnosis order, the incident table and the gotchas are embedded
here **on purpose**. Work the symptom with them, and say which exact name, path or step you could
not confirm instead of guessing one.

## Diagnosis order (cross-stack)

Follow the chain **from outside in**, stopping at the first layer that explains the symptom:

1. **Manager (the backend)** — does it reflect the expected state/action? (manager logs/state).
2. **GitOps (Git)** — is the change merged in the right repo and on the branch ArgoCD watches?
3. **ArgoCD** — is the app `Synced`/`Healthy`? is there a stuck `operationState`?
4. **Operator** — does it report status? reconcile errors? (probes/API/metrics).
5. **Cluster** — pods/events/CNPG/ESO/network (ingress gateways).

## Common incidents (first step → fix)

| Symptom | First diagnosis (read-only) | Fix |
|---|---|---|
| **ImagePullBackOff** | `describe pod` → tag/digest does not exist in registry | correct bump (tag; +digest if values pins it) via `gitops-change`; verify pipeline SHA==commit |
| **App OutOfSync/Progressing stuck** | `operationState.phase=Running` waiting for impossible health | drop `progressDeadlineSeconds` (autosync re-applies) or fix the manifest in Git |
| **SecretSyncError** | `ExternalSecret` not `SecretSynced` → wrong store or path | `secrets-openbao`: point it at the right store + grant the path in policy via CI |
| **TLS / cert-manager** | `Certificate`/`Order`/HTTP-01 challenge not issuing | check DNS→load balancer and allowlist; fix via GitOps |
| **Envoy rate-limit** | `BackendTrafficPolicy` rejected (duplicate descriptor) | a single `sourceCIDR` per rule; fix the policy/chart |
| **Environment won't deploy** | the environment's fleet entry / release pinning / values | `gitops-change` (ask the documentation which repo holds each) |
| **Red webhook** | manager pipeline `webhook` (notify) job | **cosmetic**: look at `run-odoo-tests`/`pre-commit`/`build`, ignore the webhook |

## GOTCHAS
1. **Never hardcode the name of the MCP server.** It is the user's to choose and it differs per machine; identify it by the `mcp__*__docs_*` tool family and read the real name at runtime. Same for the documentation space: match it by name, never by numeric id.
2. **ImagePullBackOff** almost always = wrong tag/digest (see `gitops-change`): 8-char short SHA, tag+digest together.
3. App **OutOfSync/Progressing stuck**: a previous sync waits for an impossible health → it unblocks when `progressDeadlineSeconds` (~10 min) elapses; the site keeps running the old pod (rolling update).
4. **SecretSyncError** = wrong ESO store or policy missing the path (→ `secrets-openbao`).
5. The red `webhook` job is usually **cosmetic** (dead tunnel), not a real failure.
6. ALWAYS start read-only (`cluster-inspector`); ALWAYS fix via GitOps (`gitops-change`).

## Delegates to
| Skill | Purpose |
|-------|----------|
| `cluster-inspector` | read-only diagnosis |
| `gitops-change` | fix via GitOps |
| `secrets-openbao` | secret failures |
| `observability-guide` | signals/metrics |
| `common-skills:dynbrain` | how to query the documentation MCP (retrieval, tool choice, navigation) |

## Consumed by
| Skill | Purpose |
|-------|----------|
| `devops-assistant` | routing |

## REFERENCES
- `references/shared/danger-gates.md` · `references/shared/docs-access.md` · `references/shared/access-model.md`
- Live platform detail: the documentation MCP (`mcp__*__docs_*`), as `common-skills:dynbrain` describes.
