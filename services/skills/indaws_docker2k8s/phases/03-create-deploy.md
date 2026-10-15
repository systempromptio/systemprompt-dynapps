# Phase 3 — Create and deploy the Kubernetes project

> **docker2k8s — Phase 3 of 7.** Before starting: Phase 2 finished: the Docker project now carries the `-docker` suffix, so the customer's real name is free. You also need 4.0's answer — which environments migrate — because the wizard's second step is where they are declared.
> The router, the safety rules and the Prerequisites live in `SKILL.md`.


**AUTO in two calls, with a confirmation between them.** Both drive
`cloud.project.deploy` — the wizard the UI uses — so every computed field, default and validation is the wizard's, not a
reimplementation.

**The project takes the customer's REAL name** — the one § Phase 2 (`phases/02-rename.md`) just freed by moving
the Docker project to `<customer>-docker`. That is what the rename was for; do not
improvise a name here.

1. **`platform_project_deploy_prepare`** — call it with what you know and it answers with
   what you may still choose. **It is the discovery tool**: the wizard computes the
   allowed providers, the git platform (derived from the release, for k8s), the repo
   paths, the DNS zone, and once a `release_tag_id` is set it **generates the first
   environment line with its domains**. You review hostnames; you do not invent them.
   Call it again with more values to narrow further — that is the form being filled in.
2. **`platform_project_deploy_commit`** — takes the `wizard_id` and calls `action_deploy()`.
   It returns the project and its enqueued chain: **close the action** before Phase 4
   touches anything. The repos, the bucket and the branches are created by that chain, so
   a failure here is not "the project is a bit incomplete" — it is a half-built project
   that every later step will assume is finished.

> **`action_deploy()` IS NOT REVERSIBLE.** It creates repositories, an S3 bucket, ArgoCD
> credentials and the project record. Put the resolved plan on screen — project name,
> technical name, release, cluster, the repo URLs it derived, the environments and their
> domains — and get a human yes before calling commit. The cost of a wrong name is not a
> rollback: `_rename` may not be available to you (§ Phase 2 (`phases/02-rename.md`)).

> **Until `name` AND `provider_id` are both set, the wizard cannot be saved** — those two
> are required with no defaults. `prepare` returns a *virtual* wizard in the meantime:
> same payload, `wizard_id: null`, `persisted: false`. A resumable id appears once both
> are known, and only then can step 2 generate environment lines.

> **A `wizard_id` can expire.** `ir.autovacuum` sweeps transients, so a wizard prepared
> before a meeting may not outlive it. That answers `E_DEPLOY_WIZARD_EXPIRED` — start
> again with `prepare`, do not go hunting for a different id.

> **`Cloud: User` minimum.** The wizard had no access control at all until 2026-08-21;
> now `Cloud: External` cannot deploy anything. An `AccessError` here is the policy
> working, not a fault.

> **No production stage, no environments — and it will not say so.** For k8s, step 2
> generates the `main` line only when `stage_ids` contains a stage whose `stage_type` is
> `production`: the code reads `prod = stage_ids.filtered(... == "production")[:1]` and
> then `if prod:`. With no match it creates **nothing** and still advances to the
> environments step, so `prepare` answers with an empty environment list that looks like
> a successful call. `stage_ids` defaults to the stages flagged `is_default`, which is not
> the same set. **Check that a production stage is in there before trusting an empty
> list** — and if the list is empty, that is the first thing to look at.

> **Step 2 is where you say how many environments deploy on day one**, so § 4.0 (`phases/04-equivalence.md`)'s answer
> is needed HERE, not later: the generated `main` line can be edited, and more lines
> added, each with its own stage, cluster and domains — before commit. Anything not
> declared now is a separate deploy afterwards.

> **Creating a DOCKER project has one gap:** `allowed_branches_ids` never populates
> through the tool — the wizard only fetches template branches while it is unsaved, i.e.
> inside a live UI session. Pass `template_branch_id` explicitly, or use the UI. Does not
> affect this runbook, which creates k8s projects.

At this point the environment comes up on the **demo database the platform generates at
boot**. That is intended: Phase 5 checks the runtime is healthy before any customer data
goes near it.

**VERIFY:** the environment reaches a Running phase, and its URL serves the fresh Odoo.

> **VERIFY WHO OWNS IT, and do it now.** Open the new project and check two things: it is
> in the **customer's company**, and **you** are in its administrators. Both are written
> at creation from the identity the manager thinks is acting, and when that identity is
> wrong nothing fails — the deploy succeeds and the project simply belongs to somebody
> else.
>
> This is not hypothetical. On an older manager every MCP tool
> ran under the transport's superuser environment, so a project created this way came out
> with `create_uid = __system__`, in OdooBot's company, and with `admin_user_ids` pointing
> at OdooBot — which **reads as empty**, because that user is archived. The project looked
> like it had no administrator at all, and it did not appear in `list_projects` for the
> real company. If you see any of that, the manager is older than the fix: stop, say so,
> and get the ownership corrected before Phase 4 builds on top of it.

