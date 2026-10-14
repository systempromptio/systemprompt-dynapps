# Write tools — `platform_ops`

Everything here changes something. On a production stage the whole category normally sits
behind a JIT grant (`access-model.md`), and every one of these refuses while the project's
queue is blocked.

**Enqueued is not done.** Most of these return `action_ids` and stop. Follow through with
`platform_list_actions`, and on k8s with `platform_job_runs`, before you tell anyone it
worked.

Shared conventions in `README.md`.

---

## platform_deploy

The environment lifecycle wizards — the same ones the UI drives, not a reimplementation.

Input: `{ "op": "create" | "fork" | "upgrade_release" | "image_change" | "scale", … }`

The permission is resolved **per op**, so being allowed to scale says nothing about being
allowed to create:

| op | Permission | Needs |
| --- | --- | --- |
| `create` | `:environment_create:` | `project_id`, `environment_name`, `stage_id`, `release_tag_id` |
| `fork` | `:environment_fork:` | `source_environment_id`, `environment_name`, `stage_id` |
| `upgrade_release` | `:k8s_image_change:` | `release_tag_id`, `target_environment_ids` |
| `image_change` | `:k8s_image_change:` | `image_build_ids`, `target_environment_ids` |
| `scale` | `:k8s_tune:` | `environment_id`, `scale_lines` |

An op that does not apply to the environment's runtime answers
`E_NOT_SUPPORTED_FOR_RUNTIME` — `scale` is k8s-only, for instance.

**`upgrade_release` requires a `release_tag_id` and `list_release_tags` is the only tool
that produces one.** Read it first, take the id, and check `is_current` so you are not
enqueueing a no-op.

**`image_change` has no equivalent discovery tool for `image_build_ids`.** The build domain
is scoped per deploy line rather than per environment, so there is no flat
environment-keyed list to read. You need the ids from elsewhere — usually the UI.

---

## platform_lifecycle

Input: `{ "environment_id": 3, "operation": "stop" | "start" | "restart" }`.
Permission `:environment_lifecycle:`.

**The backend is chosen by the runtime — never guess the mechanism.** Docker enqueues a
`docker_action`; k8s enqueues a `k8s_lifecycle` with its components, derived from the
environment. On docker, components do not apply and are ignored; on k8s an empty component
selection is refused.

A `restart` is **not** how you apply a configuration change on docker — see
`platform_rebuild`.

---

## platform_rebuild

**Docker only** — `E_NOT_SUPPORTED_FOR_RUNTIME` on k8s. Permission `:docker_rebuild:`.

Regenerates the docker-compose extend and recreates the containers with it. **Not
destructive**: no `down --rmi all`, no volumes touched.

This is the way to apply a configuration change, and the reason it exists as its own tool:
a plain `restart` does not pick changes up, because it never goes through
`put_compose_extend`. If someone changed configuration and "restarted" and nothing
happened, this is the missing step.

---

## platform_domains

Input: `{ "environment_id": 3, "op": "create" | "update" | "remove" | "apply", … }`.
Permission `:environment_dns:`.

`create` / `update` / `remove` are ORM writes on `cloud.fqdn`; validation comes from the
model, not from this tool, so a rejected value is the model talking.

**`apply` is where the runtimes diverge.** Docker creates the zone records
**synchronously**. K8s enqueues `k8s_update_domains` and returns an `action_id` — the
effect lands through gitops, later, and "the call returned" is not "DNS changed".

Read `platform_list_domains` first: on k8s it reports the release's available purposes,
which is what a write needs to name a valid `purpose_name`.

---

## platform_submodules_apply

Applies a submodule plan by the same contract the UI uses. Read
`platform_submodules_plan` first — the `intro` tells you, in the engine's words, what
confirming does.

**The answer shapes differ, deliberately, and there is no fake symmetry:**

- **docker** — enqueues the per-submodule work plus **one restart action**, returns
  `action_ids`.
- **k8s** — patches `dyn-build.yaml` and commits. No enqueue: returns `branch` and
  `commit_url`, with `action_ids` empty.

So on k8s a successful apply has changed a repository and nothing else yet; whatever picks
that commit up runs on its own schedule. Do not report a deployed change.

Permission is the scope's own check, not the tool's.

---

## platform_backup_create

Input: `{ "environment_id": 3, "method": "auto" | "manual" }`. Permission
`:backup_create:`.

Creates a backup through the environment's own backup engine — `docker_yaml` or `k8s_job`,
resolved from the environment, never chosen by the caller. Refuses on a blocked project.

**It takes no other parameters.** There is no way through this tool to ask a k8s backup to
skip the filestore or to write somewhere else; the job's parameters are all Cloud-resolved.

Returns `action_ids`. On k8s, confirm with `platform_job_runs` — the action will say `done`
as soon as the job is dispatched.

---

## platform_action_control

Input: `{ "environment_id": 3, "action_id": 222, "op": "requeue" | "cancel" }`.
Permission `:project_actions:`.

**The way out of a blocked project queue.**

- **`requeue`** re-runs the chain. It re-renders from the environment's *current*
  configuration, so if you fixed the cause — a release bump, a corrected parameter — the
  retry picks the fix up. If you did not, it fails the same way.
- **`cancel`** gives up on the chain so the queue can move past it.

Requeue of a `k8s_destroy_*` action is unavailable due to a known platform defect and
returns a typed error; escalate that to a human rather than working around it.

---

## platform_enqueue_action

Input: `{ "environment_id": 3, "action_type": "...", "payload": { … } }`.

The escape hatch for action types with no dedicated tool. Deliberately narrow:

- **Destructive types are refused.**
- **Steps of a larger chain are refused** — you cannot hand-run half a deployment.
- **Types already covered by a dedicated tool are refused**, and the error names the tool
  to use instead.

The payload is validated against the action type's JSON-Schema with
`additionalProperties: false`, and a violation comes back verbatim, naming the offending
key. Read `platform_describe_actions` first for the schema.

---

## platform_rename_project

Input: `{ "project_id": 12, "name": "..." }`. Permission `:project_settings:` **and**
membership of the *Project Rename* group.

Renames `name` and `technical_name` together through `cloud.project._rename()` — the same
path the UI wizard uses. The tool does not `sudo()`, so both the group and the permission
apply exactly as they would to a human.

**Refused on a k8s project that already has a storage bucket.** The bucket name is derived
from the technical name, and the backup job recomputes it; renaming would leave the job
asking for a bucket nobody created, which is an outage that looks like a green backup.
Returns the project id, its previous name and its new one.

---

## platform_project_deploy_prepare / platform_project_deploy_commit

**Creating a project, in two calls with a human confirmation between them.** Both drive
`cloud.project.deploy` — the same wizard the UI uses — so every computed field, default
and validation is the wizard's own and not a reimplementation.

**`platform_project_deploy_prepare` is the discovery tool.** Call it with what you know
and it answers with what you may still choose: the allowed providers, the git platform
(derived from the release, on k8s), the repository paths, the DNS zone. Call it again with
more values to narrow further — it is a form being filled in, not a one-shot. Once a
`release_tag_id` is set it **generates the first environment line with its domains**, which
you review rather than invent.

**`platform_project_deploy_commit`** takes the `wizard_id` and calls `action_deploy()`.
It returns the project and its enqueued chain.

> **`action_deploy()` IS NOT REVERSIBLE.** It creates repositories, an S3 bucket, ArgoCD
> credentials and the project record. Put the resolved plan on screen — project name,
> technical name, release, cluster, the repository URLs it derived, the environments and
> their domains — and get a human yes before committing. The cost of a wrong name is not a
> rollback: `platform_rename_project` refuses on a k8s project that already has a bucket,
> which this one will have.

Four failure shapes that all look like something else:

- **Until `name` AND `provider_id` are both set the wizard cannot be saved.** Those two are
  required with no defaults. `prepare` returns a *virtual* wizard meanwhile — same payload,
  `wizard_id: null`, `persisted: false`. A resumable id appears only once both are known,
  and only then can the environment step generate lines.
- **A `wizard_id` expires.** `ir.autovacuum` sweeps transients, so a wizard prepared before
  a meeting may not outlive it: `E_DEPLOY_WIZARD_EXPIRED`. Start again with `prepare`; do
  not go looking for a different id.
- **No production stage, no environments — and it will not say so.** On k8s the `main` line
  is generated only when `stage_ids` contains a stage whose `stage_type` is `production`.
  With no match it creates **nothing** and still advances, so `prepare` answers with an
  empty environment list that reads like a successful call. `stage_ids` defaults to the
  stages flagged `is_default`, which is not the same set.
- **`Cloud: User` minimum.** `Cloud: External` cannot deploy anything; an `AccessError`
  here is the policy working.

**Creating a DOCKER project has a gap:** `allowed_branches_ids` never populates through the
tool, because the wizard only fetches template branches while unsaved — i.e. inside a live
UI session. Pass `template_branch_id` explicitly, or use the UI.

**Close the action** returned by commit before anything else touches the new project. The
repositories, the bucket and the branches are created by that chain, so a failure here is
not "the project is slightly incomplete" — it is a half-built project every later step will
assume is finished.

---

## platform_set_deploy_config

Input: `{ "environment_id": 3, "image_key": "", "deploy_policy": "...",
"deploy_target_id": 7 }`.

Wires an environment to the branch it deploys from and decides whether a push deploys it.
Read the current wiring with `platform_deploy_config` (`tools-read.md`).

| Stage | `deploy_policy` |
|---|---|
| development / staging | `autodeploy` |
| production | `manual` |

The default is `manual`, so staging and development are the ones that need changing.

**This is a plain ORM write and enqueues nothing.** It returns `changed`, not `action_ids`,
so there is no action to close.

Two deliberate refusals. It **never creates** a row: a missing one answers
`E_DEPLOY_CONFIG_NOT_FOUND` and lists the `image_key`s that do exist, because creating the
wiring belongs to the deploy wizard that owns the invariants tying a target to a
repository. And a write that moved nothing answers `changed: {}` rather than a success
indistinguishable from a real change.

> **Read the policy before pushing to a shared branch.** If `deploy_policy` is
> `autodeploy`, the push IS the deploy — and if several environments follow that branch,
> one push deploys all of them.

---

## Declaring a value the Tune wizard cannot introduce

A recipe worth knowing, because the obvious route silently does nothing.

The k8s Tune wizard cannot **add** a key that is absent from `values.yaml`. It fills the
row's current value from the release catalogue's `defaultValue` when the key does not
exist, then drops the row at commit time because "nothing changed" — so typing the default
and pressing Apply produces no commit, no error and no warning. Declaring an inherited
value through the UI is therefore impossible.

`platform_enqueue_action` reaches the same handler directly, past that filter:

```json
{
  "environment_id": 699,
  "action_type": "k8s_scale",
  "payload": {"scale_params": [{
    "path": "postgresql.internal.parameters.max_connections",
    "value": "100",
    "name": "PostgreSQL max_connections",
    "value_type": "string"
  }]}
}
```

Same machinery as the wizard — `commit_values_patches`, the same commit format, the same
ArgoCD refresh — and **idempotent**: no commit is emitted when `values.yaml` already
matches, so a re-run is free.

> **`value_type` is not optional.** Without it the legacy heuristic coerces `"100"` to the
> integer `100`. CloudNativePG's `parameters` map is string-to-string, the CRD rejects an
> integer, and the Application stops converging — the failure the declared-type work exists
> to prevent. The types that MUST reach `values.yaml` as strings are `string`, `storage`,
> `cpu` and `duration`; only `integer` and `boolean` are coerced. Kubernetes normalises
> every Quantity to a string, so an integer `2` can never converge against a live `'2'`.

Take `value_type` from the release catalogue's `valueType` for that scale component, not
from what the value looks like.
