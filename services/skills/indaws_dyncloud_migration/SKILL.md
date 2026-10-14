---
name: indaws-dyncloud-migration
description: >-
  Installs the Dyncloud Manager connector (dyncloud_auth + dyncloud_api) on a managed Odoo environment
  so the manager can drive it over MCP/Connect-as. Mode A (working) — legacy Docker environments, via
  the parts manifest (repositories.yml or .gitmodules) plus docker-compose environment variables. Mode
  B (not implemented) — migrating an environment from Docker to Kubernetes. Covers the read-only
  deploy token, the signing-key lineage, the JWKS bootstrap and the silent failures along the way. Use
  it when someone wants to connect an environment to the manager, enable the MCP on a customer, fix a
  404 on /dyncloud/api, or prepare the jump from Docker to K8s. Triggers: "install dyncloud_api",
  "connect environment to the manager", "enable MCP on a customer", "instalar dyncloud_api", "conectar
  entorno al manager", "activar MCP en cliente", dyncloud_auth, "404 dyncloud api", "migrate to
  kubernetes", repositories.yml, JWKS.
version: 0.7.1
effort: medium
model: opus
allowed-tools: [Read, Bash, Grep, Glob, Edit, Write]
---

# Dyncloud Migration

Connects a managed Odoo environment to **Dyncloud Manager** by installing the two modules the
manager talks to: **`dyncloud_auth`** (verifies the Ed25519 tokens the manager signs) and
**`dyncloud_api`** (the ORM/admin dispatcher the MCP proxy calls). Without them the manager can
see the environment but cannot operate it: every `/dyncloud/api/v1/*` answers **404**.

## ⚠️ Safety rules

They apply **always** (detail in `references/shared/danger-gates.md`):

- **GitOps / repo-first.** The change is a **commit to the environment's code repo**; the deploy is
  triggered by its webhook. Never edit files on the server by hand.
- **`kubectl` READ-ONLY** (Mode B and any inspection). Only write allowed = ArgoCD `refresh=hard`.
- **Never write a token, password or key into a repo you do not control** — and never into this
  skill. The deploy token is shared out of band; § *Step 1* explains how to mint one.
- **A deploy token in `repositories.yml` is committed and pushed**: it lands in that repo's history
  and cannot be removed without rewriting it. Say so **before** the user chooses that route.
- Protected branches **MR-only** → delegate the flow to **`developer-skills:git-manager`**.
- **NEVER invent** a uuid, an FQDN, a host or a commit SHA: look them up (§ *Where each value
  comes from*); if you cannot, say what is missing.

## When to use / the two modes

| Mode | State | Use it when |
|---|---|---|
| **A — Docker** | **operational** | A legacy Docker environment must be reachable by the manager. This is the vast majority of the fleet. → `references/modes/docker.md` |
| **B — Docker → K8s** | **not implemented** | Deferred; never run end to end. What is known and what is missing → `references/modes/k8s.md` |

To *use* the MCP once connected → **`common-skills:dyncloud-mcp`**. To change cluster infra →
`gitops-change`. To look at a cluster → `cluster-inspector`.

## Prerequisite that gates everything: the Odoo series

`dyncloud-odoo-auth` must have a branch **and a tag for the target Odoo series**. Confirm it
before promising anything — at the time of writing only one series is published, and the module
has version-specific code (Odoo 19 made `Environment.cr` read-only, for instance). Targeting a
series with no branch means **porting the module first** — a development task, not this procedure.

## Mode A — the shape of it

Two files in the environment's own repository, **one commit**, one manual install. Nothing is
edited on the server by hand. **The procedure, with the check that closes each step, is
`references/modes/docker.md`** — follow it there; this is only the shape:

1. **Deploy token** (`read_repository`) on the `dyncloud-odoo-auth` project — the SSH deploy-key
   route fails silently (GOTCHA 1).
2. **The parts manifest** → add the module, pinned to the series' tag. **Check which format this
   repo uses first**: newer ones carry `repositories.yml`, legacy ones a standard `.gitmodules`.
   They are different code paths in Dyncloud and editing the wrong one changes nothing.
3. **`docker-compose.yml`** → the environment variables. Which ones, and **which two must stay
   unset**, decide whether this works: `references/values-and-sources.md`. You do **not** need
   database access to fill them: `list_environments` returns the environment's `uuid` and `aud`
   directly — take both verbatim.
4. **Push — both files in the SAME commit.** Every push to the environment's branch deploys, so
   splitting them costs two deploys and the first lands in a known-broken state. Watch the action,
   don't assume.
5. **Install both modules in the database — MANUAL, by a developer.** Deliberately not automated:
   `auto_install` only fires on a *fresh* database, and on an existing one `must_install()` needs a
   dependency in state `'to install'` **in that same operation**. The module ships in the image,
   appears in the list, and stays uninstalled.

**Verify in this order:** `list_databases` first — it proves the environment fetched the manager's
keys and validated an Ed25519 signature — then `system_odoo_version`, which must report a real
version and a `capabilities` list. No cron to run by hand: the `post_init_hook` does the first JWKS
fetch at install time.

## Platform detail: ask the documentation MCP

Hosts, the manager's URL, repo paths, chart values and per-environment data are **read when
needed**, never copied here. Identify the server by its **`mcp__*__docs_*` tool family**, never by
name; preflight before the first lookup; cite the source. Contract in
`references/shared/docs-access.md`; how to query well is `common-skills:dynbrain`.

**MCP unavailable** → say it plainly, still answer with this skill's core (safety rules, the five
steps, the variables, the gotchas), and **mark explicitly what stays unconfirmed**. Never invent a
uuid, an FQDN or a URL.

## GOTCHAS

1. **The SSH deploy-key route fails silently.** The platform token holds Reporter (20) on the group
   and creating a deploy key needs Maintainer (40) → GitLab answers **403**, which is swallowed by
   an `except Exception: pass`. The action reports `done`, the key exists in Dyncloud but not in
   GitLab, and the clone then fails inside a command run with `warn=True` whose exit code nobody
   checks. **Two green actions, nothing downloaded.** Tell: the SSH key's `external_id` is empty.
   Use HTTPS + deploy token instead.
2. **`LINEAGE_NAME` has no usable default.** The module falls back to the literal `"default"`,
   which matches no lineage — the real ones are `k8s` and `docker`. Unset → the backend is asked
   for keys it never returns.
3. **Setting `BACKEND_SPKI` disables normal TLS validation** (`verify_mode = CERT_NONE`) and pins
   the SHA-256 of the **whole DER certificate**, not the SPKI field, despite the name. With a
   Let's Encrypt certificate that pin breaks on every renewal. Unset = full CA validation, and the
   module pins on first contact (TOFU) into its own table.
4. **`SIGNING_KEYS_PATH` is a K8s-only setting.** Its mere presence switches the module to disk
   mode and **disables the JWKS cron** this environment depends on.
5. **`auto_install` covers a fresh database and nothing else.** Three cases, and only the first
   is automatic: a **new** database installs both modules by itself (`initialize()` recurses); a
   **restored** one (`pg_restore` from odoo.sh) installs neither until something names them —
   that is what `requiredModules` exists for on K8s, and the manual step on Docker; and an
   environment that **already had `dyncloud_auth`** installs `dyncloud_api` under no
   circumstances, because the auth module only upgrades and never re-enters `'to install'`.
   K8s `requiredModules` deliberately lists only `dyncloud_auth` today — fine while every new
   environment is built fresh, wrong the day one of the other two cases appears
   (`references/modes/k8s.md`).

## Delegates to

| Skill | Purpose |
|-------|----------|
| `developer-skills:git-manager` | branch / commit / MR on the environment and module repos |
| `common-skills:dyncloud-mcp` | using the MCP once the environment is connected, and verifying it |
| `common-skills:dynbrain` | how to query the documentation MCP |
| `devops-assistant` | **Mode B**: the router for everything the migration touches beyond the connector — cluster, GitOps, secrets, observability |
| `gitops-change` | Mode B, when it is implemented: infra changes the GitOps way |

## Consumed by

| Skill | Purpose |
|-------|----------|
| `devops-assistant` | routing |
| `dyncloud-development` | when the module itself must be ported to a new series |

## REFERENCES

- `references/modes/docker.md` — Mode A, step by step, with the checks at each stage.
- `references/values-and-sources.md` — every variable and where its value is read from.
- `references/troubleshooting.md` — the silent failures and how to tell them apart.
- `references/modes/k8s.md` — Mode B: what is known, what is missing.
- `references/shared/danger-gates.md` · `references/shared/docs-access.md`
