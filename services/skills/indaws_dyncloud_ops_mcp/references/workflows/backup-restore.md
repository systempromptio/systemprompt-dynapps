# Workflows — data in and out

Taking backups, handing them over, and putting one back. Tool details in `../tools-read.md`
and `../tools-destructive.md`; the index is `README.md`.

A restore overwrites a database. Say what you are about to do, and to which environment,
before you do it.

---

## 1. Take a backup and prove it landed

```text
platform_backup_create { environment_id, method: "manual" }
  → { ok: true, action_ids: [221] }
```

That is a dispatch, not a backup. Two more calls turn it into a fact:

```text
platform_job_runs { environment_id, limit: 1 }
  → job_state: "succeeded", duration_seconds: 78,
    artifacts: { backup: { s3Bucket: "…", s3Key: "…", sizeBytes: 4502374 } }

platform_list_backups { environment_id }
  → manual.rows[0]: { id: 66, name: "20260820_101858.tgz", size_pretty: "4.29 MB" }
```

**Check the size against the environment.** A backup an order of magnitude smaller than
you expect is the interesting kind of success. `artifacts` tells you the bucket and key it
really wrote to, which is also how you learn the bucket name if you later need it.

On docker there is no `platform_job_runs`; verify with `platform_list_backups` and the
action history.

Note `platform_backup_create` takes no other parameters — there is no way through it to
skip the filestore or redirect the destination.

---

## 2. Hand a backup to someone

```text
platform_list_backups { environment_id }        # find the id, check `can.download`
platform_backup_url { storage_object_id: 66 }
  → { url: "https://…", ttl_seconds: … , expires_at: "…" }
```

**That URL is a bearer credential over customer data.** Anyone holding it can download the
database. Give it to the person who asked and nobody else, never paste it into a ticket or
a channel that outlives it, and never cache it — the answer tells you when it dies and it
means it.

Say the expiry out loud when you hand it over. A link that stops working an hour later
looks like a broken tool if nobody was told.

---

## 3. Restore into an environment

The full shape, because it has more moving parts than it looks.

```text
platform_list_backups { environment_id }         # which backup, and can you restore?
  → can.restore: true

platform_backup_restore { environment_id, storage_object_id: 66 }
  → E_CATEGORY_NEEDS_GRANT          ← nothing was prepared; this is normal

request_mcp_access { environment_id, scope: "category",
                     category: "platform_destructive", duration_hours: 2,
                     reason: "Restore backup 66 into <env> — overwrites its database, requested by <who>" }
  → state: "pending", auto_approved: false, approver_count: 4
```

**Stop here.** Tell the user the grant's `name` and what it authorises. You cannot approve
it from the MCP; a human signs it in the web UI. Then:

```text
get_my_mcp_access { environment_id }    → effective_state: "active"
platform_backup_restore { environment_id, storage_object_id: 66 }
  → { ok: true, mode: "simple", action_ids: [229],
      chain: ["k8s_env_quiesce_for_job", "k8s_await_argocd_hib", "k8s_job_launch"] }
```

Now watch it, and know what you are watching:

```text
platform_job_runs { environment_id, limit: 1 }
  → job_state: "succeeded",
    artifacts: { restore: { filestoreFilesRestored: 26, dbname: "odoo", … } }
```

`filestoreFilesRestored` is the number worth reading: it proves the filestore half of the
archive was actually processed, not just the database.

**Then confirm the environment came back.** The un-quiesce is enqueued by the webhook when
the job reports, so it is a separate step from the chain you were given:

```text
platform_env_status { environment_id }   → running again, no red health issue
```

**If the job never reports, the environment stays down.** That is the failure mode to watch
for: a restore that hangs is also an outage. Go straight to `platform_job_runs`, and if the
chain is wedged, §3 of `diagnose.md`.

When you are done: `revoke_mcp_access`.

---

## 4. Restore from a URL, or size the job by hand

Two cases the plain restore cannot cover: the archive lives outside the platform, or the
scratch volume needs to be bigger than Cloud would guess.

Both travel in `params`:

```text
platform_backup_restore {
  environment_id, storage_object_id: 66,
  params: { jobStorage: "25Gi", sourceUrl: "https://…" }
}
  → mode: "parametrised"
```

- **`jobStorage`** overrides the scratch-volume size. Cloud otherwise sizes it from the
  source object when it knows it, and from the environment's own volumes when it does not —
  which is exactly what it cannot do for a URL pointing somewhere else. A Kubernetes
  quantity: `25Gi`, `500Mi`, `1Ti`. Note `Gi` is 1024-based and `G` is 1000-based.
- **`sourceUrl`** takes precedence over the stored object when both are present. Use
  `platform_backup_url` to make one from a backup the platform already holds.

**Verify before trusting `"mode": "parametrised"`.** On manager builds before the fix that
threads `params` through the dispatcher, that string is printed while the values are
dropped — the restore runs with the defaults and nothing says otherwise. If you have
cluster access, the provisioned volume is the honest check; otherwise check the manager
version.

Parametrised mode is k8s-only. On docker, `params` raises rather than being ignored — which
is the correct behaviour, and the reason you can trust the refusal.
