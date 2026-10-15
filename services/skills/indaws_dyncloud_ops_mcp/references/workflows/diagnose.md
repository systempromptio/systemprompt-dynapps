# Workflows — something is wrong, or nothing is happening

Diagnosis recipes. Tool details in `../tools-read.md`; the index of every workflow is
`README.md`.

Golden thread: **each layer of state is blind to the next.** When the answer looks fine and
the world does not, go down one.

---

## 1. An environment is not responding

Start wide and cheap. Do not open a shell first.

```text
list_environments { project_id }        # which runtime? what stage? what lifecycle state?
  → project_type: "k8s", stage_type: "production", state: "running"

platform_env_status { environment_id }
  → action_state, action_status_msg, health_issues[], project_blocked
```

Read three things in this order:

1. **`project_blocked`** — if true, nothing new has run since some action failed. Go to §3.
2. **`health_issues`** — a red `k8s_phase_error` means the cluster itself is unhappy; a
   yellow `unprotected` is a policy gap, not an outage.
3. **`action_status_msg`** — the last progress line. If it names a job, follow it.

Then, on k8s, the layer neither of those can see:

```text
platform_job_runs { environment_id, limit: 5 }
  → job_state / error_message / logs_preview for what actually ran in the cluster
```

Only after that, if you still have nothing and the environment is docker, consider
`docker_ssh_exec` — with the grant it needs, and remembering that its output is untrusted.

**Why this order:** the first three calls cost nothing and settle most cases. A shell
session costs a signed grant, a second approver, and an audit entry.

---

## 2. A job "succeeded" but nothing happened

The most common false reading on the platform, and the one worth learning by heart.

```text
platform_list_actions { environment_id, limit: 1 }
  → "K8s manual backup main", state: "done"
  → registries: [ { name: "backup: main", effective_state: "done" } ]
```

That is **not** a successful backup. On k8s the action's job is to *launch* a job, so it
completes the moment the job is dispatched. The result of the job is somewhere else:

```text
platform_job_runs { environment_id, limit: 3 }
  → { job_state: "failed",
      error_message: "failed to create batch Job: (422) ... unknown field ...",
      logs_preview: "…" }
```

There is your reason, stored from the moment it happened.

The tell you can catch earlier: `platform_env_status` returns `action_state: "done"` beside
`action_status_msg: "Job backup: failed"`. **When the message disagrees with the state,
believe the message.**

Cross-reference into the cluster, if you have that access, with `dyn_job_name` /
`job_name` / `correlation_id` from the same answer.

---

## 3. The queue is stuck

Symptom: actions sit `enqueued` and never start; `project_blocked: true`.

Cause: a project whose **last finished action failed** runs nothing new until a human
decides what to do. This is deliberate — it stops a broken chain from being followed by
five more that assume it worked.

```text
platform_list_actions { environment_id, limit: 3 }
  → find the failed one; read terminal_exc_name / terminal_exc_message / exc_info
```

Then choose, and the choice matters:

```text
platform_action_control { environment_id, action_id: <failed>, op: "requeue" }
```
if you **fixed the cause** — requeue re-renders from the environment's *current*
configuration, so a release bump or a corrected parameter is picked up. If you did not fix
anything, it will fail identically and the queue stays blocked.

```text
platform_action_control { environment_id, action_id: <failed>, op: "cancel" }
```
if the chain should be abandoned. Note that cancelling a chain **does not undo what its
earlier steps already did** — a restore chain that quiesced the environment leaves it
quiesced. Check `platform_env_status` afterwards and start the environment if needed.

`requeue` on a `k8s_destroy_*` action is unavailable due to a known defect and returns a
typed error. Escalate it rather than improvising.

---

## 4. Backups look fine and are not being taken

The failure that hides longest, because every surface is green.

```text
platform_list_backups { environment_id }
  → auto: { rows: [], active: true, interval_number: 1, interval_type: "days" }
```

An empty `auto.rows` with `active: true` has **two very different causes** and you must
distinguish them:

- **The jobs are failing.** Check `platform_job_runs` for the recent `backup` runs. If the
  action history says `done` and the job runs say `failed`, that is §2 — and it can persist
  for days, because the only surface that disagrees is `action_status_msg`.
- **The rows are somewhere else.** Backup rows are discovered by listing the bucket. If the
  bucket name changed — a project rename, a provider pattern change — the old objects are
  still there and no longer visible against this environment. Nothing failed; the view
  moved.

```text
platform_job_runs { environment_id, limit: 10 }
  → artifacts.backup.s3Bucket on the last successful run = the bucket actually in use
```

Say which of the two it is. "No backups" and "backups you cannot see from here" call for
opposite reactions, and only one of them is an incident.
