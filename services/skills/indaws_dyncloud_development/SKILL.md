---
name: indaws-dyncloud-development
description: >-
  Develop the Dyncloud platform itself — the manager (Odoo 19, cloud_* modules) and the operator. Use
  when the user wants to add a feature, fix a bug, write tests, or open an MR against the manager
  (branch 19.0, MR-only) or the operator. Works against the GitLab repos, never against a local path;
  which modules, models, endpoints and pipeline stages exist is read live from the documentation MCP,
  never from memory. Triggers: "cloud_ module", "manager code", "operator code", "kopf", "fastapi",
  "cloud_base", "cloud_k8s", "MR to 19.0", "dyncloud development".
version: 0.2.3
effort: high
model: opus
allowed-tools: [Read, Edit, Write, Bash, Grep, Glob]
---

# Dyncloud development (manager + operator)

Development of the platform's two code pieces: the **manager** (Odoo 19, `cloud_*` modules) and
the **operator**. It works against the **GitLab repos** — not the local paths on anyone's
workstation. What lives inside them (modules, models, endpoints, pipeline stages, contracts) is
read **live from the documentation MCP**; never invented.

## ⚠️ Safety rules

Detail in `references/shared/danger-gates.md`. Here, the critical points:
- **Protected MR-only branches**: `manager` branch `19.0` and the operator (`main`) do **not**
  accept direct push or `--force`. Every change = `feature/…` branch → **MR** (merged by a
  Maintainer). Delegate the flow to `developer-skills:git-manager`.
- **Before pushing**: `pre-commit` green + local tests (never `--no-verify`).
- **`kubectl exec` on Odoo pods** (`odoo-bin shell`, logs) is allowed for the
  **developer**/**devops** level (see `references/shared/access-model.md`); **BUT** against
  **production** it can write data → warn about the blast-radius and confirm first.
- Commits with the identity configured for the repo (see `references/shared/danger-gates.md`).

## What it is / when

Use me to touch the platform **code**: the manager's `cloud_*` modules (Python/XML/OWL) or the
operator (Python). To deploy the result to the cluster, that's **GitOps** → `gitops-change`.

## Platform detail: ask the documentation MCP

The codebase map — which `cloud_*` modules exist and what each one owns, the models, the wizards,
the operator's reconciler and REST endpoints, the pipeline stages, the release and schema
contracts, the charts, the repo paths and versions — is **documentation, read at the moment it is
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
  rules, the flow below, gotchas), and **mark explicitly what stays unconfirmed**. Never invent a
  version, a host or a path.

Before writing code: ask for the module/endpoint that owns the behaviour, and **read the real
source in the repo** before changing it. The documentation says *where*; the repo says *what it
does today*.

## Flow
1. **Step 1** — `feature/…` branch from `origin/19.0` (manager) or `main` (operator); delegate to `git-manager`.
2. **Step 2** — locate the owning module/endpoint (ask the MCP), then read the code before editing.
3. **Step 3** — develop + local tests + `pre-commit` green (never `--no-verify`).
4. **Step 4** — **MR** to the protected branch; merged by a Maintainer. Read the pipeline result before declaring anything done.

## GOTCHAS
1. `manager` branch **`19.0` protected = MR-only**; never direct push or `--force`.
2. The manager upgrade (`click-odoo-update`) is **version-gated**: for views/assets to apply you must **bump the version** in `__manifest__`.
3. **Playwright / real-browser tests FORBIDDEN** (token cost): only `TransactionCase`, pure JS, JSDOM, hoot.
4. `queue_job` v19 **allows `cr.commit()`** in workers, but every handler MUST be **idempotent**.
5. The pipeline's `webhook` job comes out **cosmetically red** (dead tunnel); look at `run-odoo-tests`/`pre-commit`/`build`.
6. The **developer** level can `kubectl exec` `odoo-bin shell` on Odoo pods (→ `access-model`); **careful in production** (it writes data).
7. **Never hardcode the name of the MCP server.** It is the user's to choose and it differs per machine; identify it by the `mcp__*__docs_*` tool family and read the real name at runtime. Same for the documentation space: match it by name, never by numeric id.

## Delegates to
| Skill | Purpose |
|-------|----------|
| `developer-skills:git-manager` | branch / MR / pipeline |
| `components-ci` | the code-repo pipeline |
| `common-skills:dynbrain` | how to query the documentation MCP (retrieval, tool choice, navigation) |

## Consumed by
| Skill | Purpose |
|-------|----------|
| `devops-assistant` | routing |

## REFERENCES
- `references/shared/danger-gates.md` · `references/shared/docs-access.md` · `references/shared/access-model.md`
- Live platform detail: the documentation MCP (`mcp__*__docs_*`), as `common-skills:dynbrain` describes.
