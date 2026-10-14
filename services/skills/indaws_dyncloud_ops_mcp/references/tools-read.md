# Read tools — looking without touching

Everything here is read-only and normally `open` on every stage, so it needs no grant. Read
before you write; most of these cost nothing and several will change your plan.

Shared conventions in `README.md`.

---

## The three layers of state

The single most useful thing in this file. State lives in three places and each is blind to
the next.

```text
platform_env_status    → what the environment is doing, and is its queue blocked
platform_list_actions  → which step of the chain ran, and the traceback of the one that broke
platform_job_runs      → what the job did INSIDE the cluster: its state, its error, its logs
```

**The seam is between the second and the third.** A k8s job is dispatched by a chain whose
final step is "launch the job", and launching is that step's entire responsibility. So the
chain says `done`, the action says `done` — and that says the job *started*, not that it
worked. If you stop at layer two you will report success for a job that died seconds later.

Go down a layer whenever the answer above looks fine and the world does not.

---

## platform_env_status

Input: `{ "environment_id": 3 }`.

The action in progress, its last progress message, the environment's health issues, and
whether the project's queue is blocked.

```json
{ "ok": true, "environment_id": 3, "name": "main", "stage_type": "production",
  "action_id": 221, "action_state": "done",
  "action_status_msg": "Job backup: succeeded",
  "last_action_summary": { "…": "registries, progress ring, actor" },
  "health_issues": [ { "code": "unprotected", "severity": "yellow",
                       "label": "Deletion protection expected by stage but disabled" } ],
  "project_blocked": false }
```

Two fields deserve attention:

- **`project_blocked`** — read it rather than assuming "no output" means healthy. A blocked
  project runs nothing new until a human intervenes.
- **`action_status_msg`** — free text written as work progresses. **When it disagrees with
  the state beside it**, believe the message: `action_state: "done"` next to
  `"Job backup: failed"` is a job that failed under an action that only ever promised to
  launch it. That disagreement is your cue to open `platform_job_runs`.

---

## platform_list_actions

Input: `{ "environment_id": 3 }` or `{ "project_id": 12 }`, `limit` (default 20),
`detail: true` to include each registry's payload.

Per registry step: `effective_state`, terminal execution time, the exception name and
message, `exc_info` (the full traceback) and `exc_info_available`.

Two details that save confusion:

- **Always read `effective_state`, never a raw `state`.** The underlying `queue.job` rows
  are vacuumed, and a raw state outlives its truth.
- **`exc_info_available` is computed, not stored**, and distinguishes three situations that
  otherwise look identical: the job is alive and carries an exception (`exc_info` present);
  alive with no exception (`exc_info` null, `available` true); vacuumed away (`exc_info`
  null, `available` **false** — the trace existed and is lost). `terminal_exc_name` and
  `terminal_exc_message` are own columns and survive the vacuum, so a failure is still
  named long after its traceback is gone.

`detail: true` returns the payload each step ran with — deployment configuration, the job
run id, a quiesce snapshot. That is why the payload lives here and not in `list_actions`.

---

## platform_job_runs

**k8s only.**

Input: `{ "environment_id": 3, "limit": 20 }`.

The runtime status of jobs actually dispatched to the cluster — the layer the two tools
above cannot see.

```json
{ "id": 89, "job_type": "restore", "kind": "restore", "trigger_method": "restore",
  "job_state": "succeeded",
  "started_at": "…", "completed_at": "…", "duration_seconds": 110,
  "error_message": null,
  "logs_preview": null,
  "artifacts": { "restore": { "s3Bucket": "…", "s3Key": "…",
                              "filestoreFilesRestored": 26, "dbname": "odoo" } },
  "source_storage_object_id": 66,
  "correlation_id": "…", "dyn_job_name": "…", "job_name": "…" }
```

- **`error_message`** is the operator's own failure text — a Kubernetes API 422, a schedule
  failure, whatever actually happened. It is stored the moment it happens and it is the
  first thing to read when a chain looks fine and the result does not.
- **`logs_preview`** is the tail of the job's output, capped and marked when cut. A job
  says why it died at the end, which is why the tail is what survives.
- **`artifacts`** is what the job produced — for a backup the bucket, key and size; for a
  restore the file count. Useful as proof, not just as diagnosis.
- **`dyn_job_name` / `job_name` / `correlation_id`** are the cluster-side handles, for
  cross-referencing `kubectl` or the operator's logs when you have that access.

Newest first. A **negative `duration_seconds`** means the job died while its Job object was
being created: `completed_at` comes from the cluster, `started_at` from the manager, and
they can invert. Read it as "failed immediately".

---

## platform_project_health

Input: `{ "project_id": 12 }`.

Health and fleet KPIs for a whole project at a glance: environments without a backup,
unprotected production, scheduled deletions, fleet counts. Scoped to what the caller may
see, so the numbers are their view, not the absolute truth.

Use it to size up a project you have not seen before, and before promising a customer that
"backups are running".

---

## platform_list_backups

Input: `{ "environment_id": 3 }`.

Automatic and manual backups with size and date, plus **the caller's own permissions on
them** (`download` / `restore` / `destroy` / `create`), so you can tell in advance whether
a restore is even possible for you.

```json
{ "auto": { "rows": [], "active": true, "interval_number": 1,
            "interval_type": "days", "limit": 10 },
  "manual": { "rows": [ { "id": 66, "name": "20260820_101858.tgz",
                          "date": "…", "size": 4502374, "size_pretty": "4.29 MB",
                          "download_url": "/cloud/api/storage/download/66" } ] },
  "can": { "download": true, "restore": true, "destroy": false, "create": true } }
```

**An empty `auto.rows` is worth a second look.** Rows are discovered by listing the bucket,
so a bucket rename leaves the old objects where they are and invisible here. Empty can mean
"no backups are being taken" **or** "they are somewhere this environment no longer points
at". Both are worth telling the user; they are not the same problem.

---

## platform_backup_url

Input: `{ "storage_object_id": 66 }`. Requires `:backup_download:`.

A presigned download URL for one backup. It is **a bearer credential over customer data**:
hand it to the caller and nobody else, and treat it as short-lived — the answer carries
`ttl_seconds` and the expiry instant. Never cache it, never paste it somewhere durable.

Also the input for a restore from a URL, when the manager build supports passing `params`.

---

## platform_list_domains

Input: `{ "environment_id": 3 }`.

The environment's domains: names, URLs, DNS state, TLS/CORS/noindex flags, allow and deny
lists, redirect configuration, effective parameters. On k8s it additionally returns the
**release's available purposes**, which is what a subsequent write needs in order to name a
valid `purpose_name`. Read it before `platform_domains`, not after.

---

## platform_describe_jobs

**k8s only.**

Input: `{ "environment_id": 3 }`.

The jobs this environment's **release** declares, and which are eligible right now given
its phase.

- `parameter_schema` is **not** JSON Schema — a small custom dict of `type` / `enum` /
  `required` / `default` / `label`, with no `additionalProperties`.
- `cloud_resolved_params` lists the keys Cloud fills in itself. They are not yours to set,
  and a write tool that lets you send them will override you anyway.
- `restore_available: false` means the release declares no restore job, so the environment
  **cannot be restored at all** — worth knowing before promising one.

Descriptions come from the release, so their accuracy is the release author's, not the
platform's. Prefer what the schema says over what the prose says.

---

## platform_describe_actions

Input: `{}`.

The catalogue of every `action_type` and the JSON-Schema of its payload, read from the live
registry at call time — downstream modules register their own types at import, so a
hardcoded list would be stale.

Read it before `platform_enqueue_action`, which validates against exactly these schemas
with `additionalProperties: false`.

---

## platform_submodules_plan · platform_submodules_refs

Input: `{ "environment_id": 3 }` (or a project scope).

`platform_submodules_plan` returns the engine's plan **verbatim** — `engine`, `scope`,
`rows`, `capabilities`, `intro`, `warning` — and never a normalised form, because the two
engines do genuinely different things and flattening them would hide it. The `intro` says,
in the engine's own words, what confirming will do: rewrite `.gitmodules` and restart on
docker; patch `dyn-build.yaml` and commit on k8s.

**Pass `warning` through, never summarise it.** It distinguishes "this scope has no
submodules" from "the manifest could not be read", and those call for opposite reactions.

`platform_submodules_refs` lists one page of branches or tags for one row, so a ref can be
chosen before it is pinned. Returns the adapter's `items` / `has_more` / `truncated`
untouched; GitLab searches server-side, GitHub cannot and reports a capped scan through
`truncated`.

---

## platform_deploy_config

Input: `{ "environment_id": 3 }` — or `{ "project_id": 12 }` to see every environment of a
project in one call.

Reads the deploy wiring: the deploy target (whose **name IS the branch**, with
`target_type` saying whether it is a release, hotfix or feature branch), the
`deploy_policy`, and the current image per `image_key`.

**`autodeploy_count` comes back at the top level**, so you can branch on one number instead
of walking the list. Read this before pushing to any shared branch: where the policy is
`autodeploy`, the push is the deploy.

An environment with no rows answers `rows: []` with a note saying so — which is not the
same as failing to read it. Writing the wiring is `platform_set_deploy_config`
(`tools-write.md`).

> **The `current_image.id` here is a `cloud.image`, NOT a `cloud.build`.** Feeding it to
> `platform_deploy(image_build_ids=[…])`, which wants build ids, fails with a foreign-key
> violation and an HTTP 500 rather than a validation error — the two id spaces overlap, so
> the number looks plausible right up to the traceback.
