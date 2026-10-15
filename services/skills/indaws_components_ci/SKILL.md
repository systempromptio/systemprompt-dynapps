---
name: indaws-components-ci
description: >-
  The shared CI systems of the Dyncloud platform, starting with the client CODE-repo pipeline (not the
  gitops repo) — the pipeline-components, the base-image build and the skeleton/Copier that stamps a
  uniform .gitlab-ci.yml. Use for pinning/versioning a component, understanding a client repo
  pipeline, or its gotchas; the component names, tags, images and repo paths are read live from the
  documentation MCP, never from memory. Designed to grow with other CI systems. Triggers: "gitlab ci",
  "pipeline", "pipeline-components", "odoo-base", "skeleton", "copier", "job-token", "client repo ci".
version: 0.2.2
effort: medium
model: sonnet
allowed-tools: [Read, Bash, Grep, Glob]
---

# Components CI

The platform's **shared CI systems**. Today it covers the **client CODE-repo pipeline** (the one
that makes every client repo have the same pipeline). Designed to **keep adding** other CI systems.
The live detail is read from the documentation MCP (`references/shared/docs-access.md`).

## ⚠️ Safety rules

Detail in `references/shared/danger-gates.md`. Here: changes to components/images = **branch → MR**
in their repo (never direct push to `main`; delegate to `git-manager`); **never** secret values in
the `.gitlab-ci.yml` or in variables (they go masked in GitLab); **never invent** a component/tag
name — verify it in the repo or look it up. Who can merge the MR depends on the user's **Azure AD
group** → `references/shared/access-model.md`.

## What it is / when

Use me to understand/touch the **client code-repo CI**: how it is stamped, which stages it runs,
how a component is versioned or the base image is pushed. **Scope = the code-repo pipeline**;
environment/fleet onboarding or GitOps changes go in `gitops-change`.

## Platform detail: ask the documentation MCP

Which pipeline-components exist and where they live, the base image and its build, the
skeleton/Copier template, component tags, stage names and repo paths — that is **documentation,
read at the moment it is needed**. A copy in this file is a copy that goes stale.

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
  rules, `Flow`, gotchas), and **mark explicitly what stays unconfirmed**. Never invent a
  version, a host or a path.

## Gotchas

- **Never hardcode the name of the MCP server.** It is the user's to choose and it differs per
  machine; identify it by the `mcp__*__docs_*` tool family and read the real name at runtime.
  Same for the documentation space: match it by name, never by numeric id.
- **New client repo fails at `pre-commit`/`build`**: almost always the **client's group** is
  missing from the **job-token** `groups_allowlist` of the infra images → `403` in
  `before_script`. Add the group to the allowlist.
- **`webhook` job in red** is usually **cosmetic** (it notifies an endpoint); look at
  `run-odoo-tests`/`pre-commit`/`build` for the real verdict.

## Flow
1. **Step 1** — a client code repo inherits the pipeline through the **skeleton/Copier** (includes the pipeline-components).
2. **Step 2** — to version/pin a component: MR to the component's repo + re-pin in the central pipeline config.
3. **Step 3** — if a new repo/group fails at `before_script` (403), check the job-token's **`groups_allowlist`**.

## Delegates to
| Skill | Purpose |
|-------|----------|
| `developer-skills:git-manager` | branch / MR |
| `common-skills:dynbrain` | how to query the documentation MCP (retrieval, tool choice, navigation) |

## Consumed by
| Skill | Purpose |
|-------|----------|
| `devops-assistant` | routing |
| `dyncloud-development` | the code-repo pipeline |

## REFERENCES
- `references/shared/danger-gates.md` · `references/shared/docs-access.md` · `references/shared/access-model.md`
- Live platform detail: the documentation MCP (`mcp__*__docs_*`), as `common-skills:dynbrain` describes.
