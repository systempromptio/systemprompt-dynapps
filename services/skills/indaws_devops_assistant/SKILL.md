---
name: indaws-devops-assistant
description: >-
  Entry-point router for DevOps work on the Dyncloud platform. Use when the user asks anything about
  the two clusters (es-prod-01, es-dev-01), the GitOps flow, the manager/operator, observability,
  secrets, CI, or the NetBird mesh, and you need to map the request to the right specialized skill.
  Holds the global safety rules, the routing table and the kubeconfig/mesh setup; every platform fact
  is read live from the documentation MCP, never from memory. Triggers: "devops", "dyncloud infra",
  "how is X deployed", "which cluster", "who do I ask about".
version: 0.3.1
effort: low
model: opus
allowed-tools: [Read, Bash, Grep, Glob]
---

# DevOps Assistant (router)

Entry point for operating the **Dyncloud** platform. It applies the safety rules, **routes**
to the right specialized skill, and gets every concrete platform fact **live from the
documentation MCP** — it does not carry a copy of the platform in its head, and it never
makes one up.

## ⚠️ Safety rules

They apply **always** (detail in `references/shared/danger-gates.md`; what each person can do according to their Azure AD group —devops/developer/readonly— in `references/shared/access-model.md`):

- **GitOps is the only path**; no manual change on the cluster.
- **`kubectl` READ-ONLY**; the only write allowed: `kubectl -n argocd annotate application <app> argocd.argoproj.io/refresh=hard --overwrite`.
- Any unavoidable `kubectl` write → **ask for permission + explain blast-radius** first.
- **Never** read/expose secret values; **never** touch the v17 prod (`:8099`/`asoo`); protected branches **MR-only**.
- **NEVER make up**: if a piece of data is missing and cannot be looked up, say so and point to the source.

## What it is / when

Use me when the request is about Dyncloud infrastructure/DevOps but it's not clear **which**
skill resolves it, or to answer "how is X set up?", "which cluster does Y live in?", "where do
I change Z?". I route you, and I look up what the answer needs.

## Platform detail: ask the documentation MCP

The platform inventory — which clusters exist and what each one hosts, the GitOps chain, the
repos and their paths, hosts, versions, chart values, runbook steps — is **documentation, read
at the moment it is needed**. A copy in this file is a copy that goes stale.

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
  rules, its own procedure, gotchas), and **mark explicitly what stays unconfirmed**. Never invent a
  version, a host or a path.

## Setup: cluster access

Access is **read-only** and goes via **Pinniped/Azure AD** over the **NetBird mesh**.
- **Ask the user** where each kubeconfig is (`es-prod-01`, `es-dev-01`); do not assume paths.
- If a `kubectl` call fails at the network level → the **mesh** is probably missing: say so and
  ask the user to connect to NetBird (detail in **`netbird-mesh`**). Do not make up the cluster
  state.

## Routing (what you want to do → which skill)

| I want to… | Skill |
|---|---|
| See the state of a cluster / an environment (read-only) | **`cluster-inspector`** |
| Make an infra change (via GitOps) | **`gitops-change`** |
| Update / publish the **documentation** (the public / platform / base sites) | **`gitops-change`** |
| Metrics / logs / dashboards / status page | **`observability-guide`** |
| OpenBao policies / secrets / ESO | **`secrets-openbao`** |
| Diagnose a cross-stack incident | **`incident-runbooks`** |
| Develop in the manager or operator | **`dyncloud-development`** |
| The CI of the client **code** repo (uniform pipeline) | **`components-ci`** |
| The NetBird mesh (deployment or config) | **`netbird-mesh`** |
| **Connect an environment to the manager** (install `dyncloud_auth`/`dyncloud_api`, enable the MCP on a customer, a 404 on `/dyncloud/api`) | **`dyncloud-migration`** |
| **Migrate an environment from Docker to Kubernetes** | **`dyncloud-migration`** (Mode B — *not implemented*; route the infra parts here) |
| Drive a customer's Odoo once it is connected (ORM, modules, files, imports) | `common-skills:dyncloud-mcp` |
| **Hide a service from the public** / expose **only over the mesh** (same URL) | **`netbird-mesh`** |
| Create branch / MR / PR in any repo | `developer-skills:git-manager` |
| Look up any platform fact (versions, names, paths, steps) | the documentation MCP (see above) |

## GOTCHAS
1. **Never hardcode the name of the MCP server.** It is the user's to choose and it differs per machine; identify it by the `mcp__*__docs_*` tool family and read the real name at runtime. Same for the documentation space: match it by name, never by numeric id.
2. **Never hardcode** kubeconfig paths: ask for them (Pinniped/mesh).
3. **Never make up**: if a piece of data is missing and cannot be looked up, say what you need and point to the source.
4. What the user **can do depends on their Azure AD group** (→ `access-model.md`); when in doubt, assume the most restrictive.

## Delegates to
| Skill | Purpose |
|-------|----------|
| `cluster-inspector`, `gitops-change`, `observability-guide`, `secrets-openbao`, `incident-runbooks`, `dyncloud-development`, `components-ci`, `netbird-mesh` | depending on the task |
| `developer-skills:git-manager` | branch / MR |
| `common-skills:dynbrain` | how to query the documentation MCP (retrieval, tool choice, navigation) |

## Consumed by
None — it is the plugin's entry point.

## REFERENCES
- `references/shared/danger-gates.md` · `references/shared/docs-access.md` · `references/shared/access-model.md`
- Live platform detail: the documentation MCP (`mcp__*__docs_*`), as `common-skills:dynbrain` describes.
