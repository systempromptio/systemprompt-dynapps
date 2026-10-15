---
name: indaws-netbird-mesh
description: >-
  The NetBird mesh of the Dyncloud platform in two layers — (a) deployment of the self-hosted
  components via ArgoCD/GitOps, and (b) mesh configuration (ACLs, policies, setup keys, per-client
  isolation) in its own IaC repo, applied by CI with a manual apply after the MR. Azure AD is the IdP;
  Pinniped cluster access rides the mesh. Covers the mesh-only exposure pattern and webssh. Which
  components run where, repo names and values are read live from the documentation MCP, never from
  memory. Triggers: "netbird", "mesh", "vpn", "acl", "setup key", "pinniped access", "webssh", "client
  isolation".
version: 0.2.2
effort: medium
model: opus
allowed-tools: [Read, Bash, Grep, Glob]
---

# NetBird mesh

The **NetBird** mesh connects workstations, clusters and client servers on a private network. It is
the path over which **Pinniped access** to the clusters travels (if you're not on the mesh, you
don't reach it). Which components exist, where each one runs and which repos hold them is read
**live from the documentation MCP**; never invented.

## ⚠️ Safety rules

Detail in `references/shared/danger-gates.md`. Here: both planes go through **GitOps/CI** (never
manual changes in the panel except what isn't GitOps-able, and with a warning); **never** read/expose
**setup keys** or tokens (they live in OpenBao — use `secrets-openbao`, paths only); **Azure AD** is
the IdP; be careful with the **ACLs** (a mistake can isolate or expose a client → high blast-radius).

## What it is / when

Use me to understand/change the mesh: its components, how it is deployed, how ACLs and setup keys
are configured, and how a client is onboarded with **isolation**.

## Platform detail: ask the documentation MCP

The mesh inventory — which self-hosted components exist and on which cluster each one runs, the
repos and charts that deploy them, the IaC repo that holds the mesh configuration, hosts, chart
values, which services are already exposed mesh-only — is **documentation, read at the moment it is
needed**. A copy in this file is a copy that goes stale.

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
  rules, the two layers and the flow below, the mesh-only pattern, gotchas), and **mark explicitly
  what stays unconfirmed**. Never invent a version, a host or a path.

## The two layers (this is the decision, and it is not symmetric)

- **Deployment layer** — the self-hosted components. Like everything else on the cluster: chart +
  values + catalog entry, branch → MR → merge → **ArgoCD** reconciles. Route to `gitops-change`.
- **Configuration layer** — ACLs, policies, **setup keys**, teams/clients, **per-client isolation**.
  It does **NOT** go through ArgoCD: it lives in its **own IaC repo** and is applied by **CI with a
  manual apply** after the MR. Merging is therefore *not* the same as applying — say so out loud.
  Setup keys are stored in **OpenBao** (`secrets-openbao`, paths only).
- **Client onboarding** is a configuration-layer change: entry in the IaC repo + setup key in
  OpenBao + an ACL that **isolates** that client's devices from the rest.

Ask the MCP for the repo and chart names behind each layer; don't type them from memory.

## Expose a service mesh-only

**The platform already has the capability**: besides the public Envoy fleet there is an **internal
fleet with no public load balancer** — a **reusable capability**, not a one-off. Hang a service's
`HTTPRoute` off an internal Gateway and it becomes **hidden from the public**, reachable **only over
the mesh**, **at the same URL**. Ask the MCP for the fleet's real name and for a service already
using it.

Full pattern (per service):
1. **Internal Gateway** with the same hostname (reuses the same cert Secret).
2. **Pod-level NetBird peer-proxy** (StatefulSet, cloned from the existing Pinniped peer-proxy —
   **never a node agent/router**): peer `--disable-dns` + `socat :443`→internal Envoy + `dnsmasq :5353`.
3. **NetBird nameserver group**: the FQDN → mesh-IP of the proxy `:5353` (to whichever groups apply).
4. **Policies**: `tcp/443` (traffic) + `udp/5353` (DNS). The **public** route stops serving those
   paths (they fall to the redirect).
5. **INVARIANT**: do NOT touch `:80` / `/.well-known/acme-challenge` / public DNS → **Let's Encrypt
   keeps issuing** the cert.

## Flow
1. **Step 1** — the **deployment** layer (components) or the mesh **configuration** layer? Pick the path.
2. **Step 2** — deployment → GitOps/ArgoCD (`gitops-change`); ask the MCP which chart and which values file.
3. **Step 3** — mesh config (ACLs/setup keys/clients) → the IaC repo: MR → **manual apply**. Not applied until someone runs it.

## GOTCHAS
1. **Envoy Gateway v1.8: a single `sourceCIDR` per rule**. Several in the same rule → the `rate-limit-service` **rejects the config and keeps the previous one** (your changes are silently ignored).
2. The mesh does **NOT route the cluster's services CIDR** (only the apiserver TCP for Pinniped); Grafana/others are not peers.
3. FQDNs in Cloudflare must be **DNS-only** (grey cloud), not proxied, or the ACME `:80` challenge doesn't reach Envoy and the cert isn't issued.
4. The **setup keys** live in OpenBao (→ `secrets-openbao`); never in the repo.
5. **Per-client isolation** via ACLs: verify a new client doesn't end up with cross access.

**From the mesh-only pattern (peer-proxy):**

6. **dnsmasq on `:5353`, not `:53`**: the `netbird` container already occupies `:53` of the pod's netns; its dnsmasq would give `Address in use`.
7. **`netbird_policy` = 1 rule per policy** (the provider forces it): the DNS rule (`udp/5353`) goes in a **separate policy** from the traffic one (`tcp/443`).
8. Without the **`udp/5353` policy** the mesh's `default-deny` blocks the DNS → resolution fails (`could not resolve host`).
9. netbird provider: `nameservers = [{ ip, ns_type, port }]` is a **list attribute**, not a `nameserver {}` block.
10. **The StatefulSet's RollingUpdate gets stuck** with the pod *unhealthy* → you must **delete the pod** for it to roll to the new spec (ask permission, it's a `kubectl` write).

## Delegates to
| Skill | Purpose |
|-------|----------|
| `secrets-openbao` | setup keys in OpenBao |
| `gitops-change` | deployment via ArgoCD |
| `developer-skills:git-manager` | branch / MR |
| `common-skills:dynbrain` | how to query the documentation MCP (retrieval, tool choice, navigation) |

## Consumed by
| Skill | Purpose |
|-------|----------|
| `devops-assistant` | routing |
| `cluster-inspector` | Pinniped access travels over the mesh |

## REFERENCES
- `references/shared/danger-gates.md` · `references/shared/docs-access.md` · `references/shared/access-model.md`
- Live platform detail: the documentation MCP (`mcp__*__docs_*`), as `common-skills:dynbrain` describes.
