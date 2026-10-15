---
name: indaws-cluster-inspector
description: >-
  Read-only inspection of the Dyncloud Kubernetes clusters (es-prod-01, es-dev-01) via Pinniped/Azure
  AD kubeconfigs over the NetBird mesh. Use when the user wants to check cluster/app/environment
  health, look at pods, ArgoCD apps, CNPG databases, ExternalSecrets, the operator, or read logs —
  without changing anything. Holds the safety rules and the read-only command catalogue; which cluster
  hosts what is read live from the documentation MCP, never from memory. kubectl is READ-ONLY; the
  only allowed write is the ArgoCD hard refresh. Triggers: "inspect cluster", "check es-prod-01", "is
  X healthy", "get pods", "argocd status", "why is the env down", "look at the logs".
version: 0.2.1
effort: low
model: sonnet
allowed-tools: [Bash, Read, Grep, Glob]
---

# Cluster Inspector (read-only)

Inspect the state of the Dyncloud clusters **without touching anything**. The commands and the
safety rules live here; **what each cluster hosts is read live from the documentation MCP** and
never made up.

## ⚠️ Safety rules

They apply **always** (detail in `references/shared/danger-gates.md`):

- **`kubectl` READ-ONLY**. The **only** write allowed is
  `kubectl -n argocd annotate application <app> argocd.argoproj.io/refresh=hard --overwrite`.
- Any other write (edit/apply/delete/scale/rollout) → **STOP, ask for explicit permission and
  explain the blast-radius** (who it affects, how to revert). By default change it via GitOps
  (delegate to `gitops-change`), not by hand.
- **Never** read secret values (`kubectl get secret -o yaml` **forbidden**); look only at metadata.
- **Never** touch the v17 prod (`:8099`/`asoo`).
- **`kubectl exec` on Odoo pods** (`odoo-bin shell`, viewing logs) is a **developer/devops** capability
  (→ `references/shared/access-model.md`), not a mutation of the cluster; **but** in **production** it can write
  data → warn about the blast-radius and confirm first.
- **NEVER make up** the state: if `kubectl` fails, say so and ask to connect NetBird; do not assume.

## What it is / when

Use me to answer "how is X doing?" safely: health of nodes/pods/apps, why an environment
won't start, what a log says, whether an `ExternalSecret` is syncing. To **fix** something →
`gitops-change`. To chase an incident end-to-end → `incident-runbooks`.

## Platform detail: ask the documentation MCP

Which clusters exist and what each one hosts, where the ArgoCD hub lives, namespaces, component
and service names, versions, chart values, the inspection order per case — that is
**documentation, read at the moment it is needed**. A copy in this file is a copy that goes stale.

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
  rules, the kubeconfig setup, the read-only command catalogue, the flow, gotchas), and **mark
  explicitly what stays unconfirmed**. Never invent a version, a host or a path.

**This skill still works with the MCP down.** It is the skill you reach for when something is
broken, and the documentation may be broken with it: the safety rules, the command catalogue and
the gotchas are embedded **on purpose**. What you lose is the platform detail — say which cluster
or which name you could not confirm, and ask the user, rather than guessing one.

## Setup: kubeconfig

**Read-only** access via **Pinniped/Azure AD** over the **NetBird mesh**.
- **Ask the user** where the `es-prod-01` and `es-dev-01` kubeconfigs are; do not assume paths.
- Use `kubectl --kubeconfig <path>` (or `KUBECONFIG=<path>`). If it fails on the network → the mesh is missing:
  say so and ask to connect to NetBird. Do not make up the state.

## What to look at (read-only commands)

- **Nodes / pods**: `kubectl get nodes -o wide`, `kubectl -n <ns> get pods`, `describe`, `logs`.
- **ArgoCD** (on the cluster that hosts the hub — ask the documentation which one):
  `kubectl -n argocd get applications`, and per app the `sync`/`health`
  (`-o jsonpath` of `.status.sync.status`, `.status.health.status`, `.status.operationState.phase`).
- **CNPG** (databases): `kubectl -n <ns> get clusters.postgresql.cnpg.io`, `-db-*` pods,
  `kubectl cnpg status <cluster>` if the plugin is present.
- **ESO** (secrets): `kubectl -n <ns> get externalsecret,secretstore` and their `Ready`/`SecretSynced`
  (never the value).
- **Operator**: `/healthz` `/readyz` `/metrics` probes, and its REST API (read) for the aggregated state.
- **Observability**: delegate to `observability-guide` (metrics/logs/dashboards) and to the status
  page (availability).

To **force a sync** of ArgoCD (the only write allowed): `kubectl -n argocd annotate
application <app> argocd.argoproj.io/refresh=hard --overwrite`.

## Flow
1. **Step 1** — ask for/confirm the cluster kubeconfig (Pinniped, over the mesh); do not hardcode it.
2. **Step 2** — inspect **read-only** (`kubectl get/describe/logs`, `argocd app get`, `cnpg status`, `ExternalSecret` status).
3. **Step 3** — if something needs to **change**, delegate to `gitops-change` (never write to the cluster).

## GOTCHAS
1. **Never hardcode the name of the MCP server.** It is the user's to choose and it differs per machine; identify it by the `mcp__*__docs_*` tool family and read the real name at runtime. Same for the documentation space: match it by name, never by numeric id.
2. A **stuck previous sync** of ArgoCD (waiting for an impossible `health`) blocks the new sync until `progressDeadlineSeconds` fires. Terminating the operation is a write → ask for permission.
3. **`kubectl get secret -o yaml` FORBIDDEN** (exposes values). Look at status, not content.
4. **Pinniped** sometimes hangs port 8001 → kill the `pinniped login oidc` process; the `selfheal` purges `~/.config/pinniped/*`.

## Delegates to
| Skill | Purpose |
|-------|----------|
| `observability-guide` | metrics/logs/dashboards |
| `gitops-change` | apply any change (via GitOps) |
| `common-skills:dynbrain` | how to query the documentation MCP (retrieval, tool choice, navigation) |

## Consumed by
| Skill | Purpose |
|-------|----------|
| `devops-assistant` | entry point / routing |
| `incident-runbooks` | first read-only diagnosis |

## REFERENCES
- `references/shared/danger-gates.md` · `references/shared/docs-access.md` · `references/shared/access-model.md`
- Live platform detail: the documentation MCP (`mcp__*__docs_*`), as `common-skills:dynbrain` describes.
