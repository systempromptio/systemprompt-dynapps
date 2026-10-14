---
name: indaws-secrets-openbao
description: >-
  Guides secret management on the Dyncloud platform with OpenBao and External Secrets Operator (ESO).
  Use when the user needs to create or change an OpenBao policy (via the policy repo's GitLab CI),
  decide which ClusterSecretStore a path belongs to, or wire an ExternalSecret into a pod. Store
  names, paths and policy names are read live from the documentation MCP, never from memory. NEVER
  reads or exposes secret VALUES — paths only. Triggers: "openbao", "vault", "secret", "policy",
  "externalsecret", "ESO", "clustersecretstore", "seal".
version: 0.2.2
effort: medium
model: opus
allowed-tools: [Read, Bash, Grep, Glob, Edit, Write]
---

# Secrets — OpenBao + ESO

Platform secret management: **OpenBao** (store) + **External Secrets Operator** (ESO,
delivers the secret to the pod). **Policies** are changed via **GitLab CI**, not by hand. The
exact detail (which store serves which path prefix, policy names, examples) is read **live**
from the documentation MCP; never invented.

## ⚠️ Safety rules

They apply **always** (detail in `references/shared/danger-gates.md`):

- **Never read or expose secret VALUES**: paths yes, values **no**. `kubectl get secret -o yaml`
  and `bao kv get` of the value are **forbidden**. Work with **paths and metadata**.
- **Never** read or touch the Azure OIDC `client_secret` (neither its value nor whether it is set).
- Policy changes = **GitOps/CI** (branch → MR → manual apply of the pipeline), never a live edit.
- **NEVER invent** paths or policy names: look them up; if you can't, say so.

## What it is / when

Use me to: create/edit an **OpenBao policy**, understand **which store** to use, or **wire** an
`ExternalSecret` so a pod receives a credential. To deploy the change to the cluster →
`gitops-change`. To check whether an `ExternalSecret` syncs (read-only) → `cluster-inspector`.

## Platform detail: ask the documentation MCP

The secret inventory — which `ClusterSecretStore` serves which path prefix, the policy names and
their grants, the path layout of a client environment, which repo and file holds each manifest —
is **documentation, read at the moment it is needed**. A copy in this file is a copy that goes
stale. (Asking about a **path** is fine; a **value** is never read, from anywhere.)

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
  rules, the store rule, `Change a policy`, the flow, gotchas), and **mark explicitly what stays
  unconfirmed**. Never invent a version, a host or a path.

## Which store: the path decides

The **path decides the store**, not the namespace — and picking the wrong one is **failure #1**
(`403` / `SecretSyncError`). So resolve the path first, then the store: ask the documentation
which store reads that prefix and cite the answer. Never pick by which namespace the pod is in,
and never by which store you saw last.

## Change a policy (via GitLab CI)

1. **Ask the documentation for the policy repo** — the IaC repo (Terraform/tofu) that owns the
   OpenBao policies — and edit the policy file there. Policies are **auto-discovered**: editing
   the policy file is enough, there is **no separate registration script** to run.
2. **Branch → MR → merge** (use `git-manager`). The `apply` is **manual** in that repo's
   protected-branch pipeline (human gate) → trigger it after the merge.
3. Verify the effect without reading values (e.g. that the `ExternalSecret` turns `SecretSynced`).

## Flow
1. **Step 1** — identify the secret **path**, then look up **which store** reads that prefix.
2. **Step 2** — change the **policy** via GitLab CI (the policy repo, tofu): branch → MR → **manual apply**.
3. **Step 3** — verify that the `ExternalSecret` syncs (without reading the value).

## GOTCHAS
1. **Never hardcode the name of the MCP server.** It is the user's to choose and it differs per machine; identify it by the `mcp__*__docs_*` tool family and read the real name at runtime. Same for the documentation space: match it by name, never by numeric id.
2. **The path decides the store**, not the namespace; the wrong store → **403 / SecretSyncError**. There is more than one store (platform vs client environments) — which one serves a given prefix is documentation: look it up, don't guess.
3. **Not every cluster's ESO has the same reach**: at least one is a **least-privilege** allowlist (no broad prefix wildcard) → there, a new path needs a **grant in the policy + manual apply** of the policy repo's pipeline. Which cluster is documentation: confirm it, don't assume the grant is already there.
4. **Never read VALUES** (`bao kv get` of a value, `get secret -o yaml`); paths/keys yes. Never the Azure OIDC `client_secret`.
5. The policy is evaluated at request-time → after the grant no AppRole re-login is needed, just retry the ESO.

## Delegates to
| Skill | Purpose |
|-------|----------|
| `gitops-change` | the policy MR / ESO manifests |
| `common-skills:dynbrain` | how to query the documentation MCP (retrieval, tool choice, navigation) |

## Consumed by
| Skill | Purpose |
|-------|----------|
| `devops-assistant` | routing |
| `netbird-mesh` | mesh setup keys in OpenBao |
| `gitops-change` | secrets referenced in changes |

## REFERENCES
- `references/shared/danger-gates.md` · `references/shared/docs-access.md` · `references/shared/access-model.md`
- Live platform detail: the documentation MCP (`mcp__*__docs_*`), as `common-skills:dynbrain` describes.
