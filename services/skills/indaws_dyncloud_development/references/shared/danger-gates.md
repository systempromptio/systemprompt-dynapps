# Danger gates — global safety rules

**These rules are inherited by EVERY skill in the `devops-skills` plugin.** Every skill links this
file from its `## ⚠️ Safety rules` block. When in doubt, the most conservative rule wins.

## Summary (always read it)

1. **GitOps is the only path.** No manual changes on the cluster.
2. **`kubectl` is READ-ONLY.** Only write allowed: the ArgoCD refresh (below).
3. **Unavoidable write → ask permission + explain blast-radius** before touching anything.
4. **Never read/expose secret VALUES** (OpenBao/ESO/OIDC). Paths yes; values no.
5. **Never touch the production v17 Odoo** (`:8099` / DB `asoo`).
6. **Protected branches = MR-only.** Never a direct push or `--force`.
7. **Ask about the kubeconfigs**, don't hardcode them.
8. **NEVER invent.** If a fact is missing and can't be fetched live, say what's missing and point to the source.

---

## 1. GitOps is the only path

The desired state lives in Git; **ArgoCD** (hub in `es-prod-01`) reconciles both clusters.
Never `helm upgrade`, `helm install`, `kubectl apply/edit/patch/scale`, nor `docker build && push`
by hand against the cluster. Every change = **branch → MR → merge** in the corresponding GitOps repo
(`cluster-config`, `argocd-config`, `fleet-registry`, `dyncloud-releases`, or the `tofu` repo of
`netbird-config` for the mesh). The branch/MR flow is delegated to `developer-skills:git-manager`.

## 2. `kubectl` is READ-ONLY

Allowed: `get`, `describe`, `logs`, `top`, `explain`, `api-resources`, `argocd app get/list`, etc.
**Only pre-authorized write** (without asking permission):

```
kubectl -n argocd annotate application <app> argocd.argoproj.io/refresh=hard --overwrite
```

Any mutation of cluster **resources** (`apply`, `edit`, `patch`, `delete`, `scale`,
`rollout restart`, `cordon/drain`, `annotate` other than that refresh) is **forbidden by
default** (it goes through GitOps).

**Exception by role — `kubectl exec` on Odoo pods**: `exec` for `odoo-bin shell`, viewing logs or
inspecting **does NOT mutate the cluster state** and is a **legitimate capability of the
`developer` (and `devops`) level** — see [`access-model.md`](access-model.md). Allowed for those
levels. **BUT** an `odoo-bin shell` against **production** can write business data:
treat it as a dangerous write (rule 3: warn about the blast-radius and confirm) before
running anything that modifies data.

## 3. Unavoidable write → permission + blast-radius

If there really is no GitOps alternative (e.g. an emergency one-shot), **STOP and ask for explicit
permission** from the user, explaining first:
- **What** exact command would run.
- **Blast-radius**: who/what it affects (an environment? a cluster? production?).
- **How to revert** and how it re-aligns with GitOps afterwards (so ArgoCD doesn't overwrite it and there's no drift).
Without an explicit "yes", it doesn't run.

## 4. Secrets: paths yes, values no

Never read or print secret **values**: `kubectl get secret -o yaml`, `bao kv get` of a
value, contents of a `.env`, etc. Especially **never** the Azure OIDC `client_secret` (neither
its value nor whether it's set). You may talk about **paths/keys** (e.g. `kv/infra/...`, the name
of an `ExternalSecret`), which is what goes in docs and config.

## 5. Never touch the production v17

The **v17** Odoo on `:8099` / DB `asoo` is **live production** (Indaws Cloud). Never a generic
`pkill` of Odoo, nor `-u`, nor write queries. Always filter by the correct target.

## 6. Protected branches = MR-only

`manager` branch `19.0` and the infra repos (`main`) are **protected**: never a direct `git push`
nor `--force`. Changes via `feature/…`/`hotfix/…` branch → **MR** (a Maintainer merges it). Before
pushing: `pre-commit` green + local tests. Delegate to `developer-skills:git-manager`.

## 7. Cluster access = ask

Access is via **Pinniped** (**Azure AD** auth) and travels over the **NetBird mesh**. The skill
**asks** the user where each kubeconfig is (`es-prod-01`, `es-dev-01`); it doesn't assume paths nor
hardcode them. If there's no mesh/kubeconfig → it says so and explains what it needs (see `docs-access.md`).
What each user **can do** depends on their **Azure AD group** (devops / developer /
readonly): see [`access-model.md`](access-model.md). Adapt what you propose to that level.

## 8. NEVER invent

If you don't have a fact (exact version, resource name, a line of a file…) and it can't be
**fetched live** from the docs (`docs-access.md`) nor verified in a repo, **say so** and point to the
**exact source** (repo/URL/command to obtain it). Zero assumptions presented as facts.

This includes hosts: with **MCP unavailable** you do **not** compose the full URL of a repo from
memory — you say that confirming it needs the documentation MCP.

## Naming and identity (in whatever we write)

- Brand in text: **Dyncloud** (never «DynCloud»), **Dynapps España** (never «Spain»/«Indaws»).
- Legacy technical identifiers (domains, `infra-es/dyncloud/…` paths) are kept as they are.
- Commits always with the identity already configured for that repo — never a default, an
  anonymous one, or one you made up. If it is not configured, ask; don't guess.
