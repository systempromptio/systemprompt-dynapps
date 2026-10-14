---
name: indaws-gitops-change
description: >-
  Make ANY change to the Dyncloud infrastructure the GitOps way — via a branch and MR to the infra
  repos, never with a direct helm upgrade or kubectl apply. Also covers publishing the documentation
  sites (public / platform / base): edit and build the mkdocs repo, then deploy the image the GitOps
  way. Which repo and which file hold a given setting is read live from the documentation MCP, never
  from memory. Use when the user wants to deploy, bump, roll, or reconfigure something on the cluster,
  onboard a new environment/fleet entry, or update/publish the docs. Triggers: "deploy", "bump image",
  "roll operator", "change values", "add environment", "gitops", "argocd sync", "publish docs",
  "update documentation", "mkdocs".
version: 0.2.2
effort: medium
model: opus
allowed-tools: [Read, Bash, Grep, Glob, Edit, Write]
---

# GitOps Change

**GitOps is the only path** to the cluster. Every change goes via **branch → MR → merge → ArgoCD
refresh**, never by hand. The exact detail (repos, paths, names, versions) is read **live** from
the documentation MCP; never make it up.

## ⚠️ Safety rules

They apply **always** (detail in `references/shared/danger-gates.md`):

- **Forbidden** `helm upgrade`, `kubectl apply/edit/delete`, `docker build && push` **directly** to the
  cluster. The desired state lives in Git.
- **`kubectl` READ-ONLY**; the only write allowed = ArgoCD's `annotate … refresh=hard`.
- Protected branches **MR-only** (`main` of the infra repos, `19.0` of the manager): **never** direct
  push or `--force`. The branch/MR flow is delegated to **`developer-skills:git-manager`**.
- If there really is no other option than a manual `kubectl` write → **ask for permission and explain blast-radius**.
- Who can merge/apply depends on the user's **Azure AD group** → `references/shared/access-model.md`.
- **NEVER make up** paths/versions: look them up; if you can't, say so.

## What it is / when

Use me to **change** something in the infra: deploy/update a component, bump an image,
roll the operator, tweak a cluster's values, **onboard an environment/fleet**, or **publish the
documentation** (the public / platform / base sites; see below). To just
**look** → `cluster-inspector`. For secrets → `secrets-openbao`. For the CI of the client code
repo → `components-ci`. To **hide a service from the public (mesh-only)** — Gateway
`envoy-internal` + NetBird peer-proxy, same URL — the pattern is in **`netbird-mesh`**.

## Platform detail: ask the documentation MCP

Which repo owns a setting and which file holds it, the GitOps chain, the app names, hosts,
versions and chart values — that is **documentation, read at the moment it is needed**. A copy in
this file is a copy that goes stale.

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
  rules, `The flow`, the publishing steps, the cut-off rule, gotchas), and **mark explicitly
  what stays unconfirmed**. Never invent a version, a host or a path.

## The flow (always the same)

1. **Locate the repo and the file**: ask the documentation which repo owns that setting and which
   file inside it holds it. **Do not guess a path** — with **MCP unavailable**, say you need it to
   name the file instead of reconstructing one.
2. **Branch from `origin/main`** (NOT from someone else's branch on the shared clone) → edit → **MR** →
   merge. Use `git-manager` for branch/precheck/MR.
3. **Apply**: `kubectl -n argocd annotate application <app> argocd.argoproj.io/refresh=hard --overwrite`
   (most apps are autosync selfHeal; the refresh speeds it up).
4. **Verify** (read-only, or delegate to `cluster-inspector`): that the app ends up `Synced/Healthy` and the
   resource at the expected value.

### Image bump (gotcha)

- Verify **pipeline SHA == commit** before bumping (otherwise → `ImagePullBackOff`).
- If the values pins **tag only** → change the tag. If it pins **tag + digest** → change **both**
  (k8s pulls by digest; the tag alone deploys old code).

## Publish documentation (public / platform / base)

Publishing docs is a GitOps change of **two halves**: (A) edit+build the site in its mkdocs
repo, (B) deploy the resulting image with the flow above. Ask the documentation which repo backs
the site you are touching before editing anything.

**Cut-off rule** (where to write): **concept → the base site** · **contract/command/config value →
the platform site** · **client-facing → the public site**. NEVER cross internal content into the
public repo: it is a security boundary, not a style preference.

**Half A — edit and build** (mkdocs repo, `main` **MR-only**):
1. Branch from `origin/main` → edit the `.md` (i18n **suffix**: Spanish base, `*.en.md` alongside).
2. **`mkdocs build --strict` green** (breaks on broken links; **mandatory**). A venv with
   the repo's `requirements.txt` is enough.
3. MR → merge to `main` → the **CI builds and pushes the image**, tagged with the **8-char short
   SHA** (+ `:latest`).

**Half B — deploy** (the GitOps flow above):
4. Bump the image **tag** in the docs deployment manifest of the per-cluster config repo (ask the
   documentation for the exact file) → MR → merge.
5. `annotate application <docs-app> … refresh=hard` → ArgoCD rotates the pod. Verify **200**.

## GOTCHAS
1. **Never hardcode the name of the MCP server.** It is the user's to choose and it differs per machine; identify it by the `mcp__*__docs_*` tool family and read the real name at runtime. Same for the documentation space: match it by name, never by numeric id.
2. **Image bump: tag AND digest**. If the values pins `digest`, k8s pulls by digest → changing only the tag deploys the old code. Change **both**.
3. The **short SHA is 8** (`CI_COMMIT_SHORT_SHA`), not 7 (the one from `git log --oneline`). A 7-char tag → `ImagePullBackOff` "not found".
4. **Verify that the pipeline SHA == the commit** before bumping (the probe may catch a previous pipeline).
5. ArgoCD per-app **does not auto-apply** values changes quickly → `annotate … refresh=hard`. A stuck sync unblocks when `progressDeadlineSeconds` fires.
6. `cluster-config`/`argocd-config` **have no CI**; validate ArgoCD at render.
7. **Docs — build**: `mkdocs build --strict` ALWAYS before the MR (breaks on broken links). Mermaid: labels with `<br/>` (not literal `\n`) and **quote** those carrying `: " ( )` — `--strict` does NOT detect it, it breaks in the browser (client-side render).
8. **Docs — content**: always brand **Dyncloud** (lowercase c) and **Dynapps España**; **never** internal content in the public docs repo (security boundary). The image tag is the **8-char SHA** (see gotcha 3).

## Delegates to
| Skill | Purpose |
|-------|----------|
| `developer-skills:git-manager` | branch / MR |
| `secrets-openbao` | secrets referenced in the change |
| `common-skills:dynbrain` | how to query the documentation MCP (retrieval, tool choice, navigation) |

## Consumed by
| Skill | Purpose |
|-------|----------|
| `devops-assistant` | routing |
| `incident-runbooks` | fix via GitOps |
| `cluster-inspector` | when action is needed |
| `netbird-mesh` | component deployment |

## REFERENCES
- `references/shared/danger-gates.md` · `references/shared/docs-access.md` · `references/shared/access-model.md`
- Live platform detail: the documentation MCP (`mcp__*__docs_*`), as `common-skills:dynbrain` describes.
